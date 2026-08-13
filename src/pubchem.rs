use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

use crate::types::CompoundInfo;

#[derive(Clone)]
pub struct PubChem {
    client: Client,
}

impl PubChem {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    pub async fn search_compound(&self, name: &str) -> Result<Option<CompoundInfo>> {
        let url = format!(
            "https://pubchem.ncbi.nlm.nih.gov/rest/pug/compound/name/{}/JSON",
            urlencoding(name)
        );
        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let compound = resp["PC_Compounds"].as_array().and_then(|a| a.first());
        match compound {
            Some(c) => {
                let cid = c["id"]["id"]["cid"].as_u64().unwrap_or(0);
                Ok(Some(self.get_properties(cid).await?.unwrap_or(CompoundInfo {
                    cid, iupac_name: None, molecular_formula: None,
                    molecular_weight: None, canonical_smiles: None,
                })))
            }
            None => Ok(None),
        }
    }

    pub async fn get_properties(&self, cid: u64) -> Result<Option<CompoundInfo>> {
        let url = format!(
            "https://pubchem.ncbi.nlm.nih.gov/rest/pug/compound/cid/{}/property/MolecularFormula,MolecularWeight,IUPACName,CanonicalSMILES/JSON",
            cid
        );
        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let props = resp["PropertyTable"]["Properties"].as_array().and_then(|a| a.first());
        match props {
            Some(p) => Ok(Some(CompoundInfo {
                cid,
                iupac_name: p["IUPACName"].as_str().map(String::from),
                molecular_formula: p["MolecularFormula"].as_str().map(String::from),
                molecular_weight: p["MolecularWeight"].as_f64(),
                canonical_smiles: p["CanonicalSMILES"].as_str().map(String::from),
            })),
            None => Ok(None),
        }
    }
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "%20")
}
