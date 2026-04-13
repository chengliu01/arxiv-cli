pub mod arxiv_api;
pub mod cli;
pub mod config;
pub mod downloader;
pub mod library;
pub mod models;
pub mod output;

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, NaiveDate, Utc};
use clap::Parser;
use futures_util::StreamExt;
use tracing::debug;

use crate::arxiv_api::{ArxivClient, SearchParams};
use crate::cli::{
    Cli, Command, ConfigCommand, DownloadArgs, LibraryCommand, SearchSort, SortOrder,
};
use crate::config::RuntimePaths;
use crate::downloader::Downloader;
use crate::library::LibraryStore;
use crate::models::LibraryEntry;

struct DownloadSummary {
    id: String,
    pdf_path: Option<PathBuf>,
    source_path: Option<PathBuf>,
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose)?;

    let mut config = config::load_config()?;
    let runtime_paths = RuntimePaths::from_config(&config);
    runtime_paths.ensure()?;

    let client = ArxivClient::new(&config)?;
    let mut library = LibraryStore::load(runtime_paths.library_file())?;

    match cli.command {
        Command::Search(args) => {
            let params = SearchParams {
                query: args.query.unwrap_or_default(),
                limit: args.limit,
                start: args.start,
                sort: args.sort,
                order: args.order,
                category: args.category,
                author: args.author,
                title: args.title,
                from: args.from,
                to: args.to,
            };
            let results = client.search(&params).await?;
            output::print_papers(&results, args.json, args.include_abstract)?;
        }
        Command::Latest(args) => {
            let params = SearchParams {
                query: String::new(),
                limit: args.limit,
                start: 0,
                sort: SearchSort::Submitted,
                order: SortOrder::Desc,
                category: Some(args.category),
                author: None,
                title: None,
                from: args.from,
                to: args.to,
            };
            let results = client.search(&params).await?;
            output::print_papers(&results, args.json, args.include_abstract)?;
        }
        Command::Show(args) => {
            let paper = client.fetch_paper(&args.id).await?;
            let entry = library.get(&normalize_id(&args.id));
            output::print_paper_detail(&paper, entry, args.json)?;
        }
        Command::Download(args) => {
            let should_persist_library = !args.no_library_update;
            handle_download(&client, &mut library, &config, args).await?;
            if should_persist_library {
                library.persist(runtime_paths.library_file())?;
            }
        }
        Command::Library { command } => match command {
            LibraryCommand::Add(args) => {
                for id in args.ids {
                    let paper = client.fetch_paper(&id).await?;
                    let entry = LibraryEntry::from_paper(paper, Utc::now());
                    library.upsert(entry);
                }
                library.persist(runtime_paths.library_file())?;
                output::print_library_entries(library.entries().collect(), args.json)?;
            }
            LibraryCommand::List(args) => {
                let entries = library.filtered(
                    args.downloaded_only,
                    args.category.as_deref(),
                    args.author.as_deref(),
                );
                output::print_library_entries(entries, args.json)?;
            }
            LibraryCommand::Show(args) => {
                let id = normalize_id(&args.id);
                let entry = library
                    .get(&id)
                    .with_context(|| format!("paper `{id}` not found in library"))?;
                output::print_library_entry(entry, args.json)?;
            }
            LibraryCommand::Remove(args) => {
                let id = normalize_id(&args.id);
                let removed = library
                    .remove(&id)
                    .with_context(|| format!("paper `{id}` not found in library"))?;
                if args.purge_files {
                    purge_entry_files(&removed)?;
                }
                library.persist(runtime_paths.library_file())?;
                println!("removed {id}");
            }
        },
        Command::Config { command } => match command {
            ConfigCommand::Show => {
                output::print_config(&config, &runtime_paths)?;
            }
            ConfigCommand::SetDownloadDir { path } => {
                let resolved = absolutize(path)?;
                config.download_dir = resolved;
                config::persist_config(&config)?;
                println!("{}", config.download_dir.display());
            }
        },
        Command::Path => {
            output::print_paths(&runtime_paths)?;
        }
    }

    Ok(())
}

async fn handle_download(
    client: &ArxivClient,
    library: &mut LibraryStore,
    config: &config::AppConfig,
    args: DownloadArgs,
) -> Result<()> {
    let output_root = args
        .output
        .map(absolutize)
        .transpose()?
        .unwrap_or_else(|| config.download_dir.clone());
    tokio::fs::create_dir_all(&output_root).await?;

    let downloader = Downloader::new(config)?;
    let download_jobs = args.jobs.max(1);

    let mut papers = Vec::new();
    for id in &args.ids {
        papers.push(client.fetch_paper(id).await?);
    }

    let results = futures_util::stream::iter(papers.into_iter().map(|paper| {
        let downloader = downloader.clone();
        let output_root = output_root.clone();
        let format = args.format;
        let force = args.force;
        async move {
            let target_dir = output_root.join(normalize_id(&paper.summary.id));
            let report = downloader
                .download(&paper, format, &target_dir, force)
                .await?;
            Ok::<_, anyhow::Error>((paper, report))
        }
    }))
    .buffer_unordered(download_jobs)
    .collect::<Vec<_>>()
    .await;

    let mut failures = Vec::new();
    let mut downloads = Vec::new();

    for result in results {
        match result {
            Ok((paper, report)) => {
                let normalized_id = normalize_id(&paper.summary.id);
                let pdf_path = report.pdf_path.clone();
                let source_path = report.source_path.clone();
                downloads.push(DownloadSummary {
                    id: normalized_id.clone(),
                    pdf_path,
                    source_path,
                });
                if !args.no_library_update {
                    let mut entry = library
                        .get(&normalized_id)
                        .cloned()
                        .unwrap_or_else(|| LibraryEntry::from_paper(paper.clone(), Utc::now()));
                    entry.paper = paper;
                    if let Some(path) = report.pdf_path {
                        entry.downloaded_pdf = true;
                        entry.pdf_path = Some(path);
                    }
                    if let Some(path) = report.source_path {
                        entry.downloaded_source = true;
                        entry.source_path = Some(path);
                    }
                    library.upsert(entry);
                }
            }
            Err(err) => failures.push(err),
        }
    }

    if !downloads.is_empty() {
        for download in &downloads {
            println!("downloaded {}", download.id);
            if let Some(path) = &download.pdf_path {
                println!("  pdf: {}", path.display());
            }
            if let Some(path) = &download.source_path {
                println!("  source: {}", path.display());
            }
        }
    }

    if failures.is_empty() {
        return Ok(());
    }

    for err in &failures {
        eprintln!("download failed: {err:#}");
    }

    bail!("{} download(s) failed", failures.len())
}

fn purge_entry_files(entry: &LibraryEntry) -> Result<()> {
    for path in [&entry.pdf_path, &entry.source_path].into_iter().flatten() {
        if path.exists() {
            if path.is_dir() {
                std::fs::remove_dir_all(path)?;
            } else {
                std::fs::remove_file(path)?;
            }
        }
    }

    let mut parent_dirs = Vec::new();
    if let Some(path) = &entry.pdf_path {
        if let Some(parent) = path.parent() {
            parent_dirs.push(parent.to_path_buf());
        }
    }
    if let Some(path) = &entry.source_path {
        if let Some(parent) = path.parent() {
            parent_dirs.push(parent.to_path_buf());
        }
    }
    parent_dirs.sort();
    parent_dirs.dedup();

    for dir in parent_dirs {
        if dir.exists() && dir.read_dir()?.next().is_none() {
            std::fs::remove_dir(dir)?;
        }
    }

    Ok(())
}

fn absolutize(path: PathBuf) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

pub fn normalize_id(input: &str) -> String {
    let trimmed = input.trim();
    if let Some(rest) = trimmed.strip_prefix("http://arxiv.org/abs/") {
        return rest.to_string();
    }
    if let Some(rest) = trimmed.strip_prefix("https://arxiv.org/abs/") {
        return rest.to_string();
    }
    trimmed.to_string()
}

pub fn parse_date(value: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .with_context(|| format!("invalid date `{value}`, expected YYYY-MM-DD"))
}

fn init_tracing(verbose: bool) -> Result<()> {
    let filter = if verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .try_init()
        .map_err(|err| anyhow!("failed to initialize tracing: {err}"))?;
    Ok(())
}

pub fn parse_rfc3339_to_utc(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc))
}

pub fn ensure_date_range(from: Option<NaiveDate>, to: Option<NaiveDate>) -> Result<()> {
    if let (Some(from), Some(to)) = (from, to) {
        if from > to {
            bail!("`--from` must be before or equal to `--to`");
        }
    }
    debug!("validated date range");
    Ok(())
}
