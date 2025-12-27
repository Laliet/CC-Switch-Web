use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;
use tokio::time::timeout;

use crate::config::get_home_dir;
use crate::error::format_skill_error;

const MAX_SKILL_SCAN_DEPTH: usize = 32;

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

/// 持久化存储结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStore {
    /// directory -> 安装状态
    pub skills: HashMap<String, SkillState>,
    /// 仓库列表
    pub repos: Vec<SkillRepo>,
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

        let has_traversal = trimmed
            .split(['/', '\\'])
            .any(|segment| segment == "..");

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

    /// 列出所有技能
    pub async fn list_skills(&self, repos: Vec<SkillRepo>) -> Result<(Vec<Skill>, Vec<String>)> {
        let mut skills = Vec::new();
        let mut warnings = Vec::new();

        // 仅使用启用的仓库，并行获取技能列表，避免单个无效仓库拖慢整体刷新
        let enabled_repos: Vec<SkillRepo> = repos.into_iter().filter(|repo| repo.enabled).collect();

        let fetch_tasks = enabled_repos
            .iter()
            .map(|repo| self.fetch_repo_skills(repo));

        let results: Vec<Result<Vec<Skill>>> = futures::future::join_all(fetch_tasks).await;

        for (repo, result) in enabled_repos.into_iter().zip(results.into_iter()) {
            match result {
                Ok(repo_skills) => skills.extend(repo_skills),
                Err(e) => {
                    let warning = format!("获取仓库 {}/{} 失败: {}", repo.owner, repo.name, e);
                    log::warn!("{warning}");
                    warnings.push(warning);
                }
            }
        }

        // 合并本地技能
        self.merge_local_skills(&mut skills)?;

        // 去重并排序
        Self::deduplicate_skills(&mut skills);
        skills.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok((skills, warnings))
    }

    /// 从仓库获取技能列表
    async fn fetch_repo_skills(&self, repo: &SkillRepo) -> Result<Vec<Skill>> {
        // 为单个仓库加载增加整体超时，避免无效链接长时间阻塞
        let temp_dir = timeout(Duration::from_secs(180), self.download_repo(repo))
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
        let temp_path = temp_dir.path().to_path_buf();
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
                return Ok(skills);
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

        Ok(skills)
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
                log::warn!(
                    "读取扫描目录 {} 元数据失败: {}",
                    current_dir.display(),
                    e
                );
                return Ok(());
            }
        };

        if root_metadata.file_type().is_symlink() {
            log::warn!(
                "跳过符号链接目录 {}，避免路径穿越",
                current_dir.display()
            );
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
                        log::warn!(
                            "跳过符号链接文件 {}，避免路径穿越",
                            skill_md.display()
                        );
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
            if !ext.map(|ext| ext.eq_ignore_ascii_case("md")).unwrap_or(false) {
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
                        log::warn!(
                            "跳过符号链接文件 {}，避免路径穿越",
                            skill_md.display()
                        );
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
            self.merge_local_skills_recursive_inner(
                scan_root,
                &entry.path(),
                skills,
                depth + 1,
            )?;
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
    async fn download_repo(&self, repo: &SkillRepo) -> Result<tempfile::TempDir> {
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

            match self.download_and_extract(&url, temp_dir.path()).await {
                Ok(_) => {
                    return Ok(temp_dir);
                }
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("所有分支下载失败")))
    }

    /// 下载并解压 ZIP
    async fn download_and_extract(&self, url: &str, dest: &Path) -> Result<()> {
        // 下载 ZIP
        let response = self.http_client.get(url).send().await?;
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

        let bytes = response.bytes().await?.to_vec();
        let dest = dest.to_path_buf();
        tokio::task::spawn_blocking(move || Self::extract_zip_to_dir(bytes, dest)).await??;

        Ok(())
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
            self.download_repo(&repo),
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
