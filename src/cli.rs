use std::path::PathBuf;

use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::parse_date;

#[derive(Debug, Parser)]
#[command(
    name = "arxiv",
    version,
    about = "Search, download, and manage arXiv papers."
)]
pub struct Cli {
    #[arg(long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Search(SearchArgs),
    Latest(LatestArgs),
    Show(ShowArgs),
    Download(DownloadArgs),
    Library {
        #[command(subcommand)]
        command: LibraryCommand,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Path,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  arxiv search \"skill\"\n  arxiv search \"diffusion models\" --category cs.CL --from 2025-01-01 --to 2025-12-31\n  arxiv search --title \"skill\" --include-abstract\n  arxiv search \"transformer\" --author \"Vaswani\" --sort submitted"
)]
pub struct SearchArgs {
    #[arg(
        help = "Keyword query. Multiple words are combined with AND, e.g. \"diffusion models\"."
    )]
    pub query: Option<String>,

    #[arg(
        long,
        default_value_t = 10,
        help = "Maximum number of results to return."
    )]
    pub limit: usize,

    #[arg(
        long,
        default_value_t = 0,
        help = "Zero-based offset into the result set."
    )]
    pub start: usize,

    #[arg(long, value_enum, default_value_t = SearchSort::Relevance, help = "Sort results by relevance, updated time, or submission time.")]
    pub sort: SearchSort,

    #[arg(long, value_enum, default_value_t = SortOrder::Desc, help = "Sort order for the selected sort field.")]
    pub order: SortOrder,

    #[arg(
        long,
        value_name = "CATEGORY",
        help = "arXiv category filter, e.g. cs.CL or cs.LG."
    )]
    pub category: Option<String>,

    #[arg(
        long,
        value_name = "AUTHOR",
        help = "Author filter, e.g. \"Vaswani\" or \"Yann LeCun\"."
    )]
    pub author: Option<String>,

    #[arg(
        long,
        value_name = "TITLE",
        help = "Title-only filter, e.g. \"diffusion model\"."
    )]
    pub title: Option<String>,

    #[arg(long, value_name = "YYYY-MM-DD", value_parser = parse_date, help = "Submitted-date lower bound, e.g. 2025-01-01.")]
    pub from: Option<NaiveDate>,

    #[arg(long, value_name = "YYYY-MM-DD", value_parser = parse_date, help = "Submitted-date upper bound, e.g. 2025-12-31.")]
    pub to: Option<NaiveDate>,

    #[arg(long, help = "Print machine-readable JSON output.")]
    pub json: bool,

    #[arg(long, help = "Include abstracts in table and JSON output.")]
    pub include_abstract: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  arxiv latest cs.CL\n  arxiv latest cs.CL --limit 20\n  arxiv latest cs.CL --from 2025-01-01 --to 2025-12-31\n  arxiv latest math.PR --include-abstract"
)]
pub struct LatestArgs {
    #[arg(help = "arXiv category to list, e.g. cs.CL, cs.LG, math.PR.")]
    pub category: String,

    #[arg(
        long,
        default_value_t = 10,
        help = "Maximum number of results to return."
    )]
    pub limit: usize,

    #[arg(long, value_name = "YYYY-MM-DD", value_parser = parse_date, help = "Submitted-date lower bound, e.g. 2025-01-01.")]
    pub from: Option<NaiveDate>,

    #[arg(long, value_name = "YYYY-MM-DD", value_parser = parse_date, help = "Submitted-date upper bound, e.g. 2025-12-31.")]
    pub to: Option<NaiveDate>,

    #[arg(long, help = "Print machine-readable JSON output.")]
    pub json: bool,

    #[arg(long, help = "Include abstracts in table and JSON output.")]
    pub include_abstract: bool,
}

#[derive(Debug, Args)]
pub struct ShowArgs {
    pub id: String,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct DownloadArgs {
    pub ids: Vec<String>,

    #[arg(long, value_enum, default_value_t = DownloadFormat::Pdf)]
    pub format: DownloadFormat,

    #[arg(long)]
    pub output: Option<PathBuf>,

    #[arg(long)]
    pub force: bool,

    #[arg(long)]
    pub no_library_update: bool,

    #[arg(long, default_value_t = 4)]
    pub jobs: usize,
}

#[derive(Debug, Subcommand)]
pub enum LibraryCommand {
    Add(LibraryAddArgs),
    List(LibraryListArgs),
    Show(LibraryShowArgs),
    Remove(LibraryRemoveArgs),
}

#[derive(Debug, Args)]
pub struct LibraryAddArgs {
    pub ids: Vec<String>,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct LibraryListArgs {
    #[arg(long)]
    pub downloaded_only: bool,

    #[arg(long)]
    pub category: Option<String>,

    #[arg(long)]
    pub author: Option<String>,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct LibraryShowArgs {
    pub id: String,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct LibraryRemoveArgs {
    pub id: String,

    #[arg(long)]
    pub purge_files: bool,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    Show,
    SetDownloadDir { path: PathBuf },
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchSort {
    Relevance,
    Updated,
    Submitted,
}

impl SearchSort {
    pub fn as_api_value(self) -> &'static str {
        match self {
            SearchSort::Relevance => "relevance",
            SearchSort::Updated => "lastUpdatedDate",
            SearchSort::Submitted => "submittedDate",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    pub fn as_api_value(self) -> &'static str {
        match self {
            SortOrder::Asc => "ascending",
            SortOrder::Desc => "descending",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadFormat {
    Pdf,
    Source,
    Both,
}
