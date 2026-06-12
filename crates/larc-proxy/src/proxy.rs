use rmcp::{
    ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, Implementation,
        ListToolsResult, PaginatedRequestParam, ServerCapabilities,
        ServerInfo,
    },
    service::{RequestContext, RoleServer},
    ErrorData as McpError,
};
use tracing::{debug, warn};

use crate::config::Config;

pub struct ProxyHandler {
    client: reqwest::Client,
    config: Config,
}

impl ProxyHandler {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| anyhow::anyhow!("failed to build HTTP client: {e}"))?;
        Ok(Self { client, config })
    }

    fn auth(&self) -> String {
        format!("Bearer {}", self.config.api_key)
    }
}

impl ServerHandler for ProxyHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: rmcp::model::ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "larc-proxy".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: None,
                icons: None,
                website_url: Some("https://lightarchitects.ai".into()),
            },
            instructions: Some(
                "Light Architects platform — /BUILD /PLAN /REVIEW and more. \
                 See https://lightarchitects.ai for documentation."
                    .into(),
            ),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        debug!("list_tools → {}", self.config.tools_url());

        let response = self
            .client
            .get(self.config.tools_url())
            .header("Authorization", self.auth())
            .send()
            .await
            .map_err(|e| {
                warn!("list_tools HTTP error: {e}");
                McpError::internal_error(e.to_string(), None)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("list_tools {status}: {body}");
            return Err(McpError::internal_error(
                format!("API returned {status}"),
                None,
            ));
        }

        response
            .json::<ListToolsResult>()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        debug!("call_tool: {} → {}", request.name, self.config.call_url());

        let response = self
            .client
            .post(self.config.call_url())
            .header("Authorization", self.auth())
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                warn!("call_tool HTTP error: {e}");
                McpError::internal_error(e.to_string(), None)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("call_tool {status}: {body}");
            return Err(McpError::internal_error(
                format!("API returned {status}"),
                None,
            ));
        }

        response
            .json::<CallToolResult>()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }
}
