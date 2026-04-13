use std::{collections::BTreeMap, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperSummary {
    pub id: String,
    pub title: String,
    pub abstract_text: String,
    pub authors: Vec<String>,
    pub primary_category: String,
    pub published: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperDetail {
    #[serde(flatten)]
    pub summary: PaperSummary,
    pub categories: Vec<String>,
    pub version: Option<String>,
    pub pdf_url: String,
    pub source_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub paper: PaperDetail,
    pub saved_at: DateTime<Utc>,
    pub downloaded_pdf: bool,
    pub downloaded_source: bool,
    pub pdf_path: Option<PathBuf>,
    pub source_path: Option<PathBuf>,
    pub tags: Vec<String>,
}

impl LibraryEntry {
    pub fn from_paper(paper: PaperDetail, saved_at: DateTime<Utc>) -> Self {
        Self {
            paper,
            saved_at,
            downloaded_pdf: false,
            downloaded_source: false,
            pdf_path: None,
            source_path: None,
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryIndex {
    pub version: u32,
    pub entries: BTreeMap<String, LibraryEntry>,
}

impl Default for LibraryIndex {
    fn default() -> Self {
        Self {
            version: 1,
            entries: BTreeMap::new(),
        }
    }
}
