use anyhow::Result;

use crate::types::DrugProduct;

/// EMA backend uses a pre-cached JSON file derived from the EMA XLSX download.
/// On first use, it downloads and parses the XLSX into a local JSON cache.
/// For the MCP server, we use a simplified approach: search the cached data.
pub struct Ema {
    medicines: Vec<EmaMedicine>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmaMedicine {
    pub name: String,
    pub inn: Option<String>,
    pub active_substance: Option<String>,
    pub atc_code: Option<String>,
    pub therapeutic_area: Option<String>,
    pub status: Option<String>,
    pub ema_number: Option<String>,
}

impl Ema {
    /// Create from a pre-loaded JSON array of medicines.
    /// In production, this would be loaded from a cached XLSX parse.
    #[allow(dead_code)]
    pub fn new(medicines: Vec<EmaMedicine>) -> Self {
        Self { medicines }
    }

    /// Load from the EMA XLSX download URL. Returns empty if download fails.
    pub async fn from_download() -> Self {
        match Self::download_and_parse().await {
            Ok(medicines) => Self { medicines },
            Err(_) => Self { medicines: vec![] },
        }
    }

    async fn download_and_parse() -> Result<Vec<EmaMedicine>> {
        let client = reqwest::Client::new();
        let url = "https://www.ema.europa.eu/en/documents/report/medicines-output-medicines-report_en.xlsx";
        let bytes = client.get(url).send().await?.bytes().await?;
        // Parse XLSX using zip + basic XML extraction
        parse_xlsx_medicines(&bytes)
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<DrugProduct> {
        let q = query.to_lowercase();
        self.medicines.iter()
            .filter(|m| {
                m.name.to_lowercase().contains(&q)
                    || m.inn.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || m.active_substance.as_deref().unwrap_or("").to_lowercase().contains(&q)
            })
            .take(limit)
            .map(|m| DrugProduct {
                source: "ema".into(),
                region: "EU".into(),
                brand_name: Some(m.name.clone()),
                generic_name: m.inn.clone(),
                active_ingredient: m.active_substance.clone(),
                manufacturer: None,
                dosage_form: None,
                route: None,
                status: m.status.clone(),
                product_id: m.ema_number.clone(),
            })
            .collect()
    }

    pub fn count(&self) -> usize {
        self.medicines.len()
    }
}

/// Minimal XLSX parser — extracts shared strings and sheet data.
/// EMA XLSX has a single sheet with headers in row 3.
fn parse_xlsx_medicines(bytes: &[u8]) -> Result<Vec<EmaMedicine>> {
    use std::io::Read;
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;

    // Read shared strings
    let strings = {
        let mut file = archive.by_name("xl/sharedStrings.xml")?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        parse_shared_strings(&content)
    };

    // Read sheet1
    let rows = {
        let mut file = archive.by_name("xl/worksheets/sheet1.xml")?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        parse_sheet_rows(&content, &strings)
    };

    // Find header row (contains "Name of medicine")
    let header_idx = rows.iter().position(|r| r.iter().any(|c| c.contains("Name of medicine")));
    let header_idx = match header_idx {
        Some(i) => i,
        None => return Ok(vec![]),
    };

    let headers = &rows[header_idx];
    let col = |name: &str| headers.iter().position(|h| h.contains(name));

    let name_col = col("Name of medicine").unwrap_or(0);
    let inn_col = col("International non-proprietary");
    let active_col = col("Active substance");
    let atc_col = col("ATC code");
    let therapeutic_col = col("Therapeutic area");
    let status_col = col("Medicine status");
    let ema_col = col("EMA product number");

    let mut medicines = Vec::new();
    for row in &rows[header_idx + 1..] {
        let get = |idx: Option<usize>| idx.and_then(|i| row.get(i)).cloned().filter(|s| !s.is_empty());
        let name = row.get(name_col).cloned().unwrap_or_default();
        if name.is_empty() { continue; }
        medicines.push(EmaMedicine {
            name,
            inn: get(inn_col),
            active_substance: get(active_col),
            atc_code: get(atc_col),
            therapeutic_area: get(therapeutic_col),
            status: get(status_col),
            ema_number: get(ema_col),
        });
    }
    Ok(medicines)
}

fn parse_shared_strings(xml: &str) -> Vec<String> {
    // Simple regex-free extraction of <t>...</t> values
    let mut strings = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<t") {
        let after_tag = &rest[start..];
        let content_start = match after_tag.find('>') {
            Some(i) => start + i + 1,
            None => break,
        };
        let content_end = match rest[content_start..].find("</t>") {
            Some(i) => content_start + i,
            None => break,
        };
        strings.push(rest[content_start..content_end].to_string());
        rest = &rest[content_end + 4..];
    }
    strings
}

fn parse_sheet_rows(xml: &str, strings: &[String]) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut rest = xml;
    while let Some(row_start) = rest.find("<row") {
        let row_end = match rest[row_start..].find("</row>") {
            Some(i) => row_start + i + 6,
            None => break,
        };
        let row_xml = &rest[row_start..row_end];
        let mut cells: Vec<String> = Vec::new();
        let mut cell_rest = row_xml;
        while let Some(c_start) = cell_rest.find("<c ") {
            let c_end = match cell_rest[c_start..].find("</c>") {
                Some(i) => c_start + i + 4,
                None => match cell_rest[c_start..].find("/>") {
                    Some(i) => c_start + i + 2,
                    None => break,
                },
            };
            let cell_xml = &cell_rest[c_start..c_end];
            let is_shared = cell_xml.contains("t=\"s\"");
            let value = extract_value(cell_xml);
            let resolved = if is_shared {
                value.parse::<usize>().ok()
                    .and_then(|i| strings.get(i))
                    .cloned()
                    .unwrap_or_default()
            } else {
                value
            };
            cells.push(resolved);
            cell_rest = &cell_rest[c_end..];
        }
        rows.push(cells);
        rest = &rest[row_end..];
    }
    rows
}

fn extract_value(cell_xml: &str) -> String {
    // Extract content between <v> and </v>
    if let Some(v_start) = cell_xml.find("<v>") {
        if let Some(v_end) = cell_xml[v_start..].find("</v>") {
            return cell_xml[v_start + 3..v_start + v_end].to_string();
        }
    }
    String::new()
}
