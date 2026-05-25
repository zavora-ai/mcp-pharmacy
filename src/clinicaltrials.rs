use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

use crate::types::ClinicalTrial;

pub struct ClinicalTrialsGov {
    client: Client,
}

impl ClinicalTrialsGov {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<ClinicalTrial>> {
        let url = format!(
            "https://clinicaltrials.gov/api/v2/studies?query.term={}&pageSize={}",
            urlencoding(query), limit
        );
        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let studies = resp["studies"].as_array().unwrap_or(&vec![]).clone();
        Ok(studies.iter().map(|s| parse_study(s)).collect())
    }

    pub async fn get_study(&self, nct_id: &str) -> Result<Option<ClinicalTrial>> {
        let url = format!("https://clinicaltrials.gov/api/v2/studies/{}", nct_id);
        let resp: Value = self.client.get(&url).send().await?.json().await?;
        if resp["protocolSection"].is_null() {
            return Ok(None);
        }
        Ok(Some(parse_study(&resp)))
    }
}

fn parse_study(s: &Value) -> ClinicalTrial {
    let proto = &s["protocolSection"];
    let ident = &proto["identificationModule"];
    let status_mod = &proto["statusModule"];
    let design = &proto["designModule"];
    let sponsor_mod = &proto["sponsorCollaboratorsModule"];
    let conditions_mod = &proto["conditionsModule"];
    let arms_mod = &proto["armsInterventionsModule"];

    ClinicalTrial {
        nct_id: ident["nctId"].as_str().unwrap_or_default().to_string(),
        title: ident["briefTitle"].as_str().map(String::from),
        status: status_mod["overallStatus"].as_str().map(String::from),
        phase: design.get("phases").and_then(|p| p.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .map(String::from),
        conditions: conditions_mod["conditions"].as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        interventions: arms_mod.get("interventions").and_then(|i| i.as_array())
            .map(|a| a.iter().filter_map(|v| v["name"].as_str().map(String::from)).collect())
            .unwrap_or_default(),
        start_date: status_mod.get("startDateStruct")
            .and_then(|d| d["date"].as_str()).map(String::from),
        completion_date: status_mod.get("completionDateStruct")
            .and_then(|d| d["date"].as_str()).map(String::from),
        enrollment: design.get("enrollmentInfo")
            .and_then(|e| e["count"].as_u64()).map(|n| n as u32),
        sponsor: sponsor_mod.get("leadSponsor")
            .and_then(|s| s["name"].as_str()).map(String::from),
    }
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "+")
}
