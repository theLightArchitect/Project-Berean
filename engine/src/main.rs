use berean_engine::engine::BereanEngine;
use rmcp::transport::stdio;
use rmcp::ServiceExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = BereanEngine::default().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
