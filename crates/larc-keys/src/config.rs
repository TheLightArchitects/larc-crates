use anyhow::{Context as _, Result};

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_path: String,
    pub jwt_secret: String,
    pub hmac_pepper: String,
    pub env: Env,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Env {
    Development,
    Production,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let host = std::env::var("LARC_HOST").unwrap_or_else(|_| "0.0.0.0".into());
        let port = std::env::var("LARC_PORT")
            .unwrap_or_else(|_| "3800".into())
            .parse::<u16>()
            .context("LARC_PORT must be a valid port number")?;
        let database_path =
            std::env::var("LARC_DATABASE_PATH").unwrap_or_else(|_| "./data/larc.db".into());
        let jwt_secret = std::env::var("LARC_JWT_SECRET").context("LARC_JWT_SECRET is required")?;
        let hmac_pepper =
            std::env::var("LARC_HMAC_PEPPER").context("LARC_HMAC_PEPPER is required")?;
        let env = match std::env::var("LARC_ENV").as_deref() {
            Ok("production") => Env::Production,
            _ => Env::Development,
        };
        Ok(Self {
            host,
            port,
            database_path,
            jwt_secret,
            hmac_pepper,
            env,
        })
    }
}
