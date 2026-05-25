use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugLabel {
    pub source: String,
    pub brand_name: Option<String>,
    pub generic_name: Option<String>,
    pub manufacturer: Option<String>,
    pub product_ndc: Option<String>,
    pub route: Option<String>,
    pub substance_name: Option<String>,
    pub indications: Option<String>,
    pub warnings: Option<String>,
    pub dosage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseEvent {
    pub source: String,
    pub drug_name: String,
    pub reaction: Option<String>,
    pub outcome: Option<String>,
    pub serious: Option<bool>,
    pub receive_date: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugRecall {
    pub source: String,
    pub product_description: Option<String>,
    pub reason: Option<String>,
    pub classification: Option<String>,
    pub status: Option<String>,
    pub recall_initiation_date: Option<String>,
    pub distribution: Option<String>,
    pub recalling_firm: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugProduct {
    pub source: String,
    pub region: String,
    pub brand_name: Option<String>,
    pub generic_name: Option<String>,
    pub active_ingredient: Option<String>,
    pub manufacturer: Option<String>,
    pub dosage_form: Option<String>,
    pub route: Option<String>,
    pub status: Option<String>,
    pub product_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtcClass {
    pub code: String,
    pub name: String,
    pub relation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundInfo {
    pub cid: u64,
    pub iupac_name: Option<String>,
    pub molecular_formula: Option<String>,
    pub molecular_weight: Option<f64>,
    pub canonical_smiles: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalTrial {
    pub nct_id: String,
    pub title: Option<String>,
    pub status: Option<String>,
    pub phase: Option<String>,
    pub conditions: Vec<String>,
    pub interventions: Vec<String>,
    pub start_date: Option<String>,
    pub completion_date: Option<String>,
    pub enrollment: Option<u32>,
    pub sponsor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyUpdate {
    pub source: String,
    pub title: String,
    pub url: Option<String>,
    pub date: Option<String>,
    pub description: Option<String>,
}
