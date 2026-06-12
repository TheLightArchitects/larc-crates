use axum::{Extension, Json, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    AppState,
    auth::{hash_password, issue_token, verify_password, verify_token},
    error::{AppError, Result},
    models::User,
};

// ── Register ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>> {
    if req.email.is_empty() || req.password.len() < 8 {
        return Err(AppError::BadRequest(
            "email required and password must be ≥8 chars".into(),
        ));
    }

    let id = Uuid::new_v4().to_string();
    let hash = hash_password(&req.password).map_err(AppError::Internal)?;

    let conn = state.db.lock().map_err(|_| AppError::Internal(anyhow::anyhow!("db lock")))?;
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM users WHERE email = ?1",
            [&req.email],
            |r| r.get::<_, i64>(0),
        )
        .map(|n| n > 0)
        .unwrap_or(false);

    if exists {
        return Err(AppError::Conflict("email already registered".into()));
    }

    conn.execute(
        "INSERT INTO users (id, email, name, password_hash) VALUES (?1, ?2, ?3, ?4)",
        [&id, &req.email, &req.name, &hash],
    )?;

    let user = User {
        id: id.clone(),
        email: req.email.clone(),
        name: req.name.clone(),
        password_hash: hash,
        tier: "free".into(),
        created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        updated_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    };

    let token = issue_token(&id, &req.email, &state.config.jwt_secret)
        .map_err(AppError::Internal)?;

    Ok(Json(AuthResponse { token, user }))
}

// ── Login ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    let conn = state.db.lock().map_err(|_| AppError::Internal(anyhow::anyhow!("db lock")))?;

    let user = conn
        .query_row(
            "SELECT id, email, name, password_hash, tier, created_at, updated_at \
             FROM users WHERE email = ?1",
            [&req.email],
            |r| {
                Ok(User {
                    id: r.get(0)?,
                    email: r.get(1)?,
                    name: r.get(2)?,
                    password_hash: r.get(3)?,
                    tier: r.get(4)?,
                    created_at: r.get(5)?,
                    updated_at: r.get(6)?,
                })
            },
        )
        .map_err(|_| AppError::Unauthorized)?;

    if !verify_password(&req.password, &user.password_hash).map_err(AppError::Internal)? {
        return Err(AppError::Unauthorized);
    }

    let token = issue_token(&user.id, &user.email, &state.config.jwt_secret)
        .map_err(AppError::Internal)?;

    Ok(Json(AuthResponse { token, user }))
}

// ── Me ────────────────────────────────────────────────────────────────────────

pub async fn me(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
) -> Result<Json<User>> {
    let conn = state.db.lock().map_err(|_| AppError::Internal(anyhow::anyhow!("db lock")))?;

    let user = conn
        .query_row(
            "SELECT id, email, name, password_hash, tier, created_at, updated_at \
             FROM users WHERE id = ?1",
            [&claims.sub],
            |r| {
                Ok(User {
                    id: r.get(0)?,
                    email: r.get(1)?,
                    name: r.get(2)?,
                    password_hash: r.get(3)?,
                    tier: r.get(4)?,
                    created_at: r.get(5)?,
                    updated_at: r.get(6)?,
                })
            },
        )
        .map_err(|_| AppError::NotFound)?;

    Ok(Json(user))
}

// ── JWT extractor middleware ──────────────────────────────────────────────────

pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response> {
    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims =
        verify_token(token, &state.config.jwt_secret).map_err(|_| AppError::Unauthorized)?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
