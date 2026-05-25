use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

use crate::types::AtcClass;

pub struct RxNorm {
    client: Client,
}

impl RxNorm {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    pub async fn normalize(&self, name: &str) -> Result<Option<RxNormDrug>> {
        let url = format!("https://rxnav.nlm.nih.gov/REST/rxcui.json?name={}", urlencoding(name));
        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let ids = resp["idGroup"]["rxnormId"].as_array();
        match ids.and_then(|a| a.first()) {
            Some(id) => {
                let rxcui = id.as_str().unwrap_or_default().to_string();
                Ok(Some(RxNormDrug { rxcui, name: name.to_string(), term_type: None }))
            }
            None => Ok(None),
        }
    }

    pub async fn get_properties(&self, rxcui: &str) -> Result<Option<RxNormDrug>> {
        let url = format!("https://rxnav.nlm.nih.gov/REST/rxcui/{}/properties.json", rxcui);
        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let props = &resp["properties"];
        if props.is_null() {
            return Ok(None);
        }
        Ok(Some(RxNormDrug {
            rxcui: props["rxcui"].as_str().unwrap_or_default().to_string(),
            name: props["name"].as_str().unwrap_or_default().to_string(),
            term_type: props["tty"].as_str().map(String::from),
        }))
    }

    pub async fn get_atc_classes(&self, rxcui: &str) -> Result<Vec<AtcClass>> {
        let url = format!(
            "https://rxnav.nlm.nih.gov/REST/rxclass/class/byRxcui.json?rxcui={}&relaSource=ATC",
            rxcui
        );
        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let list = resp["rxclassDrugInfoList"]["rxclassDrugInfo"]
            .as_array().unwrap_or(&vec![]).clone();
        let mut seen = std::collections::HashSet::new();
        Ok(list.iter().filter_map(|item| {
            let concept = &item["rxclassMinConceptItem"];
            let code = concept["classId"].as_str()?.to_string();
            if !seen.insert(code.clone()) { return None; }
            Some(AtcClass {
                code,
                name: concept["className"].as_str().unwrap_or_default().to_string(),
                relation: item["rela"].as_str().map(String::from),
            })
        }).collect())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RxNormDrug {
    pub rxcui: String,
    pub name: String,
    pub term_type: Option<String>,
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "+")
}
