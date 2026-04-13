use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::cli::DownloadFormat;

const QUALIFIER: &str = "io";
const ORGANIZATION: &str = "arxiv";
const APPLICATION: &str = "arxiv";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub data_dir: PathBuf,
    pub download_dir: PathBuf,
    pub default_format: DownloadFormat,
    pub request_timeout_secs: u64,
    pub user_agent: String,
    #[serde(default = "default_api_base_url")]
    pub api_base_url: String,
    #[serde(default = "default_download_base_url")]
    pub download_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiskConfig {
    pub data_dir: PathBuf,
    pub download_dir: PathBuf,
    pub default_format: DownloadFormat,
    pub request_timeout_secs: u64,
    pub user_agent: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let data_dir = data_dir_path();
        let download_dir = env::var_os("ARXIV_DOWNLOAD_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| data_dir.join("papers"));

        Self {
            data_dir,
            download_dir,
            default_format: DownloadFormat::Pdf,
            request_timeout_secs: 30,
            user_agent: format!("arxiv-cli/{}", env!("CARGO_PKG_VERSION")),
            api_base_url: default_api_base_url(),
            download_base_url: default_download_base_url(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimePaths {
    config_dir: PathBuf,
    config_file: PathBuf,
    data_dir: PathBuf,
    library_file: PathBuf,
    download_dir: PathBuf,
}

impl RuntimePaths {
    pub fn from_config(config: &AppConfig) -> Self {
        let config_dir = config_dir_path();
        let config_file = config_dir.join("config.toml");
        let data_dir = config.data_dir.clone();
        let library_file = data_dir.join("library.json");
        let download_dir = config.download_dir.clone();

        Self {
            config_dir,
            config_file,
            data_dir,
            library_file,
            download_dir,
        }
    }

    pub fn ensure(&self) -> Result<()> {
        fs::create_dir_all(&self.config_dir)?;
        fs::create_dir_all(&self.data_dir)?;
        fs::create_dir_all(&self.download_dir)?;
        Ok(())
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn config_file(&self) -> &Path {
        &self.config_file
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn library_file(&self) -> &Path {
        &self.library_file
    }

    pub fn download_dir(&self) -> &Path {
        &self.download_dir
    }
}

pub fn load_config() -> Result<AppConfig> {
    let config_path = config_dir_path().join("config.toml");
    if !config_path.exists() {
        let config = AppConfig::default();
        persist_config(&config)?;
        return Ok(apply_env_overrides(config));
    }

    let contents = fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read config from {}", config_path.display()))?;
    let config: DiskConfig = toml::from_str(&contents)
        .with_context(|| format!("failed to parse {}", config_path.display()))?;
    Ok(apply_env_overrides(AppConfig::from(config)))
}

pub fn persist_config(config: &AppConfig) -> Result<()> {
    let runtime_paths = RuntimePaths::from_config(config);
    runtime_paths.ensure()?;
    let contents = toml::to_string_pretty(&DiskConfig::from(config.clone()))?;
    fs::write(runtime_paths.config_file(), contents)
        .with_context(|| format!("failed to write {}", runtime_paths.config_file().display()))?;
    Ok(())
}

fn apply_env_overrides(mut config: AppConfig) -> AppConfig {
    if let Some(data_dir) = env::var_os("ARXIV_DATA_DIR") {
        config.data_dir = PathBuf::from(data_dir);
    }
    if let Some(download_dir) = env::var_os("ARXIV_DOWNLOAD_DIR") {
        config.download_dir = PathBuf::from(download_dir);
    }
    if let Ok(api_base_url) = std::env::var("ARXIV_API_BASE_URL") {
        config.api_base_url = api_base_url;
    }
    if let Ok(download_base_url) = std::env::var("ARXIV_DOWNLOAD_BASE_URL") {
        config.download_base_url = download_base_url;
    }
    config
}

fn project_dirs() -> ProjectDirs {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .expect("platform directories should be available")
}

fn config_dir_path() -> PathBuf {
    env::var_os("ARXIV_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| project_dirs().config_dir().to_path_buf())
}

fn data_dir_path() -> PathBuf {
    env::var_os("ARXIV_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| project_dirs().data_dir().to_path_buf())
}

fn default_api_base_url() -> String {
    "https://export.arxiv.org/api/query".to_string()
}

fn default_download_base_url() -> String {
    "https://arxiv.org".to_string()
}

impl From<DiskConfig> for AppConfig {
    fn from(value: DiskConfig) -> Self {
        Self {
            data_dir: value.data_dir,
            download_dir: value.download_dir,
            default_format: value.default_format,
            request_timeout_secs: value.request_timeout_secs,
            user_agent: value.user_agent,
            api_base_url: default_api_base_url(),
            download_base_url: default_download_base_url(),
        }
    }
}

impl From<AppConfig> for DiskConfig {
    fn from(value: AppConfig) -> Self {
        Self {
            data_dir: value.data_dir,
            download_dir: value.download_dir,
            default_format: value.default_format,
            request_timeout_secs: value.request_timeout_secs,
            user_agent: value.user_agent,
        }
    }
}
