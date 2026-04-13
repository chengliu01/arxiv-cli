#[tokio::main]
async fn main() {
    if let Err(err) = arxiv::run().await {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}
