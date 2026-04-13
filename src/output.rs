use anyhow::Result;
use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL};

use crate::{
    config::{AppConfig, RuntimePaths},
    models::{LibraryEntry, PaperDetail, PaperSummary},
};

pub fn print_papers(papers: &[PaperSummary], as_json: bool, include_abstract: bool) -> Result<()> {
    if as_json {
        let items = papers
            .iter()
            .map(|paper| {
                let mut value = serde_json::json!({
                    "id": paper.id,
                    "title": paper.title,
                    "authors": paper.authors,
                    "primary_category": paper.primary_category,
                    "published": paper.published,
                    "updated": paper.updated,
                });
                if include_abstract {
                    value["abstract_text"] = serde_json::json!(paper.abstract_text);
                }
                value
            })
            .collect::<Vec<_>>();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }

    let mut table = base_table();
    let mut header = vec!["id", "title", "authors", "category", "published"];
    if include_abstract {
        header.push("abstract");
    }
    table.set_header(header);
    for paper in papers {
        let mut row = vec![
            Cell::new(&paper.id),
            Cell::new(&paper.title),
            Cell::new(paper.authors.join(", ")),
            Cell::new(&paper.primary_category),
            Cell::new(paper.published.date_naive()),
        ];
        if include_abstract {
            row.push(Cell::new(&paper.abstract_text));
        }
        table.add_row(row);
    }
    println!("{table}");
    Ok(())
}

pub fn print_paper_detail(
    paper: &PaperDetail,
    entry: Option<&LibraryEntry>,
    as_json: bool,
) -> Result<()> {
    if as_json {
        let value = serde_json::json!({
            "paper": paper,
            "library_entry": entry,
        });
        println!("{}", serde_json::to_string_pretty(&value)?);
        return Ok(());
    }

    println!("id: {}", paper.summary.id);
    println!("title: {}", paper.summary.title);
    println!("authors: {}", paper.summary.authors.join(", "));
    println!("primary_category: {}", paper.summary.primary_category);
    println!("categories: {}", paper.categories.join(", "));
    println!("published: {}", paper.summary.published);
    println!("updated: {}", paper.summary.updated);
    println!("version: {}", paper.version.as_deref().unwrap_or("unknown"));
    println!("pdf_url: {}", paper.pdf_url);
    println!("source_url: {}", paper.source_url);
    println!("abstract: {}", paper.summary.abstract_text);
    if let Some(entry) = entry {
        println!("saved_at: {}", entry.saved_at);
        println!("downloaded_pdf: {}", entry.downloaded_pdf);
        println!("downloaded_source: {}", entry.downloaded_source);
        println!(
            "pdf_path: {}",
            entry
                .pdf_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "-".into())
        );
        println!(
            "source_path: {}",
            entry
                .source_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "-".into())
        );
    }
    Ok(())
}

pub fn print_library_entries(entries: Vec<&LibraryEntry>, as_json: bool) -> Result<()> {
    if as_json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    let mut table = base_table();
    table.set_header(vec!["id", "title", "category", "pdf", "source"]);
    for entry in entries {
        table.add_row(vec![
            Cell::new(&entry.paper.summary.id),
            Cell::new(&entry.paper.summary.title),
            Cell::new(&entry.paper.summary.primary_category),
            Cell::new(entry.downloaded_pdf),
            Cell::new(entry.downloaded_source),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub fn print_library_entry(entry: &LibraryEntry, as_json: bool) -> Result<()> {
    if as_json {
        println!("{}", serde_json::to_string_pretty(entry)?);
        return Ok(());
    }
    print_paper_detail(&entry.paper, Some(entry), false)
}

pub fn print_config(config: &AppConfig, runtime_paths: &RuntimePaths) -> Result<()> {
    let value = serde_json::json!({
        "config": config,
        "paths": {
            "config_dir": runtime_paths.config_dir(),
            "config_file": runtime_paths.config_file(),
            "data_dir": runtime_paths.data_dir(),
            "library_file": runtime_paths.library_file(),
            "download_dir": runtime_paths.download_dir(),
        }
    });
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

pub fn print_paths(runtime_paths: &RuntimePaths) -> Result<()> {
    println!("config_dir={}", runtime_paths.config_dir().display());
    println!("config_file={}", runtime_paths.config_file().display());
    println!("data_dir={}", runtime_paths.data_dir().display());
    println!("library_file={}", runtime_paths.library_file().display());
    println!("download_dir={}", runtime_paths.download_dir().display());
    Ok(())
}

fn base_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}
