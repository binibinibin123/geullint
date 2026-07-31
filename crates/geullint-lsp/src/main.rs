#[tokio::main]
async fn main() {
    geullint_lsp::run_stdio().await;
}
