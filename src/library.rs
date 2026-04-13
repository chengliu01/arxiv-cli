use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::{
    models::{LibraryEntry, LibraryIndex},
    normalize_id,
};

#[derive(Debug, Clone)]
pub struct LibraryStore {
    index: LibraryIndex,
}

impl LibraryStore {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self {
                index: LibraryIndex::default(),
            });
        }

        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let index: LibraryIndex = serde_json::from_str(&contents)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        Ok(Self { index })
    }

    pub fn persist(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(&self.index)?;
        fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&LibraryEntry> {
        self.index.entries.get(id)
    }

    pub fn upsert(&mut self, entry: LibraryEntry) {
        self.index
            .entries
            .insert(normalize_id(&entry.paper.summary.id), entry);
    }

    pub fn remove(&mut self, id: &str) -> Option<LibraryEntry> {
        self.index.entries.remove(id)
    }

    pub fn entries(&self) -> impl Iterator<Item = &LibraryEntry> {
        self.index.entries.values()
    }

    pub fn filtered(
        &self,
        downloaded_only: bool,
        category: Option<&str>,
        author: Option<&str>,
    ) -> Vec<&LibraryEntry> {
        let author = author.map(|value| value.to_lowercase());
        self.index
            .entries
            .values()
            .filter(|entry| !downloaded_only || entry.downloaded_pdf || entry.downloaded_source)
            .filter(|entry| {
                category
                    .map(|needle| {
                        entry
                            .paper
                            .categories
                            .iter()
                            .any(|category| category == needle)
                    })
                    .unwrap_or(true)
            })
            .filter(|entry| {
                author
                    .as_ref()
                    .map(|needle| {
                        entry
                            .paper
                            .summary
                            .authors
                            .iter()
                            .any(|author| author.to_lowercase().contains(needle))
                    })
                    .unwrap_or(true)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use chrono::Utc;

    use super::LibraryStore;
    use crate::models::{LibraryEntry, PaperDetail, PaperSummary};

    #[test]
    fn filters_downloaded_entries() {
        let paper = PaperDetail {
            summary: PaperSummary {
                id: "1234.5678".into(),
                title: "Title".into(),
                abstract_text: "Abstract".into(),
                authors: vec!["Alice".into()],
                primary_category: "cs.CL".into(),
                published: Utc::now(),
                updated: Utc::now(),
            },
            categories: vec!["cs.CL".into()],
            version: None,
            pdf_url: "https://arxiv.org/pdf/1234.5678".into(),
            source_url: "https://arxiv.org/e-print/1234.5678".into(),
        };
        let mut store = LibraryStore::load(Path::new("/tmp/does-not-exist")).unwrap();
        let mut entry = LibraryEntry::from_paper(paper, Utc::now());
        entry.downloaded_pdf = true;
        store.upsert(entry);
        assert_eq!(store.filtered(true, None, None).len(), 1);
    }
}
