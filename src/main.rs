mod types;
mod openfda;
mod dailymed;
mod rxnorm;
mod pubchem;
mod health_canada;
mod clinicaltrials;
mod ema;
mod server;

use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Validate manifest
    let manifest = adk_mcp_sdk::ServerManifest::from_file(std::path::Path::new("mcp-server.toml"))?;
    let errors = manifest.validate();
    if !errors.is_empty() {
        eprintln!("Manifest validation warnings:");
        for e in &errors {
            eprintln!("  - {e}");
        }
    }

    // Init backends
    let api_key = std::env::var("OPENFDA_API_KEY").ok();
    let openfda = openfda::OpenFda::new(api_key);
    let dailymed = dailymed::DailyMed::new();
    let rxnorm = rxnorm::RxNorm::new();
    let pubchem = pubchem::PubChem::new();
    let health_canada = health_canada::HealthCanada::new();
    let clinicaltrials = clinicaltrials::ClinicalTrialsGov::new();

    // EMA: download and cache XLSX data (skip if EMA_SKIP is set)
    let ema = if std::env::var("EMA_SKIP").is_ok() {
        eprintln!("EMA: skipped (EMA_SKIP set)");
        ema::Ema::from_download().await
    } else {
        eprintln!("Loading EMA medicine data...");
        let e = ema::Ema::from_download().await;
        eprintln!("EMA: {} medicines loaded", e.count());
        e
    };

    let server = server::PharmacyServer {
        openfda,
        dailymed,
        rxnorm,
        pubchem,
        health_canada,
        clinicaltrials,
        ema,
    };

    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
