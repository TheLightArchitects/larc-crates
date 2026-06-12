use axum::{Json, extract::State, http::HeaderMap};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::{AppState, error::{AppError, Result}, handlers::keys::validate_key_sync};

fn extract_bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

// ── GET /v1/mcp/tools ─────────────────────────────────────────────────────────

pub async fn list_tools(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>> {
    let raw_key = extract_bearer(&headers).ok_or(AppError::Unauthorized)?;
    let conn = state.db.lock().map_err(|_| AppError::Internal(anyhow::anyhow!("db lock")))?;
    validate_key_sync(&conn, raw_key, &state.config.hmac_pepper)?;

    // TODO: forward to private platform and return its tool list.
    // For now returns a minimal stub so larc-proxy clients can connect.
    Ok(Json(json!({
        "tools": [
            {
                "name": "lightarchitects",
                "description": "Light Architects platform tools — /BUILD /PLAN /REVIEW and more.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "description": "Skill or action to invoke" }
                    },
                    "required": ["action"]
                }
            }
        ]
    })))
}

// ── POST /v1/mcp/call ─────────────────────────────────────────────────────────

pub async fn call_tool(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Json<Value>> {
    let raw_key = extract_bearer(&headers).ok_or(AppError::Unauthorized)?;
    let conn = state.db.lock().map_err(|_| AppError::Internal(anyhow::anyhow!("db lock")))?;
    let validated = validate_key_sync(&conn, raw_key, &state.config.hmac_pepper)?;

    tracing::info!(
        key_id = %validated.key_id,
        user_id = %validated.user_id,
        tool = ?body.get("name"),
        "mcp tool call"
    );

    // TODO: forward to private platform gateway.
    // Stub response until routing layer is wired.
    Ok(Json(json!({
        "content": [{
            "type": "text",
            "text": "Light Architects platform is initializing. \
                     Full skill routing coming soon."
        }],
        "isError": false
    })))
}
