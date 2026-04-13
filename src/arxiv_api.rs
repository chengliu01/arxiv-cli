use anyhow::{Context, Result, anyhow, bail};
use chrono::{NaiveDate, Utc};
use reqwest::Client;
use roxmltree::Document;

use crate::{
    cli::{SearchSort, SortOrder},
    config::AppConfig,
    models::{PaperDetail, PaperSummary},
    normalize_id, parse_rfc3339_to_utc,
};

#[derive(Debug, Clone)]
pub struct SearchParams {
    pub query: String,
    pub limit: usize,
    pub start: usize,
    pub sort: SearchSort,
    pub order: SortOrder,
    pub category: Option<String>,
    pub author: Option<String>,
    pub title: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

#[derive(Clone)]
pub struct ArxivClient {
    http: Client,
    api_base_url: String,
    download_base_url: String,
}

impl ArxivClient {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let http = Client::builder()
            .user_agent(config.user_agent.clone())
            .timeout(std::time::Duration::from_secs(config.request_timeout_secs))
            .build()?;

        Ok(Self {
            http,
            api_base_url: config.api_base_url.clone(),
            download_base_url: config.download_base_url.clone(),
        })
    }

    pub async fn search(&self, params: &SearchParams) -> Result<Vec<PaperSummary>> {
        crate::ensure_date_range(params.from, params.to)?;
        let query = build_search_query(params)?;
        let response = self
            .http
            .get(&self.api_base_url)
            .query(&[
                ("search_query", query.as_str()),
                ("start", &params.start.to_string()),
                ("max_results", &params.limit.to_string()),
                ("sortBy", params.sort.as_api_value()),
                ("sortOrder", params.order.as_api_value()),
            ])
            .send()
            .await?
            .error_for_status()?;
        let body = response.text().await?;
        parse_feed(&body, &self.download_base_url)
            .map(|papers| papers.into_iter().map(|paper| paper.summary).collect())
    }

    pub async fn fetch_paper(&self, id: &str) -> Result<PaperDetail> {
        let normalized = normalize_id(id);
        let response = self
            .http
            .get(&self.api_base_url)
            .query(&[("id_list", normalized.as_str())])
            .send()
            .await?
            .error_for_status()?;
        let body = response.text().await?;
        let mut papers = parse_feed(&body, &self.download_base_url)?;
        papers
            .pop()
            .ok_or_else(|| anyhow!("paper `{normalized}` not found"))
    }
}

pub fn build_search_query(params: &SearchParams) -> Result<String> {
    let mut parts = Vec::new();

    let query = params.query.trim();
    if !query.is_empty() {
        parts.extend(build_all_field_clauses(query));
    }

    if let Some(category) = &params.category {
        let category = category.trim();
        if !category.is_empty() {
            parts.push(format!("cat:{category}"));
        }
    }
    if let Some(author) = &params.author {
        let author = author.trim();
        if !author.is_empty() {
            parts.push(build_phrase_clause("au", author));
        }
    }
    if let Some(title) = &params.title {
        let title = title.trim();
        if !title.is_empty() {
            parts.push(build_phrase_clause("ti", title));
        }
    }
    if params.from.is_some() || params.to.is_some() {
        let from = params
            .from
            .map(|date| date.format("%Y%m%d0000").to_string())
            .unwrap_or_else(|| "000000000000".to_string());
        let to = params
            .to
            .map(|date| date.format("%Y%m%d2359").to_string())
            .unwrap_or_else(|| Utc::now().format("%Y%m%d2359").to_string());
        parts.push(format!("submittedDate:[{from} TO {to}]"));
    }

    if parts.is_empty() {
        bail!("search query cannot be empty");
    }

    Ok(parts.join(" AND "))
}

fn build_all_field_clauses(query: &str) -> Vec<String> {
    tokenize_query(query)
        .into_iter()
        .map(|term| {
            if term.contains(char::is_whitespace) {
                format!("all:\"{}\"", escape_quotes(&term))
            } else {
                format!("all:{term}")
            }
        })
        .collect()
}

fn build_phrase_clause(field: &str, value: &str) -> String {
    if value.contains(char::is_whitespace) {
        format!("{field}:\"{}\"", escape_quotes(value))
    } else {
        format!("{field}:{value}")
    }
}

fn escape_quotes(value: &str) -> String {
    value.replace('"', "\\\"")
}

fn tokenize_query(query: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in query.chars() {
        match ch {
            '"' => {
                if in_quotes {
                    if !current.is_empty() {
                        tokens.push(std::mem::take(&mut current));
                    }
                    in_quotes = false;
                } else {
                    if !current.trim().is_empty() {
                        tokens.extend(current.split_whitespace().map(ToString::to_string));
                        current.clear();
                    }
                    in_quotes = true;
                }
            }
            ch if ch.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        if in_quotes {
            tokens.push(current);
        } else {
            tokens.extend(current.split_whitespace().map(ToString::to_string));
        }
    }

    tokens
}

pub fn parse_feed(xml: &str, download_base_url: &str) -> Result<Vec<PaperDetail>> {
    let document = Document::parse(xml).context("failed to parse arXiv XML response")?;
    let mut papers = Vec::new();

    for entry in document
        .descendants()
        .filter(|node| node.has_tag_name("entry"))
    {
        let id_url = child_text(&entry, "id")?;
        let normalized_id = normalize_id(id_url.rsplit('/').next().unwrap_or(&id_url));
        let title = sanitize_text(&child_text(&entry, "title")?);
        let abstract_text = sanitize_text(&child_text(&entry, "summary")?);
        let published = parse_rfc3339_to_utc(&child_text(&entry, "published")?)?;
        let updated = parse_rfc3339_to_utc(&child_text(&entry, "updated")?)?;
        let authors = entry
            .children()
            .filter(|node| node.has_tag_name("author"))
            .filter_map(|author| child_text(&author, "name").ok())
            .collect::<Vec<_>>();
        let primary_category = entry
            .children()
            .find(|node| node.tag_name().name() == "primary_category")
            .and_then(|node| node.attribute("term"))
            .unwrap_or_default()
            .to_string();
        let categories = entry
            .children()
            .filter(|node| node.has_tag_name("category"))
            .filter_map(|node| node.attribute("term").map(ToString::to_string))
            .collect::<Vec<_>>();
        let version = normalized_id
            .rsplit_once('v')
            .filter(|(_, suffix)| suffix.chars().all(|ch| ch.is_ascii_digit()))
            .map(|(_, version)| format!("v{version}"));

        let summary = PaperSummary {
            id: normalized_id.clone(),
            title,
            abstract_text,
            authors,
            primary_category,
            published,
            updated,
        };
        let paper = PaperDetail {
            summary,
            categories,
            version,
            pdf_url: format!("{download_base_url}/pdf/{}", normalized_id),
            source_url: format!("{download_base_url}/e-print/{}", normalized_id),
        };
        papers.push(paper);
    }

    Ok(papers)
}

fn child_text<'a, 'input>(node: &'a roxmltree::Node<'a, 'input>, tag_name: &str) -> Result<String> {
    node.children()
        .find(|child| child.has_tag_name(tag_name))
        .and_then(|child| child.text())
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("missing `{tag_name}` in arXiv response"))
}

fn sanitize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{SearchParams, build_search_query, parse_feed, tokenize_query};
    use crate::cli::{SearchSort, SortOrder};

    #[test]
    fn builds_search_query_with_filters_and_dates() {
        let params = SearchParams {
            query: "transformer".into(),
            limit: 10,
            start: 0,
            sort: SearchSort::Relevance,
            order: SortOrder::Desc,
            category: Some("cs.CL".into()),
            author: Some("Vaswani".into()),
            title: Some("attention".into()),
            from: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            to: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        };

        let query = build_search_query(&params).unwrap();
        assert_eq!(
            query,
            "all:transformer AND cat:cs.CL AND au:Vaswani AND ti:attention AND submittedDate:[202401010000 TO 202412312359]"
        );
    }

    #[test]
    fn splits_multiple_keywords_into_and_clauses() {
        let params = SearchParams {
            query: "diffusion models".into(),
            limit: 10,
            start: 0,
            sort: SearchSort::Relevance,
            order: SortOrder::Desc,
            category: None,
            author: None,
            title: None,
            from: None,
            to: None,
        };

        let query = build_search_query(&params).unwrap();
        assert_eq!(query, "all:diffusion AND all:models");
    }

    #[test]
    fn preserves_quoted_query_as_phrase() {
        assert_eq!(
            tokenize_query(r#"diffusion "large language model""#),
            vec!["diffusion", "large language model"]
        );
    }

    #[test]
    fn parses_atom_feed() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <feed xmlns="http://www.w3.org/2005/Atom" xmlns:arxiv="http://arxiv.org/schemas/atom">
          <entry>
            <id>http://arxiv.org/abs/2501.00001v2</id>
            <updated>2025-01-02T00:00:00Z</updated>
            <published>2025-01-01T00:00:00Z</published>
            <title> Test Paper </title>
            <summary> Some abstract text. </summary>
            <author><name>Alice</name></author>
            <author><name>Bob</name></author>
            <arxiv:primary_category term="cs.CL" />
            <category term="cs.CL" />
            <category term="cs.AI" />
          </entry>
        </feed>"#;

        let papers = parse_feed(xml, "https://arxiv.org").unwrap();
        assert_eq!(papers.len(), 1);
        assert_eq!(papers[0].summary.id, "2501.00001v2");
        assert_eq!(papers[0].summary.primary_category, "cs.CL");
        assert_eq!(papers[0].summary.abstract_text, "Some abstract text.");
        assert_eq!(papers[0].categories, vec!["cs.CL", "cs.AI"]);
    }
}
