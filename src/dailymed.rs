use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

pub struct DailyMed {
    client: Client,
}

impl DailyMed {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    pub async fn search_labels(&self, drug_name: &str, limit: u32) -> Result<Vec<DailyMedSpl>> {
        let url = format!(
            "https://dailymed.nlm.nih.gov/dailymed/services/v2/spls.json?drug_name={}&pagesize={}",
            urlencoding(drug_name), limit
        );
        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let data = resp["data"].as_array().unwrap_or(&vec![]).clone();
        Ok(data.iter().map(|r| DailyMedSpl {
            setid: r["setid"].as_str().unwrap_or_default().to_string(),
            title: r["title"].as_str().unwrap_or_default().to_string(),
            published_date: r["published_date"].as_str().map(String::from),
            spl_version: r["spl_version"].as_u64().map(|v| v as u32),
        }).collect())
    }

    pub async fn get_label_xml(&self, setid: &str) -> Result<String> {
        let url = format!(
            "https://dailymed.nlm.nih.gov/dailymed/services/v2/spls/{}.xml",
            setid
        );
        let text = self.client.get(&url).send().await?.text().await?;
        Ok(text)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DailyMedSpl {
    pub setid: String,
    pub title: String,
    pub published_date: Option<String>,
    pub spl_version: Option<u32>,
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "+")
}
