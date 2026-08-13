use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

use crate::types::DrugProduct;

#[derive(Clone)]
pub struct HealthCanada {
    client: Client,
}

impl HealthCanada {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    pub async fn search_products(&self, query: &str, by_ingredient: bool) -> Result<Vec<DrugProduct>> {
        let param = if by_ingredient { "ingredient" } else { "brandname" };
        let url = format!(
            "https://health-products.canada.ca/api/drug/drugproduct/?{}={}",
            param, urlencoding(query)
        );
        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let products = match &resp {
            Value::Array(arr) => arr.clone(),
            _ => vec![],
        };
        Ok(products.iter().take(20).map(|p| DrugProduct {
            source: "health_canada_dpd".into(),
            region: "CA".into(),
            brand_name: p["brand_name"].as_str().map(String::from),
            generic_name: None,
            active_ingredient: None,
            manufacturer: p["company_name"].as_str().map(String::from),
            dosage_form: p["descriptor"].as_str().map(String::from),
            route: None,
            status: p["class_name"].as_str().map(String::from),
            product_id: p["drug_identification_number"].as_str().map(String::from),
        }).collect())
    }

    pub async fn get_product(&self, din: &str) -> Result<Option<DrugProduct>> {
        let url = format!(
            "https://health-products.canada.ca/api/drug/drugproduct/?din={}",
            din
        );
        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let products = match &resp {
            Value::Array(arr) => arr.clone(),
            _ => vec![],
        };
        Ok(products.first().map(|p| DrugProduct {
            source: "health_canada_dpd".into(),
            region: "CA".into(),
            brand_name: p["brand_name"].as_str().map(String::from),
            generic_name: None,
            active_ingredient: None,
            manufacturer: p["company_name"].as_str().map(String::from),
            dosage_form: p["descriptor"].as_str().map(String::from),
            route: None,
            status: p["class_name"].as_str().map(String::from),
            product_id: p["drug_identification_number"].as_str().map(String::from),
        }))
    }
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "%20")
}
