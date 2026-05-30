use std::net::SocketAddr;
use std::path::PathBuf;

/// Runtime profile (KTD10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Dev,
    Prod,
}

impl Profile {
    pub fn from_env() -> Self {
        match std::env::var("ORCHESTRATOR_PROFILE")
            .unwrap_or_else(|_| "dev".into())
            .to_lowercase()
            .as_str()
        {
            "prod" | "production" => Self::Prod,
            _ => Self::Dev,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Prod => "prod",
        }
    }
}

/// HTTP server configuration from environment.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub profile: Profile,
    pub bind_addr: String,
    pub port: u16,
    pub data_dir: PathBuf,
    pub static_dir: Option<PathBuf>,
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let profile = Profile::from_env();
        let default_bind = match profile {
            Profile::Dev => "127.0.0.1",
            Profile::Prod => "0.0.0.0",
        };
        let bind_addr =
            std::env::var("ORCHESTRATOR_BIND_ADDR").unwrap_or_else(|_| default_bind.into());
        let port = std::env::var("ORCHESTRATOR_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(47821);
        let data_dir = std::env::var("ORCHESTRATOR_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(".data"));
        let static_dir = std::env::var("ORCHESTRATOR_STATIC_DIR").ok().map(PathBuf::from);

        Self {
            profile,
            bind_addr,
            port,
            data_dir,
            static_dir,
        }
    }

    pub fn database_url(&self) -> String {
        std::fs::create_dir_all(&self.data_dir).ok();
        let db_path = self.data_dir.join("orchestrator.db");
        format!("sqlite:{}", db_path.display())
    }

    pub fn socket_addr(&self) -> anyhow::Result<SocketAddr> {
        Ok(format!("{}:{}", self.bind_addr, self.port).parse()?)
    }
}
