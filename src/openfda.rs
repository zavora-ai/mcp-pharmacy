use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

use crate::types::{AdverseEvent, DrugLabel, DrugProduct, DrugRecall};

#[derive(Clone)]
pub struct OpenFda {
    client: Client,
    api_key: Option<String>,
}

impl OpenFda {
    pub fn new(api_key: Option<String>) -> Self {
        Self { client: Client::new(), api_key }
    }

    fn url(&self, endpoint: &str, query: &str, limit: u32) -> String {
        let key_param = self.api_key.as_ref().map(|k| format!("&api_key={k}")).unwrap_or_default();
        format!("https://api.fda.gov/drug/{endpoint}.json?search={query}&limit={limit}{key_param}")
    }

    pub async fn search_labels(&self, query: &str, limit: u32) -> Result<Vec<DrugLabel>> {
        let search = format!("openfda.generic_name:{query}+openfda.brand_name:{query}");
        let url = self.url("label", &search, limit);
        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let results = resp["results"].as_array().unwrap_or(&vec![]).clone();
        Ok(results.iter().map(|r| {
            let openfda = &r["openfda"];
            DrugLabel {
                source: "openfda".into(),
                brand_name: arr_first(openfda, "brand_name"),
                generic_name: arr_first(openfda, "generic_name"),
                manufacturer: arr_first(openfda, "manufacturer_name"),
                product_ndc: arr_first(openfda, "product_ndc"),
                route: arr_first(openfda, "route"),
                substance_name: arr_first(openfda, "substance_name"),
                indications: arr_first(r, "indications_and_usage"),
                warnings: arr_first(r, "warnings"),
                dosage: arr_first(r, "dosage_and_administration"),
            }
        }).collect())
    }

    pub async fn get_adverse_events(&self, drug: &str, limit: u32) -> Result<Vec<AdverseEvent>> {
        let search = format!("patient.drug.openfda.generic_name:{drug}");
        let url = self.url("event", &search, limit);
        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let results = resp["results"].as_array().unwrap_or(&vec![]).clone();
        Ok(results.iter().map(|r| {
            let reactions = r["patient"]["reaction"].as_array()
                .and_then(|a| a.first())
                .and_then(|rx| rx["reactionmeddrapt"].as_str())
                .map(String::from);
            AdverseEvent {
                source: "openfda_faers".into(),
                drug_name: drug.to_string(),
                reaction: reactions,
                outcome: r["patient"]["reaction"].as_array()
                    .and_then(|a| a.first())
                    .and_then(|rx| rx["reactionoutcome"].as_str())
                    .map(String::from),
                serious: r["serious"].as_str().map(|s| s == "1"),
                receive_date: r["receivedate"].as_str().map(String::from),
                country: r["occurcountry"].as_str().map(String::from),
            }
        }).collect())
    }

    pub async fn search_recalls(&self, query: &str, limit: u32) -> Result<Vec<DrugRecall>> {
        let search = format!("openfda.generic_name:{query}+reason_for_recall:{query}");
        let url = self.url("enforcement", &search, limit);
        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let results = resp["results"].as_array().unwrap_or(&vec![]).clone();
        Ok(results.iter().map(|r| DrugRecall {
            source: "openfda".into(),
            product_description: r["product_description"].as_str().map(String::from),
            reason: r["reason_for_recall"].as_str().map(String::from),
            classification: r["classification"].as_str().map(String::from),
            status: r["status"].as_str().map(String::from),
            recall_initiation_date: r["recall_initiation_date"].as_str().map(String::from),
            distribution: r["distribution_pattern"].as_str().map(String::from),
            recalling_firm: r["recalling_firm"].as_str().map(String::from),
        }).collect())
    }

    pub async fn get_ndc(&self, query: &str, limit: u32) -> Result<Vec<DrugProduct>> {
        let search = format!("generic_name:{query}+brand_name:{query}");
        let url = self.url("ndc", &search, limit);
        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let results = resp["results"].as_array().unwrap_or(&vec![]).clone();
        Ok(results.iter().map(|r| DrugProduct {
            source: "openfda_ndc".into(),
            region: "US".into(),
            brand_name: r["brand_name"].as_str().map(String::from),
            generic_name: r["generic_name"].as_str().map(String::from),
            active_ingredient: r["active_ingredients"].as_array()
                .and_then(|a| a.first())
                .and_then(|i| i["name"].as_str())
                .map(String::from),
            manufacturer: r["labeler_name"].as_str().map(String::from),
            dosage_form: r["dosage_form"].as_str().map(String::from),
            route: r["route"].as_array()
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                .map(String::from),
            status: r["marketing_category"].as_str().map(String::from),
            product_id: r["product_ndc"].as_str().map(String::from),
        }).collect())
    }
}

fn arr_first(v: &Value, key: &str) -> Option<String> {
    v[key].as_array()
        .and_then(|a| a.first())
        .and_then(|s| s.as_str())
        .map(String::from)
        .or_else(|| v[key].as_str().map(String::from))
}
