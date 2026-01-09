use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use reqwest::{header, Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;
use tokio::time::timeout;

use crate::config::{get_app_config_dir, get_home_dir, write_json_file};
use crate::error::format_skill_error;

const MAX_SKILL_SCAN_DEPTH: usize = 32;
const DEFAULT_SKILL_CACHE_TTL_SECS: u64 = 0;

/// 技能对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// 唯一标识: "owner/name:directory" 或 "local:directory"
    pub key: String,
    /// 显示名称 (从 SKILL.md 解析)
    pub name: String,
    /// 技能描述
    pub description: String,
    /// 目录名称 (安装路径的相对路径，可能包含子目录)
    pub directory: String,
    /// 父目录路径 (相对技能根目录，包含嵌套信息)
    #[serde(rename = "parentPath", skip_serializing_if = "Option::is_none")]
    pub parent_path: Option<String>,
    /// 嵌套深度 (0 表示直接位于技能根目录)
    #[serde(default)]
    pub depth: usize,
    /// GitHub README URL
    #[serde(rename = "readmeUrl")]
    pub readme_url: Option<String>,
    /// 是否已安装
    pub installed: bool,
    /// 仓库所有者
    #[serde(rename = "repoOwner")]
    pub repo_owner: Option<String>,
    /// 仓库名称
    #[serde(rename = "repoName")]
    pub repo_name: Option<String>,
    /// 分支名称
    #[serde(rename = "repoBranch")]
    pub repo_branch: Option<String>,
    /// 技能所在的子目录路径 (可选, 如 "skills")
    #[serde(rename = "skillsPath")]
    pub skills_path: Option<String>,
    /// workflows 中的命令
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<SkillCommand>,
}

/// 技能 workflows 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCommand {
    /// 命令名称
    pub name: String,
    /// 命令描述
    pub description: String,
    /// workflow 文件路径 (相对技能目录)
    #[serde(rename = "filePath")]
    pub file_path: String,
}

/// 仓库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRepo {
    /// GitHub 用户/组织名
    pub owner: String,
    /// 仓库名称
    pub name: String,
    /// 分支 (默认 "main")
    pub branch: String,
    /// 是否启用
    pub enabled: bool,
    /// 技能所在的子目录路径 (可选, 如 "skills", "my-skills/subdir")
    #[serde(rename = "skillsPath")]
    pub skills_path: Option<String>,
}

/// 技能安装状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillState {
    /// 是否已安装
    pub installed: bool,
    /// 安装时间
    #[serde(rename = "installedAt")]
    pub installed_at: DateTime<Utc>,
}

/// 仓库技能缓存
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRepoCache {
    /// 缓存的技能列表
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<Skill>,
    /// 缓存时间
    #[serde(rename = "fetchedAt", alias = "cachedAt")]
    pub fetched_at: DateTime<Utc>,
    /// ETag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
    /// Last-Modified
    #[serde(rename = "lastModified", skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
}

/// 缓存存储结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillCacheStore {
    #[serde(default)]
    pub repos: HashMap<String, SkillRepoCache>,
}

/// 持久化存储结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStore {
    /// directory -> 安装状态
    pub skills: HashMap<String, SkillState>,
    /// 仓库列表
    pub repos: Vec<SkillRepo>,
    /// 仓库缓存
    #[serde(default, rename = "repoCache", skip_serializing_if = "HashMap::is_empty")]
    pub repo_cache: HashMap<String, SkillRepoCache>,
}

#[derive(Debug, Clone)]
pub struct SkillListResult {
    pub skills: Vec<Skill>,
    pub warnings: Vec<String>,
    pub cache_hit: bool,
    pub refreshing: bool,
}

impl Default for SkillStore {
    fn default() -> Self {
        SkillStore {
            skills: HashMap::new(),
            repos: vec![
                SkillRepo {
                    owner: "ComposioHQ".to_string(),
                    name: "awesome-claude-skills".to_string(),
                    branch: "master".to_string(),
                    enabled: true,
                    skills_path: None, // 扫描根目录
                },
                SkillRepo {
                    owner: "anthropics".to_string(),
                    name: "skills".to_string(),
                    branch: "main".to_string(),
                    enabled: true,
                    skills_path: None, // 扫描根目录
                },
                SkillRepo {
                    owner: "cexll".to_string(),
                    name: "myclaude".to_string(),
                    branch: "master".to_string(),
                    enabled: true,
                    skills_path: Some("skills".to_string()), // 扫描 skills 子目录
                },
            ],
            repo_cache: HashMap::new(),
        }
    }
}

/// 技能元数据 (从 SKILL.md 解析)
#[derive(Debug, Clone, Deserialize)]
pub struct SkillMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct WorkflowMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct SkillService {
    http_client: Client,
    install_dir: PathBuf,
}

#[derive(Debug, Clone)]
struct RepoCacheHeaders {
    etag: Option<String>,
    last_modified: Option<String>,
}

struct DownloadedRepo {
    temp_dir: tempfile::TempDir,
    etag: Option<String>,
    last_modified: Option<String>,
}

enum DownloadOutcome {
    Downloaded {
        etag: Option<String>,
        last_modified: Option<String>,
    },
    NotModified,
}

enum RepoDownloadResult {
    Downloaded(DownloadedRepo),
    NotModified,
}

enum RepoFetchOutcome {
    Updated {
        skills: Vec<Skill>,
        etag: Option<String>,
        last_modified: Option<String>,
    },
    NotModified,
}

impl SkillService {
    pub fn new() -> Result<Self> {
        let install_dir = Self::get_install_dir()?;

        // 确保目录存在
        fs::create_dir_all(&install_dir)?;

        let http_client = Client::builder()
            .user_agent("cc-switch")
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(120))
            .build()?;

        Ok(Self {
            http_client,
            install_dir,
        })
    }

    fn get_install_dir() -> Result<PathBuf> {
        let home = get_home_dir().context(format_skill_error(
            "GET_HOME_DIR_FAILED",
            &[],
            Some("checkPermission"),
        ))?;
        Ok(home.join(".claude").join("skills"))
    }
}

// 核心方法实现
impl SkillService {
    fn normalize_skills_path(skills_path: &str) -> Result<Option<String>> {
        let trimmed = skills_path.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        let trimmed = trimmed.trim_matches(|c| c == '/' || c == '\\');
        if trimmed.is_empty() {
            return Ok(None);
        }

        let normalized = trimmed.replace('\\', "/");
        let normalized_path = Path::new(&normalized);
        let has_traversal = normalized_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        });

        if has_traversal {
            return Err(anyhow!(format_skill_error(
                "SKILL_PATH_INVALID",
                &[("path", skills_path)],
                Some("checkRepoUrl"),
            )));
        }

        Ok(Some(normalized))
    }

    pub(crate) fn validate_skill_directory(directory: &str) -> Result<()> {
        let trimmed = directory.trim();
        if trimmed.is_empty() {
            return Err(anyhow!(format_skill_error(
                "SKILL_DIR_INVALID",
                &[("directory", directory)],
                Some("checkDirectory"),
            )));
        }

        let path = Path::new(trimmed);
        let mut has_component = false;
        let mut has_invalid_component = false;

        for component in path.components() {
            match component {
                Component::Normal(_) => has_component = true,
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    has_invalid_component = true;
                }
            }
        }

        let has_traversal = trimmed.split(['/', '\\']).any(|segment| segment == "..");

        if !has_component || has_invalid_component || path.is_absolute() || has_traversal {
            return Err(anyhow!(format_skill_error(
                "SKILL_DIR_INVALID",
                &[("directory", directory)],
                Some("checkDirectory"),
            )));
        }

        Ok(())
    }

    fn relative_path_components(root: &Path, current_dir: &Path) -> Option<Vec<String>> {
        let relative = current_dir.strip_prefix(root).ok()?;
        let components: Vec<String> = relative
            .components()
            .filter_map(|component| match component {
                Component::Normal(os) => Some(os.to_string_lossy().to_string()),
                _ => None,
            })
            .collect();
        if components.is_empty() {
            None
        } else {
            Some(components)
        }
    }

    fn build_path_info(components: &[String]) -> (String, Option<String>, usize, String) {
        let directory = components.join("/");
        let depth = components.len().saturating_sub(1);
        let parent_path = if depth > 0 {
            Some(components[..depth].join("/"))
        } else {
            None
        };
        let leaf_name = components.last().cloned().unwrap_or_default();
        (directory, parent_path, depth, leaf_name)
    }

    fn cache_key(repo: &SkillRepo) -> String {
        let raw_path = repo.skills_path.as_deref().unwrap_or("");
        let normalized_path = raw_path
            .trim()
            .trim_matches(|c| c == '/' || c == '\\')
            .replace('\\', "/");
        if normalized_path.is_empty() {
            format!("{}/{}/{}", repo.owner, repo.name, repo.branch)
        } else {
            format!(
                "{}/{}/{}:{}",
                repo.owner, repo.name, repo.branch, normalized_path
            )
        }
    }

    fn cache_ttl() -> Duration {
        let default_ttl = Duration::from_secs(DEFAULT_SKILL_CACHE_TTL_SECS);
        let raw = match env::var("CC_SWITCH_SKILLS_CACHE_TTL_SECS") {
            Ok(value) => value,
            Err(_) => return default_ttl,
        };

        match raw.trim().parse::<u64>() {
            Ok(value) => Duration::from_secs(value),
            Err(_) => {
                log::warn!(
                    "环境变量 CC_SWITCH_SKILLS_CACHE_TTL_SECS 无法解析: {}，使用默认值 {} 秒",
                    raw,
                    DEFAULT_SKILL_CACHE_TTL_SECS
                );
                default_ttl
            }
        }
    }

    fn is_cache_fresh(fetched_at: DateTime<Utc>) -> bool {
        let ttl_secs = Self::cache_ttl().as_secs() as i64;
        if ttl_secs == 0 {
            return false;
        }
        let elapsed = Utc::now().signed_duration_since(fetched_at);
        elapsed <= chrono::Duration::seconds(ttl_secs)
    }

    fn load_repo_cache(&self) -> SkillCacheStore {
        let cache_path = match get_app_config_dir() {
            Ok(dir) => dir.join("skills-cache.json"),
            Err(e) => {
                log::warn!("获取技能缓存目录失败: {}", e);
                return SkillCacheStore::default();
            }
        };

        let content = match fs::read_to_string(&cache_path) {
            Ok(content) => content,
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    log::warn!("读取技能缓存文件 {} 失败: {}", cache_path.display(), e);
                }
                return SkillCacheStore::default();
            }
        };

        match serde_json::from_str::<SkillCacheStore>(&content) {
            Ok(store) => store,
            Err(e) => {
                log::warn!("解析技能缓存文件 {} 失败: {}", cache_path.display(), e);
                SkillCacheStore::default()
            }
        }
    }

    fn save_repo_cache(&self, cache_store: &SkillCacheStore) {
        let cache_path = match get_app_config_dir() {
            Ok(dir) => dir.join("skills-cache.json"),
            Err(e) => {
                log::warn!("获取技能缓存目录失败: {}", e);
                return;
            }
        };

        if let Err(e) = write_json_file(&cache_path, cache_store) {
            log::warn!("写入技能缓存文件 {} 失败: {}", cache_path.display(), e);
        }
    }

    /// 列出所有技能
    pub async fn list_skills(
        &self,
        repos: Vec<SkillRepo>,
        repo_cache: &mut HashMap<String, SkillRepoCache>,
    ) -> Result<SkillListResult> {
        let mut skills = Vec::new();
        let mut warnings = Vec::new();
        let mut cache_store = self.load_repo_cache();
        let mut cache_updated = false;

        if !repo_cache.is_empty() {
            for (key, entry) in repo_cache.iter() {
                let should_replace = match cache_store.repos.get(key) {
                    None => true,
                    Some(existing) => entry.fetched_at > existing.fetched_at,
                };
                if should_replace {
                    cache_store.repos.insert(key.clone(), entry.clone());
                    cache_updated = true;
                }
            }
        }

        // 仅使用启用的仓库，并行获取技能列表，避免单个无效仓库拖慢整体刷新
        let enabled_repos: Vec<SkillRepo> = repos.into_iter().filter(|repo| repo.enabled).collect();
        let mut fetch_tasks = Vec::new();

        for repo in enabled_repos.iter().cloned() {
            let cache_key = Self::cache_key(&repo);
            let cached_entry = cache_store.repos.get(&cache_key).cloned();

            if let Some(entry) = cached_entry.as_ref() {
                if Self::is_cache_fresh(entry.fetched_at) {
                    skills.extend(entry.skills.clone());
                    continue;
                }
            }

            fetch_tasks.push(async move {
                let result = self
                    .fetch_repo_skills_with_cache(&repo, cached_entry.as_ref())
                    .await;
                (repo, cache_key, cached_entry, result)
            });
        }

        let refreshing = !fetch_tasks.is_empty();
        let cache_hit = !refreshing;

        let results: Vec<(SkillRepo, String, Option<SkillRepoCache>, Result<RepoFetchOutcome>)> =
            futures::future::join_all(fetch_tasks).await;

        for (repo, cache_key, cached_entry, result) in results {
            match result {
                Ok(outcome) => match outcome {
                    RepoFetchOutcome::Updated {
                        skills: repo_skills,
                        etag,
                        last_modified,
                    } => {
                        let fetched_at = Utc::now();
                        skills.extend(repo_skills.clone());
                        cache_store.repos.insert(
                            cache_key,
                            SkillRepoCache {
                                fetched_at,
                                skills: repo_skills,
                                etag,
                                last_modified,
                            },
                        );
                        cache_updated = true;
                    }
                    RepoFetchOutcome::NotModified => {
                        if let Some(mut entry) = cached_entry {
                            entry.fetched_at = Utc::now();
                            skills.extend(entry.skills.clone());
                            cache_store.repos.insert(cache_key, entry);
                            cache_updated = true;
                        } else {
                            let warning =
                                format!("仓库 {}/{} 返回 304，但本地没有缓存", repo.owner, repo.name);
                            log::warn!("{warning}");
                            warnings.push(warning);
                        }
                    }
                },
                Err(e) => {
                    if let Some(entry) = cached_entry {
                        let warning = format!(
                            "获取仓库 {}/{} 失败: {}，使用缓存",
                            repo.owner, repo.name, e
                        );
                        log::warn!("{warning}");
                        warnings.push(warning);
                        skills.extend(entry.skills);
                    } else {
                        let warning = format!("获取仓库 {}/{} 失败: {}", repo.owner, repo.name, e);
                        log::warn!("{warning}");
                        warnings.push(warning);
                    }
                }
            }
        }

        if cache_updated {
            self.save_repo_cache(&cache_store);
        }

        repo_cache.clear();
        repo_cache.extend(cache_store.repos.clone());

        // 合并本地技能
        self.merge_local_skills(&mut skills)?;

        // 去重并排序
        Self::deduplicate_skills(&mut skills);
        skills.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(SkillListResult {
            skills,
            warnings,
            cache_hit,
            refreshing,
        })
    }

    /// 从仓库获取技能列表
    async fn fetch_repo_skills_with_cache(
        &self,
        repo: &SkillRepo,
        cache_entry: Option<&SkillRepoCache>,
    ) -> Result<RepoFetchOutcome> {
        let cache_headers = cache_entry.map(|entry| RepoCacheHeaders {
            etag: entry.etag.clone(),
            last_modified: entry.last_modified.clone(),
        });

        // 为单个仓库加载增加整体超时，避免无效链接长时间阻塞
        let download_result = timeout(
            Duration::from_secs(180),
            self.download_repo(repo, cache_headers.as_ref()),
        )
        .await
        .map_err(|_| {
            anyhow!(format_skill_error(
                "DOWNLOAD_TIMEOUT",
                &[
                    ("owner", &repo.owner),
                    ("name", &repo.name),
                    ("timeout", "180")
                ],
                Some("checkNetwork"),
            ))
        })??;

        let download = match download_result {
            RepoDownloadResult::NotModified => {
                return Ok(RepoFetchOutcome::NotModified);
            }
            RepoDownloadResult::Downloaded(download) => download,
        };

        let temp_path = download.temp_dir.path().to_path_buf();
        let mut skills = Vec::new();

        let normalized_skills_path = match repo.skills_path.as_ref() {
            Some(skills_path) => match Self::normalize_skills_path(skills_path) {
                Ok(path) => path,
                Err(err) => {
                    return Err(err);
                }
            },
            None => None,
        };

        // 确定要扫描的目录路径
        let scan_dir = if let Some(ref normalized_skills_path) = normalized_skills_path {
            // 如果指定了 skillsPath，则扫描该子目录
            let subdir = temp_path.join(normalized_skills_path);
            if !subdir.exists() {
                log::warn!(
                    "仓库 {}/{} 中指定的技能路径 '{}' 不存在",
                    repo.owner,
                    repo.name,
                    repo.skills_path.as_deref().unwrap_or_default()
                );
                return Ok(RepoFetchOutcome::Updated {
                    skills,
                    etag: download.etag,
                    last_modified: download.last_modified,
                });
            }
            subdir
        } else {
            // 否则扫描仓库根目录
            temp_path.clone()
        };

        self.scan_skills_recursive(
            &scan_dir,
            &scan_dir,
            repo,
            normalized_skills_path.as_deref(),
            &mut skills,
        )?;

        Ok(RepoFetchOutcome::Updated {
            skills,
            etag: download.etag,
            last_modified: download.last_modified,
        })
    }

    /// 递归扫描目录树，查找所有 SKILL.md
    fn scan_skills_recursive(
        &self,
        scan_root: &Path,
        current_dir: &Path,
        repo: &SkillRepo,
        normalized_skills_path: Option<&str>,
        skills: &mut Vec<Skill>,
    ) -> Result<()> {
        let root_metadata = match fs::symlink_metadata(current_dir) {
            Ok(metadata) => metadata,
            Err(e) => {
                log::warn!("读取扫描目录 {} 元数据失败: {}", current_dir.display(), e);
                return Ok(());
            }
        };

        if root_metadata.file_type().is_symlink() {
            log::warn!("跳过符号链接目录 {}，避免路径穿越", current_dir.display());
            return Ok(());
        }

        if !root_metadata.is_dir() {
            return Ok(());
        }

        self.scan_skills_recursive_inner(
            scan_root,
            current_dir,
            repo,
            normalized_skills_path,
            skills,
            0,
        )
    }

    fn scan_skills_recursive_inner(
        &self,
        scan_root: &Path,
        current_dir: &Path,
        repo: &SkillRepo,
        normalized_skills_path: Option<&str>,
        skills: &mut Vec<Skill>,
        depth: usize,
    ) -> Result<()> {
        if let Some(components) = Self::relative_path_components(scan_root, current_dir) {
            let skill_md = current_dir.join("SKILL.md");
            match fs::symlink_metadata(&skill_md) {
                Ok(metadata) => {
                    if metadata.file_type().is_symlink() {
                        log::warn!("跳过符号链接文件 {}，避免路径穿越", skill_md.display());
                    } else if metadata.is_file() {
                        match self.parse_skill_metadata(&skill_md) {
                            Ok(meta) => {
                                let (directory, parent_path, depth, leaf_name) =
                                    Self::build_path_info(&components);
                                let readme_path = if let Some(skills_path) = normalized_skills_path
                                {
                                    format!("{}/{}", skills_path, directory)
                                } else {
                                    directory.clone()
                                };
                                let commands = match self.scan_workflow_commands(current_dir) {
                                    Ok(commands) => commands,
                                    Err(e) => {
                                        log::warn!(
                                            "扫描 {} workflows 失败: {}",
                                            current_dir.display(),
                                            e
                                        );
                                        Vec::new()
                                    }
                                };

                                skills.push(Skill {
                                    key: format!("{}/{}:{}", repo.owner, repo.name, directory),
                                    name: meta.name.unwrap_or_else(|| leaf_name.clone()),
                                    description: meta.description.unwrap_or_default(),
                                    directory,
                                    parent_path,
                                    depth,
                                    readme_url: Some(format!(
                                        "https://github.com/{}/{}/tree/{}/{}",
                                        repo.owner, repo.name, repo.branch, readme_path
                                    )),
                                    installed: false,
                                    repo_owner: Some(repo.owner.clone()),
                                    repo_name: Some(repo.name.clone()),
                                    repo_branch: Some(repo.branch.clone()),
                                    skills_path: repo.skills_path.clone(),
                                    commands,
                                });
                            }
                            Err(e) => log::warn!("解析 {} 元数据失败: {}", skill_md.display(), e),
                        }
                    }
                }
                Err(e) => {
                    if e.kind() != ErrorKind::NotFound {
                        log::warn!("读取 {} 元数据失败: {}", skill_md.display(), e);
                    }
                }
            }
        }

        if depth >= MAX_SKILL_SCAN_DEPTH {
            log::warn!(
                "扫描目录 {} 已达到最大深度 {}, 停止向下递归",
                current_dir.display(),
                MAX_SKILL_SCAN_DEPTH
            );
            return Ok(());
        }

        let entries = match fs::read_dir(current_dir) {
            Ok(entries) => entries,
            Err(e) => {
                log::warn!("读取目录 {} 失败: {}", current_dir.display(), e);
                return Ok(());
            }
        };

        for entry_result in entries {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    log::warn!("读取目录项 {} 失败: {}", current_dir.display(), e);
                    continue;
                }
            };
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(e) => {
                    log::warn!("读取 {} 类型失败: {}", entry.path().display(), e);
                    continue;
                }
            };
            if !file_type.is_dir() || file_type.is_symlink() {
                continue;
            }
            self.scan_skills_recursive_inner(
                scan_root,
                &entry.path(),
                repo,
                normalized_skills_path,
                skills,
                depth + 1,
            )?;
        }

        Ok(())
    }

    /// 解析技能元数据
    fn parse_skill_metadata(&self, path: &Path) -> Result<SkillMetadata> {
        let content = fs::read_to_string(path)?;

        // 移除 BOM
        let content = content.trim_start_matches('\u{feff}');

        // 提取 YAML front matter
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return Ok(SkillMetadata {
                name: None,
                description: None,
            });
        }

        let front_matter = parts[1].trim();
        let meta: SkillMetadata = serde_yaml::from_str(front_matter).unwrap_or(SkillMetadata {
            name: None,
            description: None,
        });

        Ok(meta)
    }

    fn scan_workflow_commands(&self, skill_dir: &Path) -> Result<Vec<SkillCommand>> {
        let workflows_dir = skill_dir.join("workflows");
        if !workflows_dir.is_dir() {
            return Ok(Vec::new());
        }

        let mut commands = Vec::new();
        for entry in fs::read_dir(&workflows_dir)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if !file_type.is_file() {
                continue;
            }

            let path = entry.path();
            let ext = path.extension().and_then(|ext| ext.to_str());
            if !ext
                .map(|ext| ext.eq_ignore_ascii_case("md"))
                .unwrap_or(false)
            {
                continue;
            }

            let relative_path = path
                .strip_prefix(skill_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");

            match self.parse_workflow_command(&path, relative_path) {
                Ok(command) => commands.push(command),
                Err(e) => log::warn!("解析 {} workflow 命令失败: {}", path.display(), e),
            }
        }

        commands.sort_by(|a, b| {
            let name_cmp = a.name.to_lowercase().cmp(&b.name.to_lowercase());
            if name_cmp == std::cmp::Ordering::Equal {
                a.file_path.cmp(&b.file_path)
            } else {
                name_cmp
            }
        });

        Ok(commands)
    }

    fn parse_workflow_command(&self, path: &Path, file_path: String) -> Result<SkillCommand> {
        let content = fs::read_to_string(path)?;
        let content = content.trim_start_matches('\u{feff}');

        let (front_matter, body) = Self::split_front_matter(content);
        let mut name = None;
        let mut description = None;

        if let Some(front_matter) = front_matter {
            let meta: WorkflowMetadata =
                serde_yaml::from_str(front_matter).unwrap_or(WorkflowMetadata {
                    name: None,
                    description: None,
                });
            name = meta.name;
            description = meta.description;
        }

        let body = body.trim_start_matches(['\n', '\r']);
        if name.is_none() || description.is_none() {
            let (heading, summary) = Self::extract_markdown_heading_and_summary(body);
            if name.is_none() {
                name = heading;
            }
            if description.is_none() {
                description = summary;
            }
        }

        let fallback_name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("command");
        let name = name
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| fallback_name.to_string());
        let description = description.unwrap_or_default();

        Ok(SkillCommand {
            name,
            description,
            file_path,
        })
    }

    fn split_front_matter(content: &str) -> (Option<&str>, &str) {
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            (None, content)
        } else {
            (Some(parts[1].trim()), parts[2])
        }
    }

    fn extract_markdown_heading_and_summary(body: &str) -> (Option<String>, Option<String>) {
        let mut heading = None;
        let mut summary = None;

        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if heading.is_none() {
                if let Some(stripped) = trimmed.strip_prefix('#') {
                    let title = stripped.trim_start_matches('#').trim();
                    if !title.is_empty() {
                        heading = Some(title.to_string());
                        continue;
                    }
                }
            }

            if heading.is_some() && summary.is_none() && !trimmed.starts_with('#') {
                summary = Some(trimmed.to_string());
                break;
            }

            if heading.is_none() && summary.is_none() && !trimmed.starts_with('#') {
                summary = Some(trimmed.to_string());
                break;
            }
        }

        (heading, summary)
    }

    /// 合并本地技能
    fn merge_local_skills(&self, skills: &mut Vec<Skill>) -> Result<()> {
        if !self.install_dir.exists() {
            return Ok(());
        }

        for skill in skills.iter_mut() {
            let skill_path = self.install_dir.join(&skill.directory);
            if skill_path.join("SKILL.md").is_file() {
                skill.installed = true;
            }
        }

        self.merge_local_skills_recursive(&self.install_dir, &self.install_dir, skills)?;

        Ok(())
    }

    fn merge_local_skills_recursive(
        &self,
        scan_root: &Path,
        current_dir: &Path,
        skills: &mut Vec<Skill>,
    ) -> Result<()> {
        self.merge_local_skills_recursive_inner(scan_root, current_dir, skills, 0)
    }

    fn merge_local_skills_recursive_inner(
        &self,
        scan_root: &Path,
        current_dir: &Path,
        skills: &mut Vec<Skill>,
        depth: usize,
    ) -> Result<()> {
        if let Some(components) = Self::relative_path_components(scan_root, current_dir) {
            let skill_md = current_dir.join("SKILL.md");
            match fs::symlink_metadata(&skill_md) {
                Ok(metadata) => {
                    if metadata.file_type().is_symlink() {
                        log::warn!("跳过符号链接文件 {}，避免路径穿越", skill_md.display());
                    } else if metadata.is_file() {
                        let (directory, parent_path, depth, leaf_name) =
                            Self::build_path_info(&components);
                        let exists = skills
                            .iter()
                            .any(|skill| skill.directory.eq_ignore_ascii_case(&directory));
                        if !exists {
                            if let Ok(meta) = self.parse_skill_metadata(&skill_md) {
                                let commands = match self.scan_workflow_commands(current_dir) {
                                    Ok(commands) => commands,
                                    Err(e) => {
                                        log::warn!(
                                            "扫描 {} workflows 失败: {}",
                                            current_dir.display(),
                                            e
                                        );
                                        Vec::new()
                                    }
                                };
                                skills.push(Skill {
                                    key: format!("local:{directory}"),
                                    name: meta.name.unwrap_or_else(|| leaf_name.clone()),
                                    description: meta.description.unwrap_or_default(),
                                    directory,
                                    parent_path,
                                    depth,
                                    readme_url: None,
                                    installed: true,
                                    repo_owner: None,
                                    repo_name: None,
                                    repo_branch: None,
                                    skills_path: None,
                                    commands,
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    if e.kind() != ErrorKind::NotFound {
                        log::warn!("读取 {} 元数据失败: {}", skill_md.display(), e);
                    }
                }
            }
        }

        if depth >= MAX_SKILL_SCAN_DEPTH {
            log::warn!(
                "扫描目录 {} 已达到最大深度 {}, 停止向下递归",
                current_dir.display(),
                MAX_SKILL_SCAN_DEPTH
            );
            return Ok(());
        }

        let entries = match fs::read_dir(current_dir) {
            Ok(entries) => entries,
            Err(e) => {
                log::warn!("读取目录 {} 失败: {}", current_dir.display(), e);
                return Ok(());
            }
        };

        for entry_result in entries {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    log::warn!("读取目录项 {} 失败: {}", current_dir.display(), e);
                    continue;
                }
            };
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(e) => {
                    log::warn!("读取 {} 类型失败: {}", entry.path().display(), e);
                    continue;
                }
            };
            if !file_type.is_dir() || file_type.is_symlink() {
                continue;
            }
            self.merge_local_skills_recursive_inner(scan_root, &entry.path(), skills, depth + 1)?;
        }

        Ok(())
    }

    /// 去重技能列表
    fn deduplicate_skills(skills: &mut Vec<Skill>) {
        let mut seen = HashSet::new();
        skills.retain(|skill| {
            // key 已包含 owner/name:directory 或 local:directory，使用它避免不同仓库同名目录被误去重
            let key = skill.key.to_lowercase();
            seen.insert(key)
        });
    }

    /// 下载仓库
    async fn download_repo(
        &self,
        repo: &SkillRepo,
        cache_headers: Option<&RepoCacheHeaders>,
    ) -> Result<RepoDownloadResult> {
        // 尝试多个分支
        let branches = if repo.branch.is_empty() {
            vec!["main", "master"]
        } else {
            vec![repo.branch.as_str(), "main", "master"]
        };

        let mut last_error = None;
        for branch in branches {
            let temp_dir = tempfile::tempdir()?;
            let url = format!(
                "https://github.com/{}/{}/archive/refs/heads/{}.zip",
                repo.owner, repo.name, branch
            );

            match self
                .download_and_extract(&url, temp_dir.path(), cache_headers)
                .await
            {
                Ok(DownloadOutcome::Downloaded { etag, last_modified }) => {
                    return Ok(RepoDownloadResult::Downloaded(DownloadedRepo {
                        temp_dir,
                        etag,
                        last_modified,
                    }));
                }
                Ok(DownloadOutcome::NotModified) => {
                    return Ok(RepoDownloadResult::NotModified);
                }
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            };
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("所有分支下载失败")))
    }

    /// 下载并解压 ZIP
    async fn download_and_extract(
        &self,
        url: &str,
        dest: &Path,
        cache_headers: Option<&RepoCacheHeaders>,
    ) -> Result<DownloadOutcome> {
        // 下载 ZIP
        let mut request = self.http_client.get(url);
        if let Some(headers) = cache_headers {
            if let Some(etag) = headers.etag.as_deref() {
                request = request.header(header::IF_NONE_MATCH, etag);
            }
            if let Some(last_modified) = headers.last_modified.as_deref() {
                request = request.header(header::IF_MODIFIED_SINCE, last_modified);
            }
        }

        let response = request.send().await?;
        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(DownloadOutcome::NotModified);
        }
        if !response.status().is_success() {
            let status = response.status().as_u16().to_string();
            return Err(anyhow::anyhow!(format_skill_error(
                "DOWNLOAD_FAILED",
                &[("status", &status)],
                match status.as_str() {
                    "403" => Some("http403"),
                    "404" => Some("http404"),
                    "429" => Some("http429"),
                    _ => Some("checkNetwork"),
                },
            )));
        }

        let etag = response
            .headers()
            .get(header::ETAG)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());
        let last_modified = response
            .headers()
            .get(header::LAST_MODIFIED)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());

        let bytes = response.bytes().await?.to_vec();
        let dest = dest.to_path_buf();
        tokio::task::spawn_blocking(move || Self::extract_zip_to_dir(bytes, dest)).await??;

        Ok(DownloadOutcome::Downloaded { etag, last_modified })
    }

    fn extract_zip_to_dir(bytes: Vec<u8>, dest: PathBuf) -> Result<()> {
        // 解压
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor)?;

        // 获取根目录名称 (GitHub 的 zip 会有一个根目录)
        let root_name = if !archive.is_empty() {
            let first_file = archive.by_index(0)?;
            let name = first_file.name();
            name.split('/').next().unwrap_or("").to_string()
        } else {
            return Err(anyhow::anyhow!(format_skill_error(
                "EMPTY_ARCHIVE",
                &[],
                Some("checkRepoUrl"),
            )));
        };

        // 解压所有文件
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_path = file.name();

            // 跳过根目录，直接提取内容
            let relative_path =
                if let Some(stripped) = file_path.strip_prefix(&format!("{root_name}/")) {
                    stripped
                } else {
                    continue;
                };

            if relative_path.is_empty() {
                continue;
            }

            let relative_path_obj = Path::new(relative_path);
            let has_traversal = relative_path_obj.components().any(|c| {
                matches!(
                    c,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            }) || relative_path
                .split(['/', '\\'])
                .any(|segment| segment == "..");

            if relative_path_obj.is_absolute() || has_traversal {
                return Err(anyhow!(format_skill_error(
                    "INVALID_ARCHIVE_PATH",
                    &[("path", file_path)],
                    Some("checkRepoUrl"),
                )));
            }

            let outpath = dest.join(relative_path_obj);

            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }

    /// 安装技能（仅负责下载和文件操作，状态更新由上层负责）
    pub async fn install_skill(&self, directory: String, repo: SkillRepo) -> Result<()> {
        Self::validate_skill_directory(&directory)?;
        let dest = self.install_dir.join(&directory);

        // 若目标目录已存在，则视为已安装，避免重复下载
        if dest.exists() {
            return Ok(());
        }

        // 下载仓库时增加总超时，防止无效链接导致长时间卡住安装过程
        let temp_dir = timeout(
            std::time::Duration::from_secs(180),
            self.download_repo(&repo, None),
        )
        .await
        .map_err(|_| {
            anyhow!(format_skill_error(
                "DOWNLOAD_TIMEOUT",
                &[
                    ("owner", &repo.owner),
                    ("name", &repo.name),
                    ("timeout", "180")
                ],
                Some("checkNetwork"),
            ))
        })??;
        let temp_dir = match temp_dir {
            RepoDownloadResult::Downloaded(download) => download.temp_dir,
            RepoDownloadResult::NotModified => {
                return Err(anyhow::anyhow!(format_skill_error(
                    "DOWNLOAD_FAILED",
                    &[("status", "304")],
                    Some("checkNetwork"),
                )));
            }
        };
        let temp_path = temp_dir.path().to_path_buf();

        // 根据 skills_path 确定源目录路径
        let source = if let Some(ref skills_path) = repo.skills_path {
            // 如果指定了 skills_path，源路径为: temp_dir/skills_path/directory
            let normalized_skills_path = match Self::normalize_skills_path(skills_path) {
                Ok(path) => path,
                Err(err) => {
                    return Err(err);
                }
            };
            match normalized_skills_path {
                Some(path) => temp_path.join(path).join(&directory),
                None => temp_path.join(&directory),
            }
        } else {
            // 否则源路径为: temp_dir/directory
            temp_path.join(&directory)
        };

        if !source.exists() {
            return Err(anyhow::anyhow!(format_skill_error(
                "SKILL_DIR_NOT_FOUND",
                &[("path", &source.display().to_string())],
                Some("checkRepoUrl"),
            )));
        }

        // 删除旧版本
        if dest.exists() {
            fs::remove_dir_all(&dest)?;
        }

        // 递归复制
        Self::copy_dir_recursive(&source, &dest)?;

        Ok(())
    }

    /// 递归复制目录
    fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
        fs::create_dir_all(dest)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let dest_path = dest.join(entry.file_name());

            if path.is_dir() {
                Self::copy_dir_recursive(&path, &dest_path)?;
            } else {
                fs::copy(&path, &dest_path)?;
            }
        }

        Ok(())
    }

    /// 卸载技能（仅负责文件操作，状态更新由上层负责）
    pub fn uninstall_skill(&self, directory: String) -> Result<()> {
        Self::validate_skill_directory(&directory)?;
        let dest = self.install_dir.join(&directory);

        if dest.exists() {
            fs::remove_dir_all(&dest)?;
        }

        Ok(())
    }

    /// 列出仓库
    pub fn list_repos(&self, store: &SkillStore) -> Vec<SkillRepo> {
        store.repos.clone()
    }

    /// 添加仓库
    pub fn add_repo(&self, store: &mut SkillStore, repo: SkillRepo) -> Result<()> {
        // 检查重复
        if let Some(pos) = store
            .repos
            .iter()
            .position(|r| r.owner == repo.owner && r.name == repo.name)
        {
            store.repos[pos] = repo;
        } else {
            store.repos.push(repo);
        }

        Ok(())
    }

    /// 删除仓库
    pub fn remove_repo(&self, store: &mut SkillStore, owner: String, name: String) -> Result<()> {
        store
            .repos
            .retain(|r| !(r.owner == owner && r.name == name));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_service_with_install_dir(dir: PathBuf) -> SkillService {
        SkillService {
            http_client: Client::builder()
                .user_agent("cc-switch-test")
                .build()
                .expect("client build should succeed"),
            install_dir: dir,
        }
    }

    fn make_skill(key: &str, directory: &str) -> Skill {
        Skill {
            key: key.to_string(),
            name: directory.to_string(),
            description: String::new(),
            directory: directory.to_string(),
            parent_path: None,
            depth: 0,
            readme_url: None,
            installed: false,
            repo_owner: None,
            repo_name: None,
            repo_branch: None,
            skills_path: None,
            commands: Vec::new(),
        }
    }

    #[test]
    fn test_normalize_skills_path() {
        let normalized = SkillService::normalize_skills_path("/skills\\nested//")
            .expect("normalize should succeed");
        assert_eq!(normalized, Some("skills/nested".to_string()));
    }

    #[test]
    fn test_normalize_skills_path_rejects_traversal() {
        let normalized = SkillService::normalize_skills_path("../skills");
        assert!(normalized.is_err());
    }

    #[test]
    fn test_validate_skill_directory_accepts_relative() {
        assert!(SkillService::validate_skill_directory("skills/subdir").is_ok());
        assert!(SkillService::validate_skill_directory("./skills/subdir").is_ok());
    }

    #[test]
    fn test_validate_skill_directory_rejects_traversal_or_absolute() {
        assert!(SkillService::validate_skill_directory("../skills").is_err());
        assert!(SkillService::validate_skill_directory("skills/../escape").is_err());
        assert!(SkillService::validate_skill_directory("..\\escape").is_err());
        assert!(SkillService::validate_skill_directory("/absolute").is_err());
        assert!(SkillService::validate_skill_directory("").is_err());
    }

    #[test]
    fn test_parse_skill_metadata() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let skill_md = temp_dir.path().join("SKILL.md");
        let content = r#"---
name: Demo Skill
description: Useful skill
---
# body
"#;
        fs::write(&skill_md, content).expect("should write skill metadata");
        let service = build_service_with_install_dir(temp_dir.path().to_path_buf());

        let metadata = service
            .parse_skill_metadata(&skill_md)
            .expect("metadata should parse");

        assert_eq!(metadata.name.as_deref(), Some("Demo Skill"));
        assert_eq!(metadata.description.as_deref(), Some("Useful skill"));
    }

    #[test]
    fn test_deduplicate_skills() {
        let mut skills = vec![
            make_skill("owner/name:skill", "SkillOne"),
            make_skill("Owner/Name:Skill", "SkillTwo"),
            make_skill("local:unique", "Unique"),
        ];

        SkillService::deduplicate_skills(&mut skills);

        assert_eq!(skills.len(), 2);
        assert!(skills.iter().any(|s| s.key == "owner/name:skill"));
        assert!(skills.iter().any(|s| s.key == "local:unique"));
    }
}
