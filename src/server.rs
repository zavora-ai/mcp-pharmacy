use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};

use crate::dailymed::DailyMed;
use crate::openfda::OpenFda;
use crate::pubchem::PubChem;
use crate::rxnorm::RxNorm;
use crate::health_canada::HealthCanada;
use crate::clinicaltrials::ClinicalTrialsGov;
use crate::ema::Ema;
use crate::types::SafetyUpdate;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DrugQuery {
    /// Drug name (generic or brand)
    pub query: String,
    /// Max results (default 5)
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SetidInput {
    /// DailyMed SPL setid
    pub setid: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RxcuiInput {
    /// RxNorm RxCUI identifier
    pub rxcui: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CidInput {
    /// PubChem Compound ID
    pub cid: u64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DinInput {
    /// Canadian Drug Identification Number
    pub din: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NctInput {
    /// ClinicalTrials.gov NCT ID (e.g. NCT03756623)
    pub nct_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CanadaSearchInput {
    /// Drug name to search
    pub query: String,
    /// Search by ingredient instead of brand name
    pub by_ingredient: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MhraQuery {
    /// Search term for MHRA drug safety updates
    pub query: String,
    /// Max results (default 5)
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GlobalSearchInput {
    /// Drug name to search across all regions
    pub query: String,
}

pub struct PharmacyServer {
    pub openfda: OpenFda,
    pub dailymed: DailyMed,
    pub rxnorm: RxNorm,
    pub pubchem: PubChem,
    pub health_canada: HealthCanada,
    pub clinicaltrials: ClinicalTrialsGov,
    pub ema: Ema,
}

#[tool_router(server_handler)]
impl PharmacyServer {
    // --- OpenFDA ---

    #[tool(description = "Search US drug labels (package inserts) by generic name, brand name, or indication")]
    async fn openfda_search_labels(&self, Parameters(input): Parameters<DrugQuery>) -> String {
        let limit = input.limit.unwrap_or(5);
        match self.openfda.search_labels(&input.query, limit).await {
            Ok(labels) => serde_json::to_string_pretty(&labels).unwrap_or_default(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Search FDA Adverse Event Reporting System (FAERS) for a drug")]
    async fn openfda_get_adverse_events(&self, Parameters(input): Parameters<DrugQuery>) -> String {
        let limit = input.limit.unwrap_or(5);
        match self.openfda.get_adverse_events(&input.query, limit).await {
            Ok(events) => serde_json::to_string_pretty(&events).unwrap_or_default(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Search FDA drug recalls and enforcement actions")]
    async fn openfda_search_recalls(&self, Parameters(input): Parameters<DrugQuery>) -> String {
        let limit = input.limit.unwrap_or(5);
        match self.openfda.search_recalls(&input.query, limit).await {
            Ok(recalls) => serde_json::to_string_pretty(&recalls).unwrap_or_default(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Look up National Drug Code (NDC) directory entries for a drug")]
    async fn openfda_get_ndc(&self, Parameters(input): Parameters<DrugQuery>) -> String {
        let limit = input.limit.unwrap_or(5);
        match self.openfda.get_ndc(&input.query, limit).await {
            Ok(products) => serde_json::to_string_pretty(&products).unwrap_or_default(),
            Err(e) => format!("Error: {e}"),
        }
    }

    // --- DailyMed ---

    #[tool(description = "Search DailyMed structured product labels (SPL) by drug name")]
    async fn dailymed_search_labels(&self, Parameters(input): Parameters<DrugQuery>) -> String {
        let limit = input.limit.unwrap_or(5);
        match self.dailymed.search_labels(&input.query, limit).await {
            Ok(spls) => serde_json::to_string_pretty(&spls).unwrap_or_default(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Retrieve full SPL XML label by setid for detailed section extraction")]
    async fn dailymed_get_label_xml(&self, Parameters(input): Parameters<SetidInput>) -> String {
        match self.dailymed.get_label_xml(&input.setid).await {
            Ok(xml) => {
                // Return first 2000 chars to avoid overwhelming context
                if xml.len() > 2000 {
                    format!("{}...\n[truncated, {} total bytes]", &xml[..2000], xml.len())
                } else {
                    xml
                }
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    // --- RxNorm ---

    #[tool(description = "Normalize a drug name to RxNorm RxCUI identifier")]
    async fn rxnorm_normalize(&self, Parameters(input): Parameters<DrugQuery>) -> String {
        match self.rxnorm.normalize(&input.query).await {
            Ok(Some(drug)) => serde_json::to_string_pretty(&drug).unwrap_or_default(),
            Ok(None) => format!("No RxCUI found for '{}'", input.query),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get drug properties (name, term type, status) by RxCUI")]
    async fn rxnorm_get_properties(&self, Parameters(input): Parameters<RxcuiInput>) -> String {
        match self.rxnorm.get_properties(&input.rxcui).await {
            Ok(Some(drug)) => serde_json::to_string_pretty(&drug).unwrap_or_default(),
            Ok(None) => format!("No properties found for RxCUI {}", input.rxcui),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get WHO ATC classification codes for a drug via RxCUI")]
    async fn rxnorm_get_atc_classes(&self, Parameters(input): Parameters<RxcuiInput>) -> String {
        match self.rxnorm.get_atc_classes(&input.rxcui).await {
            Ok(classes) => serde_json::to_string_pretty(&classes).unwrap_or_default(),
            Err(e) => format!("Error: {e}"),
        }
    }

    // --- PubChem ---

    #[tool(description = "Search PubChem for a chemical compound by name, get CID and structure")]
    async fn pubchem_search_compound(&self, Parameters(input): Parameters<DrugQuery>) -> String {
        match self.pubchem.search_compound(&input.query).await {
            Ok(Some(compound)) => serde_json::to_string_pretty(&compound).unwrap_or_default(),
            Ok(None) => format!("No compound found for '{}'", input.query),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get molecular properties (formula, weight, IUPAC name) for a compound by CID")]
    async fn pubchem_get_properties(&self, Parameters(input): Parameters<CidInput>) -> String {
        match self.pubchem.get_properties(input.cid).await {
            Ok(Some(compound)) => serde_json::to_string_pretty(&compound).unwrap_or_default(),
            Ok(None) => format!("No properties found for CID {}", input.cid),
            Err(e) => format!("Error: {e}"),
        }
    }

    // --- Health Canada ---

    #[tool(description = "Search Health Canada Drug Product Database by brand name or active ingredient")]
    async fn health_canada_search_products(&self, Parameters(input): Parameters<CanadaSearchInput>) -> String {
        let by_ingredient = input.by_ingredient.unwrap_or(false);
        match self.health_canada.search_products(&input.query, by_ingredient).await {
            Ok(products) => serde_json::to_string_pretty(&products).unwrap_or_default(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get detailed Canadian drug product information by DIN")]
    async fn health_canada_get_product(&self, Parameters(input): Parameters<DinInput>) -> String {
        match self.health_canada.get_product(&input.din).await {
            Ok(Some(product)) => serde_json::to_string_pretty(&product).unwrap_or_default(),
            Ok(None) => format!("No product found for DIN {}", input.din),
            Err(e) => format!("Error: {e}"),
        }
    }

    // --- ClinicalTrials.gov ---

    #[tool(description = "Search ClinicalTrials.gov for studies by drug, condition, or keyword")]
    async fn clinicaltrials_search(&self, Parameters(input): Parameters<DrugQuery>) -> String {
        let limit = input.limit.unwrap_or(5);
        match self.clinicaltrials.search(&input.query, limit).await {
            Ok(trials) => serde_json::to_string_pretty(&trials).unwrap_or_default(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get detailed study information by NCT ID")]
    async fn clinicaltrials_get_study(&self, Parameters(input): Parameters<NctInput>) -> String {
        match self.clinicaltrials.get_study(&input.nct_id).await {
            Ok(Some(trial)) => serde_json::to_string_pretty(&trial).unwrap_or_default(),
            Ok(None) => format!("No study found for {}", input.nct_id),
            Err(e) => format!("Error: {e}"),
        }
    }

    // --- EMA ---

    #[tool(description = "Search EU-authorized medicines from EMA data (name, ATC, therapeutic area, status)")]
    async fn ema_search_medicines(&self, Parameters(input): Parameters<DrugQuery>) -> String {
        let limit = input.limit.unwrap_or(10) as usize;
        let results = self.ema.search(&input.query, limit);
        if results.is_empty() {
            format!("No EMA medicines found for '{}' (cache has {} medicines)", input.query, self.ema.count())
        } else {
            serde_json::to_string_pretty(&results).unwrap_or_default()
        }
    }

    // --- MHRA ---

    #[tool(description = "Search UK MHRA Drug Safety Updates")]
    async fn mhra_search_safety_updates(&self, Parameters(input): Parameters<MhraQuery>) -> String {
        let limit = input.limit.unwrap_or(5);
        match search_mhra(&input.query, limit).await {
            Ok(updates) => serde_json::to_string_pretty(&updates).unwrap_or_default(),
            Err(e) => format!("Error: {e}"),
        }
    }

    // --- Cross-backend ---

    #[tool(description = "Federated drug search across multiple registries (US, EU, UK, Canada) by name")]
    async fn search_drug_global(&self, Parameters(input): Parameters<GlobalSearchInput>) -> String {
        let mut results = Vec::new();

        // US via OpenFDA NDC
        if let Ok(us) = self.openfda.get_ndc(&input.query, 3).await {
            results.extend(us);
        }
        // Canada
        if let Ok(ca) = self.health_canada.search_products(&input.query, false).await {
            results.extend(ca.into_iter().take(3));
        }
        // EU via EMA
        let eu = self.ema.search(&input.query, 3);
        results.extend(eu);

        if results.is_empty() {
            format!("No products found for '{}' across US, CA, EU", input.query)
        } else {
            serde_json::to_string_pretty(&results).unwrap_or_default()
        }
    }

    #[tool(description = "Check registration/approval status of a drug across available regions")]
    async fn get_registration_status(&self, Parameters(input): Parameters<GlobalSearchInput>) -> String {
        let mut status = serde_json::Map::new();

        // US
        let us_count = self.openfda.get_ndc(&input.query, 1).await
            .map(|v| v.len()).unwrap_or(0);
        status.insert("US".into(), serde_json::json!({
            "registered": us_count > 0,
            "source": "OpenFDA NDC"
        }));

        // Canada
        let ca_count = self.health_canada.search_products(&input.query, false).await
            .map(|v| v.len()).unwrap_or(0);
        status.insert("CA".into(), serde_json::json!({
            "registered": ca_count > 0,
            "source": "Health Canada DPD"
        }));

        // EU
        let eu_count = self.ema.search(&input.query, 1).len();
        status.insert("EU".into(), serde_json::json!({
            "registered": eu_count > 0,
            "source": "EMA"
        }));

        serde_json::to_string_pretty(&serde_json::json!({
            "drug": input.query,
            "regions": status
        })).unwrap_or_default()
    }
}

async fn search_mhra(query: &str, limit: u32) -> anyhow::Result<Vec<SafetyUpdate>> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://www.gov.uk/api/search.json?filter_document_type=drug_safety_update&q={}&count={}",
        query.replace(' ', "+"), limit
    );
    let resp: serde_json::Value = client.get(&url).send().await?.json().await?;
    let results = resp["results"].as_array().unwrap_or(&vec![]).clone();
    Ok(results.iter().map(|r| SafetyUpdate {
        source: "mhra".into(),
        title: r["title"].as_str().unwrap_or_default().to_string(),
        url: r["link"].as_str().map(|l| format!("https://www.gov.uk{l}")),
        date: r["public_timestamp"].as_str().map(String::from),
        description: r["description"].as_str().map(String::from),
    }).collect())
}
