use axum::{
    Extension, Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    AppState,
    auth::{generate_api_key, hash_api_key, verify_api_key},
    error::{AppError, Result},
    models::{ApiKey, CreatedKey},
};

// ── Create ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
    #[serde(default = "default_env")]
    pub environment: String,
    pub expires_at: Option<String>,
}

fn default_env() -> String {
    "live".into()
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Json(req): Json<CreateKeyRequest>,
) -> Result<Json<CreatedKey>> {
    if req.name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }

    let id = Uuid::new_v4().to_string();
    let secret = generate_api_key(&req.environment);
    let prefix = secret.chars().take(12).collect::<String>();
    let last_four = secret.chars().rev().take(4).collect::<String>().chars().rev().collect::<String>();
    let key_hash = hash_api_key(&secret, &state.config.hmac_pepper).map_err(AppError::Internal)?;

    let conn = state.db.lock().map_err(|_| AppError::Internal(anyhow::anyhow!("db lock")))?;

    conn.execute(
        "INSERT INTO api_keys \
         (id, user_id, name, key_hash, prefix, last_four, environment, lineage_id, expires_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            &id,
            &claims.sub,
            &req.name,
            &key_hash,
            &prefix,
            &last_four,
            &req.environment,
            &id, // lineage_id = own id for new keys
            &req.expires_at,
        ],
    )?;

    let key = ApiKey {
        id: id.clone(),
        user_id: claims.sub.clone(),
        name: req.name,
        key_hash,
        prefix,
        last_four,
        environment: req.environment,
        status: "active".into(),
        lineage_id: id,
        created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        expires_at: req.expires_at,
        last_used_at: None,
        revoked_at: None,
    };

    Ok(Json(CreatedKey { key, secret }))
}

// ── List ──────────────────────────────────────────────────────────────────────

pub async fn list(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
) -> Result<Json<Vec<ApiKey>>> {
    let conn = state.db.lock().map_err(|_| AppError::Internal(anyhow::anyhow!("db lock")))?;

    let mut stmt = conn.prepare(
        "SELECT id, user_id, name, key_hash, prefix, last_four, environment, status, \
         lineage_id, created_at, expires_at, last_used_at, revoked_at \
         FROM api_keys WHERE user_id = ?1 AND status = 'active' ORDER BY created_at DESC",
    )?;

    let keys = stmt
        .query_map([&claims.sub], |r| {
            Ok(ApiKey {
                id: r.get(0)?,
                user_id: r.get(1)?,
                name: r.get(2)?,
                key_hash: r.get(3)?,
                prefix: r.get(4)?,
                last_four: r.get(5)?,
                environment: r.get(6)?,
                status: r.get(7)?,
                lineage_id: r.get(8)?,
                created_at: r.get(9)?,
                expires_at: r.get(10)?,
                last_used_at: r.get(11)?,
                revoked_at: r.get(12)?,
            })
        })?
        .filter_map(std::result::Result::ok)
        .collect();

    Ok(Json(keys))
}

// ── Revoke ────────────────────────────────────────────────────────────────────

pub async fn revoke(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let conn = state.db.lock().map_err(|_| AppError::Internal(anyhow::anyhow!("db lock")))?;
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let rows = conn.execute(
        "UPDATE api_keys SET status = 'revoked', revoked_at = ?1 \
         WHERE id = ?2 AND user_id = ?3 AND status = 'active'",
        rusqlite::params![&now, &id, &claims.sub],
    )?;

    if rows == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "revoked": true })))
}

// ── Validate (internal — used by MCP handler) ─────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ValidatedKey {
    pub key_id: String,
    pub user_id: String,
    pub environment: String,
}

pub fn validate_key_sync(
    conn: &rusqlite::Connection,
    raw_key: &str,
    pepper: &str,
) -> Result<ValidatedKey> {
    // keys start with "la_" — extract prefix to narrow DB lookup
    let prefix: String = raw_key.chars().take(12).collect();

    let mut stmt = conn.prepare(
        "SELECT id, user_id, key_hash, environment, expires_at \
         FROM api_keys WHERE prefix = ?1 AND status = 'active'",
    )?;

    let candidates: Vec<(String, String, String, String, Option<String>)> = stmt
        .query_map([&prefix], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .filter_map(std::result::Result::ok)
        .collect();

    for (id, user_id, key_hash, environment, expires_at) in candidates {
        if !verify_api_key(raw_key, pepper, &key_hash).unwrap_or(false) {
            continue;
        }
        // check expiry
        if let Some(exp) = &expires_at {
            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            if *exp < now {
                return Err(AppError::Unauthorized);
            }
        }
        // update last_used_at
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let _ = conn.execute(
            "UPDATE api_keys SET last_used_at = ?1 WHERE id = ?2",
            rusqlite::params![&now, &id],
        );
        return Ok(ValidatedKey { key_id: id, user_id, environment });
    }

    Err(AppError::Unauthorized)
}
