use std::fs;

use assert_cmd::Command;
use flate2::{Compression, write::GzEncoder};
use predicates::prelude::*;
use tar::{Builder, Header};
use tempfile::TempDir;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path, query_param},
};

fn feed_xml(id: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom" xmlns:arxiv="http://arxiv.org/schemas/atom">
  <entry>
    <id>http://arxiv.org/abs/{id}</id>
    <updated>2025-01-02T00:00:00Z</updated>
    <published>2025-01-01T00:00:00Z</published>
    <title>Test Paper {id}</title>
    <summary>Summary for {id}</summary>
    <author><name>Alice Example</name></author>
    <arxiv:primary_category term="cs.CL" />
    <category term="cs.CL" />
  </entry>
</feed>"#
    )
}

fn source_archive_bytes() -> Vec<u8> {
    let encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut tar = Builder::new(encoder);
    let contents = b"\\documentclass{article}\n\\begin{document}\nHello\n\\end{document}\n";
    let mut header = Header::new_gnu();
    header.set_size(contents.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append_data(&mut header, "main.tex", &contents[..])
        .unwrap();
    let encoder = tar.into_inner().unwrap();
    encoder.finish().unwrap()
}

#[tokio::test]
async fn search_json_includes_expected_fields() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/query"))
        .and(query_param("search_query", "all:transformer"))
        .respond_with(ResponseTemplate::new(200).set_body_string(feed_xml("2501.00001v1")))
        .mount(&server)
        .await;

    let temp = TempDir::new().unwrap();

    Command::cargo_bin("arxiv")
        .unwrap()
        .env("ARXIV_CONFIG_DIR", temp.path().join("config"))
        .env("ARXIV_DATA_DIR", temp.path().join("data"))
        .env("ARXIV_API_BASE_URL", format!("{}/api/query", server.uri()))
        .env("ARXIV_DOWNLOAD_BASE_URL", server.uri())
        .args(["search", "transformer", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": \"2501.00001v1\""))
        .stdout(predicate::str::contains("\"abstract_text\"").not());
}

#[tokio::test]
async fn download_updates_library_and_uses_registered_download_dir() {
    let server = MockServer::start().await;
    let id = "2501.00002v1";
    Mock::given(method("GET"))
        .and(path("/api/query"))
        .and(query_param("id_list", id))
        .respond_with(ResponseTemplate::new(200).set_body_string(feed_xml(id)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/pdf/{id}")))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header(
                    "content-disposition",
                    format!("attachment; filename=\"{id}.pdf\""),
                )
                .set_body_bytes(b"pdf-bytes".to_vec()),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/e-print/{id}")))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header(
                    "content-disposition",
                    format!("attachment; filename=\"arXiv-{id}.tar.gz\""),
                )
                .set_body_bytes(source_archive_bytes()),
        )
        .mount(&server)
        .await;

    let temp = TempDir::new().unwrap();
    let download_dir = temp.path().join("registered-downloads");
    let pdf = download_dir.join(id).join(format!("{id}.pdf"));

    Command::cargo_bin("arxiv")
        .unwrap()
        .env("ARXIV_CONFIG_DIR", temp.path().join("config"))
        .env("ARXIV_DATA_DIR", temp.path().join("data"))
        .env("ARXIV_API_BASE_URL", format!("{}/api/query", server.uri()))
        .env("ARXIV_DOWNLOAD_BASE_URL", server.uri())
        .args(["config", "set-download-dir", download_dir.to_str().unwrap()])
        .assert()
        .success();

    Command::cargo_bin("arxiv")
        .unwrap()
        .env("ARXIV_CONFIG_DIR", temp.path().join("config"))
        .env("ARXIV_DATA_DIR", temp.path().join("data"))
        .env("ARXIV_API_BASE_URL", format!("{}/api/query", server.uri()))
        .env("ARXIV_DOWNLOAD_BASE_URL", server.uri())
        .args(["download", id, "--format", "both", "--jobs", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("pdf: {}", pdf.display())));

    assert!(pdf.exists(), "expected PDF at {}", pdf.display());
    let source_dir = download_dir.join(id).join(format!("arXiv-{id}"));
    let source_file = source_dir.join("main.tex");
    assert!(
        source_file.exists(),
        "expected extracted source file at {}",
        source_file.display()
    );

    let output = Command::cargo_bin("arxiv")
        .unwrap()
        .env("ARXIV_CONFIG_DIR", temp.path().join("config"))
        .env("ARXIV_DATA_DIR", temp.path().join("data"))
        .env("ARXIV_API_BASE_URL", format!("{}/api/query", server.uri()))
        .env("ARXIV_DOWNLOAD_BASE_URL", server.uri())
        .arg("path")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let library_path = stdout
        .lines()
        .find_map(|line| line.strip_prefix("library_file="))
        .unwrap();
    let library = fs::read_to_string(library_path).unwrap();
    assert!(library.contains(id));
}

#[tokio::test]
async fn search_with_date_range_maps_to_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/query"))
        .and(query_param(
            "search_query",
            "all:diffusion AND submittedDate:[202401010000 TO 202401312359]",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_string(feed_xml("2501.00003v1")))
        .mount(&server)
        .await;

    let temp = TempDir::new().unwrap();
    Command::cargo_bin("arxiv")
        .unwrap()
        .env("ARXIV_CONFIG_DIR", temp.path().join("config"))
        .env("ARXIV_DATA_DIR", temp.path().join("data"))
        .env("ARXIV_API_BASE_URL", format!("{}/api/query", server.uri()))
        .env("ARXIV_DOWNLOAD_BASE_URL", server.uri())
        .args([
            "search",
            "diffusion",
            "--from",
            "2024-01-01",
            "--to",
            "2024-01-31",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": \"2501.00003v1\""));
}

#[tokio::test]
async fn search_multiple_keywords_maps_to_and_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/query"))
        .and(query_param(
            "search_query",
            "all:diffusion AND all:models AND cat:cs.CL",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_string(feed_xml("2501.00004v1")))
        .mount(&server)
        .await;

    let temp = TempDir::new().unwrap();
    Command::cargo_bin("arxiv")
        .unwrap()
        .env("ARXIV_CONFIG_DIR", temp.path().join("config"))
        .env("ARXIV_DATA_DIR", temp.path().join("data"))
        .env("ARXIV_API_BASE_URL", format!("{}/api/query", server.uri()))
        .env("ARXIV_DOWNLOAD_BASE_URL", server.uri())
        .args([
            "search",
            "diffusion models",
            "--category",
            "cs.CL",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": \"2501.00004v1\""));
}

#[tokio::test]
async fn search_works_with_only_title_filter() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/query"))
        .and(query_param("search_query", "ti:skill"))
        .respond_with(ResponseTemplate::new(200).set_body_string(feed_xml("2501.00005v1")))
        .mount(&server)
        .await;

    let temp = TempDir::new().unwrap();
    Command::cargo_bin("arxiv")
        .unwrap()
        .env("ARXIV_CONFIG_DIR", temp.path().join("config"))
        .env("ARXIV_DATA_DIR", temp.path().join("data"))
        .env("ARXIV_API_BASE_URL", format!("{}/api/query", server.uri()))
        .env("ARXIV_DOWNLOAD_BASE_URL", server.uri())
        .args(["search", "--title", "skill", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": \"2501.00005v1\""));
}

#[tokio::test]
async fn search_can_print_abstracts_in_table_output() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/query"))
        .and(query_param("search_query", "all:skill"))
        .respond_with(ResponseTemplate::new(200).set_body_string(feed_xml("2501.00006v1")))
        .mount(&server)
        .await;

    let temp = TempDir::new().unwrap();
    Command::cargo_bin("arxiv")
        .unwrap()
        .env("ARXIV_CONFIG_DIR", temp.path().join("config"))
        .env("ARXIV_DATA_DIR", temp.path().join("data"))
        .env("ARXIV_API_BASE_URL", format!("{}/api/query", server.uri()))
        .env("ARXIV_DOWNLOAD_BASE_URL", server.uri())
        .args(["search", "skill", "--include-abstract"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Summary for 2501.00006v1"));
}

#[tokio::test]
async fn search_json_can_include_abstracts_when_requested() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/query"))
        .and(query_param("search_query", "all:skill"))
        .respond_with(ResponseTemplate::new(200).set_body_string(feed_xml("2501.00009v1")))
        .mount(&server)
        .await;

    let temp = TempDir::new().unwrap();
    Command::cargo_bin("arxiv")
        .unwrap()
        .env("ARXIV_CONFIG_DIR", temp.path().join("config"))
        .env("ARXIV_DATA_DIR", temp.path().join("data"))
        .env("ARXIV_API_BASE_URL", format!("{}/api/query", server.uri()))
        .env("ARXIV_DOWNLOAD_BASE_URL", server.uri())
        .args(["search", "skill", "--json", "--include-abstract"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"abstract_text\": \"Summary for 2501.00009v1\"",
        ));
}

#[tokio::test]
async fn latest_lists_category_by_submitted_date_desc() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/query"))
        .and(query_param("search_query", "cat:cs.CL"))
        .and(query_param("sortBy", "submittedDate"))
        .and(query_param("sortOrder", "descending"))
        .and(query_param("max_results", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_string(feed_xml("2501.00007v1")))
        .mount(&server)
        .await;

    let temp = TempDir::new().unwrap();
    Command::cargo_bin("arxiv")
        .unwrap()
        .env("ARXIV_CONFIG_DIR", temp.path().join("config"))
        .env("ARXIV_DATA_DIR", temp.path().join("data"))
        .env("ARXIV_API_BASE_URL", format!("{}/api/query", server.uri()))
        .env("ARXIV_DOWNLOAD_BASE_URL", server.uri())
        .args(["latest", "cs.CL", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": \"2501.00007v1\""));
}

#[tokio::test]
async fn latest_supports_date_range() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/query"))
        .and(query_param(
            "search_query",
            "cat:cs.CL AND submittedDate:[202501010000 TO 202501312359]",
        ))
        .and(query_param("sortBy", "submittedDate"))
        .and(query_param("sortOrder", "descending"))
        .respond_with(ResponseTemplate::new(200).set_body_string(feed_xml("2501.00008v1")))
        .mount(&server)
        .await;

    let temp = TempDir::new().unwrap();
    Command::cargo_bin("arxiv")
        .unwrap()
        .env("ARXIV_CONFIG_DIR", temp.path().join("config"))
        .env("ARXIV_DATA_DIR", temp.path().join("data"))
        .env("ARXIV_API_BASE_URL", format!("{}/api/query", server.uri()))
        .env("ARXIV_DOWNLOAD_BASE_URL", server.uri())
        .args([
            "latest",
            "cs.CL",
            "--from",
            "2025-01-01",
            "--to",
            "2025-01-31",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": \"2501.00008v1\""));
}
