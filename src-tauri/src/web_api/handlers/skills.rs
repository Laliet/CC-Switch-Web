#![cfg(feature = "web-server")]

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::Serialize;

use crate::{
    error::format_skill_error,
    error::AppError,
    services::{Skill, SkillRepo, SkillService},
    store::AppState,
};

use super::{ApiError, ApiResult};

#[derive(Serialize)]
pub struct SkillsResponse {
    pub skills: Vec<Skill>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

pub async fn list_skills(State(state): State<Arc<AppState>>) -> ApiResult<SkillsResponse> {
    let repos = {
        let cfg = state
            .config
            .read()
            .map_err(AppError::from)
            .map_err(ApiError::from)?;
        cfg.skills.repos.clone()
    };

    let service = SkillService::new().map_err(internal_error)?;
    let (skills, warnings) = service.list_skills(repos).await.map_err(internal_error)?;
    Ok(Json(SkillsResponse { skills, warnings }))
}

pub async fn install_skill(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InstallPayload>,
) -> ApiResult<bool> {
    let InstallPayload { directory } = payload;
    let service = SkillService::new().map_err(internal_error)?;

    // 收集仓库信息并查找目标技能
    let repos = {
        let cfg = state
            .config
            .read()
            .map_err(AppError::from)
            .map_err(ApiError::from)?;
        cfg.skills.repos.clone()
    };
    let (skills, _warnings) = service.list_skills(repos).await.map_err(internal_error)?;
    let skill = skills
        .iter()
        .find(|s| s.directory.eq_ignore_ascii_case(&directory))
        .ok_or_else(|| {
            ApiError::bad_request(format_skill_error(
                "SKILL_NOT_FOUND",
                &[("directory", directory.as_str())],
                Some("checkRepoUrl"),
            ))
        })?;

    if !skill.installed {
        let repo = SkillRepo {
            owner: skill.repo_owner.clone().ok_or_else(|| {
                ApiError::bad_request(format_skill_error(
                    "MISSING_REPO_INFO",
                    &[("directory", directory.as_str()), ("field", "owner")],
                    None,
                ))
            })?,
            name: skill.repo_name.clone().ok_or_else(|| {
                ApiError::bad_request(format_skill_error(
                    "MISSING_REPO_INFO",
                    &[("directory", directory.as_str()), ("field", "name")],
                    None,
                ))
            })?,
            branch: skill
                .repo_branch
                .clone()
                .unwrap_or_else(|| "main".to_string()),
            enabled: true,
            skills_path: skill.skills_path.clone(),
        };

        service
            .install_skill(directory.clone(), repo)
            .await
            .map_err(internal_error)?;
    }

    // 写入状态
    {
        let mut cfg = state
            .config
            .write()
            .map_err(AppError::from)
            .map_err(ApiError::from)?;
        cfg.skills.skills.insert(
            directory.clone(),
            crate::services::skill::SkillState {
                installed: true,
                installed_at: Utc::now(),
            },
        );
    }
    state.save().map_err(internal_error)?;

    Ok(Json(true))
}

pub async fn uninstall_skill(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InstallPayload>,
) -> ApiResult<bool> {
    let service = SkillService::new().map_err(internal_error)?;
    service
        .uninstall_skill(payload.directory.clone())
        .map_err(internal_error)?;

    {
        let mut cfg = state
            .config
            .write()
            .map_err(AppError::from)
            .map_err(ApiError::from)?;
        cfg.skills.skills.remove(&payload.directory);
    }
    state.save().map_err(internal_error)?;

    Ok(Json(true))
}

pub async fn list_repos(State(state): State<Arc<AppState>>) -> ApiResult<Vec<SkillRepo>> {
    let service = SkillService::new().map_err(internal_error)?;
    let repos = {
        let cfg = state
            .config
            .read()
            .map_err(AppError::from)
            .map_err(ApiError::from)?;
        service.list_repos(&cfg.skills)
    };
    Ok(Json(repos))
}

pub async fn add_repo(
    State(state): State<Arc<AppState>>,
    Json(repo): Json<SkillRepo>,
) -> ApiResult<bool> {
    let service = SkillService::new().map_err(internal_error)?;
    {
        let mut cfg = state
            .config
            .write()
            .map_err(AppError::from)
            .map_err(ApiError::from)?;
        service
            .add_repo(&mut cfg.skills, repo)
            .map_err(internal_error)?;
    }
    state.save().map_err(internal_error)?;
    Ok(Json(true))
}

pub async fn remove_repo(
    State(state): State<Arc<AppState>>,
    Path((owner, name)): Path<(String, String)>,
) -> ApiResult<bool> {
    let service = SkillService::new().map_err(internal_error)?;
    {
        let mut cfg = state
            .config
            .write()
            .map_err(AppError::from)
            .map_err(ApiError::from)?;
        service
            .remove_repo(&mut cfg.skills, owner, name)
            .map_err(internal_error)?;
    }
    state.save().map_err(internal_error)?;
    Ok(Json(true))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPayload {
    pub directory: String,
}

fn internal_error(err: impl ToString) -> ApiError {
    ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
