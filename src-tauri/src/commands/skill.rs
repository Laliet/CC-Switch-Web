use crate::error::format_skill_error;
use crate::services::skill::SkillState;
use crate::services::{Skill, SkillRepo, SkillService};
use crate::store::AppState;
use chrono::Utc;
use serde::Serialize;
use std::sync::Arc;
use tauri::State;

pub struct SkillServiceState(pub Arc<SkillService>);

#[derive(Serialize)]
pub struct SkillsResponse {
    pub skills: Vec<Skill>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[tauri::command]
pub async fn get_skills(
    service: State<'_, SkillServiceState>,
    app_state: State<'_, AppState>,
) -> Result<SkillsResponse, String> {
    let repos = {
        let config = app_state.config.read().map_err(|e| e.to_string())?;
        config.skills.repos.clone()
    };

    let (skills, warnings) = service
        .0
        .list_skills(repos)
        .await
        .map_err(|e| e.to_string())?;

    Ok(SkillsResponse { skills, warnings })
}

#[tauri::command]
pub async fn install_skill(
    directory: String,
    service: State<'_, SkillServiceState>,
    app_state: State<'_, AppState>,
) -> Result<bool, String> {
    // 先在不持有写锁的情况下收集仓库与技能信息
    let repos = {
        let config = app_state.config.read().map_err(|e| e.to_string())?;
        config.skills.repos.clone()
    };

    let skills = service
        .0
        .list_skills(repos)
        .await
        .map_err(|e| e.to_string())?
        .0;

    let skill = skills
        .iter()
        .find(|s| s.directory.eq_ignore_ascii_case(&directory))
        .ok_or_else(|| {
            format_skill_error(
                "SKILL_NOT_FOUND",
                &[("directory", &directory)],
                Some("checkRepoUrl"),
            )
        })?;

    if !skill.installed {
        let repo = SkillRepo {
            owner: skill.repo_owner.clone().ok_or_else(|| {
                format_skill_error(
                    "MISSING_REPO_INFO",
                    &[("directory", &directory), ("field", "owner")],
                    None,
                )
            })?,
            name: skill.repo_name.clone().ok_or_else(|| {
                format_skill_error(
                    "MISSING_REPO_INFO",
                    &[("directory", &directory), ("field", "name")],
                    None,
                )
            })?,
            branch: skill
                .repo_branch
                .clone()
                .unwrap_or_else(|| "main".to_string()),
            enabled: true,
            skills_path: skill.skills_path.clone(), // 使用技能记录的 skills_path
        };

        service
            .0
            .install_skill(directory.clone(), repo)
            .await
            .map_err(|e| e.to_string())?;
    }

    {
        let mut config = app_state.config.write().map_err(|e| e.to_string())?;

        config.skills.skills.insert(
            directory.clone(),
            SkillState {
                installed: true,
                installed_at: Utc::now(),
            },
        );
    }

    app_state.save().map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn uninstall_skill(
    directory: String,
    service: State<'_, SkillServiceState>,
    app_state: State<'_, AppState>,
) -> Result<bool, String> {
    service
        .0
        .uninstall_skill(directory.clone())
        .map_err(|e| e.to_string())?;

    {
        let mut config = app_state.config.write().map_err(|e| e.to_string())?;

        config.skills.skills.remove(&directory);
    }

    app_state.save().map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn get_skill_repos(
    _service: State<'_, SkillServiceState>,
    app_state: State<'_, AppState>,
) -> Result<Vec<SkillRepo>, String> {
    let config = app_state.config.read().map_err(|e| e.to_string())?;

    Ok(config.skills.repos.clone())
}

#[tauri::command]
pub fn add_skill_repo(
    repo: SkillRepo,
    service: State<'_, SkillServiceState>,
    app_state: State<'_, AppState>,
) -> Result<bool, String> {
    {
        let mut config = app_state.config.write().map_err(|e| e.to_string())?;

        service
            .0
            .add_repo(&mut config.skills, repo)
            .map_err(|e| e.to_string())?;
    }

    app_state.save().map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn remove_skill_repo(
    owner: String,
    name: String,
    service: State<'_, SkillServiceState>,
    app_state: State<'_, AppState>,
) -> Result<bool, String> {
    {
        let mut config = app_state.config.write().map_err(|e| e.to_string())?;

        service
            .0
            .remove_repo(&mut config.skills, owner, name)
            .map_err(|e| e.to_string())?;
    }

    app_state.save().map_err(|e| e.to_string())?;

    Ok(true)
}
