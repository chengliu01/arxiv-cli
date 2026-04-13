use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use reqwest::Client;
use tar::Archive;
use tokio::io::AsyncWriteExt;

use crate::{cli::DownloadFormat, config::AppConfig, models::PaperDetail};

#[derive(Clone)]
pub struct Downloader {
    http: Arc<Client>,
}

#[derive(Debug, Default)]
pub struct DownloadReport {
    pub pdf_path: Option<PathBuf>,
    pub source_path: Option<PathBuf>,
}

impl Downloader {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let http = Client::builder()
            .user_agent(config.user_agent.clone())
            .timeout(std::time::Duration::from_secs(config.request_timeout_secs))
            .build()?;
        Ok(Self {
            http: Arc::new(http),
        })
    }

    pub async fn download(
        &self,
        paper: &PaperDetail,
        format: DownloadFormat,
        target_dir: &Path,
        force: bool,
    ) -> Result<DownloadReport> {
        tokio::fs::create_dir_all(target_dir).await?;
        let mut report = DownloadReport::default();

        match format {
            DownloadFormat::Pdf => {
                report.pdf_path = Some(self.download_one(&paper.pdf_url, target_dir, force).await?);
            }
            DownloadFormat::Source => {
                report.source_path = Some(
                    self.download_and_extract_source(&paper.source_url, target_dir, force)
                        .await?,
                );
            }
            DownloadFormat::Both => {
                report.pdf_path = Some(self.download_one(&paper.pdf_url, target_dir, force).await?);
                report.source_path = Some(
                    self.download_and_extract_source(&paper.source_url, target_dir, force)
                        .await?,
                );
            }
        }

        Ok(report)
    }

    async fn download_one(&self, url: &str, target_dir: &Path, force: bool) -> Result<PathBuf> {
        let response = self.http.get(url).send().await?.error_for_status()?;
        let filename = filename_from_response(&response)
            .or_else(|| filename_from_url(url))
            .unwrap_or_else(|| "download.bin".to_string());
        let path = target_dir.join(filename);

        if path.exists() && !force {
            return Ok(path);
        }

        let bytes = response.bytes().await?;
        let mut file = tokio::fs::File::create(&path)
            .await
            .with_context(|| format!("failed to create {}", path.display()))?;
        file.write_all(&bytes).await?;
        Ok(path)
    }

    async fn download_and_extract_source(
        &self,
        url: &str,
        target_dir: &Path,
        force: bool,
    ) -> Result<PathBuf> {
        let default_filename =
            source_filename_from_url(url).unwrap_or_else(|| "source.tar.gz".to_string());
        let default_extraction_dir = target_dir.join(source_extract_dir_name(&default_filename));

        if default_extraction_dir.exists() && !force {
            return Ok(default_extraction_dir);
        }

        let archive_path = if !force {
            let existing_archive = target_dir.join(&default_filename);
            if existing_archive.exists() {
                existing_archive
            } else {
                self.download_source_archive(url, target_dir, &default_filename)
                    .await?
            }
        } else {
            self.download_source_archive(url, target_dir, &default_filename)
                .await?
        };

        let extraction_dir = target_dir.join(source_extract_dir_name(
            archive_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&default_filename),
        ));
        extract_archive(&archive_path, &extraction_dir, force).await?;
        Ok(extraction_dir)
    }

    async fn download_source_archive(
        &self,
        url: &str,
        target_dir: &Path,
        default_filename: &str,
    ) -> Result<PathBuf> {
        let response = self.http.get(url).send().await?.error_for_status()?;
        let filename =
            filename_from_response(&response).unwrap_or_else(|| default_filename.to_string());
        let archive_path = target_dir.join(filename);
        let bytes = response.bytes().await?;
        let mut file = tokio::fs::File::create(&archive_path)
            .await
            .with_context(|| format!("failed to create {}", archive_path.display()))?;
        file.write_all(&bytes).await?;
        Ok(archive_path)
    }
}

fn filename_from_response(response: &reqwest::Response) -> Option<String> {
    let disposition = response
        .headers()
        .get(reqwest::header::CONTENT_DISPOSITION)?;
    let disposition = disposition.to_str().ok()?;
    for part in disposition.split(';') {
        let part = part.trim();
        if let Some(filename) = part.strip_prefix("filename=") {
            return Some(filename.trim_matches('"').to_string());
        }
    }
    None
}

fn filename_from_url(url: &str) -> Option<String> {
    let last = url.rsplit('/').next()?;
    if last.is_empty() {
        None
    } else {
        Some(last.to_string())
    }
}

fn source_filename_from_url(url: &str) -> Option<String> {
    filename_from_url(url).map(|name| format!("arXiv-{name}.tar.gz"))
}

fn source_extract_dir_name(filename: &str) -> String {
    filename
        .strip_suffix(".tar.gz")
        .or_else(|| filename.strip_suffix(".tgz"))
        .or_else(|| filename.strip_suffix(".gz"))
        .unwrap_or(filename)
        .to_string()
}

async fn extract_archive(archive_path: &Path, extraction_dir: &Path, force: bool) -> Result<()> {
    let archive_path = archive_path.to_path_buf();
    let extraction_dir = extraction_dir.to_path_buf();

    tokio::task::spawn_blocking(move || -> Result<()> {
        if extraction_dir.exists() {
            if force {
                std::fs::remove_dir_all(&extraction_dir).with_context(|| {
                    format!("failed to remove existing {}", extraction_dir.display())
                })?;
            } else {
                return Ok(());
            }
        }

        std::fs::create_dir_all(&extraction_dir)?;
        let archive_file = File::open(&archive_path)
            .with_context(|| format!("failed to open {}", archive_path.display()))?;
        let decoder = GzDecoder::new(archive_file);
        let mut archive = Archive::new(decoder);
        archive
            .unpack(&extraction_dir)
            .with_context(|| format!("failed to extract {}", archive_path.display()))?;
        Ok(())
    })
    .await
    .context("source extraction task failed")??;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{filename_from_url, source_extract_dir_name};

    #[test]
    fn derives_filename_from_url() {
        assert_eq!(
            filename_from_url("https://arxiv.org/pdf/1234.5678"),
            Some("1234.5678".into())
        );
    }

    #[test]
    fn derives_source_extract_dir_name() {
        assert_eq!(
            source_extract_dir_name("arXiv-1706.03762v7.tar.gz"),
            "arXiv-1706.03762v7"
        );
    }
}
