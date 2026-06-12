use anyhow::{Context as _, Result};

const DEFAULT_API_URL: &str = "https://api.lightarchitects.ai";

#[derive(Clone, Debug)]
pub struct Config {
    pub api_key: String,
    pub api_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("LIGHTARCHITECTS_API_KEY").context(
            "LIGHTARCHITECTS_API_KEY is required — get one at https://lightarchitects.ai",
        )?;
        let api_url =
            std::env::var("LIGHTARCHITECTS_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_owned());
        Ok(Self { api_key, api_url })
    }

    pub fn tools_url(&self) -> String {
        format!("{}/v1/mcp/tools", self.api_url)
    }

    pub fn call_url(&self) -> String {
        format!("{}/v1/mcp/call", self.api_url)
    }
}
