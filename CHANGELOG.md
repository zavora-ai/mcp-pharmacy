# Changelog

## [1.0.0] — 2026-05-25

### Added
- **OpenFDA backend** — drug labels, FAERS adverse events, recalls/enforcement, NDC directory
- **DailyMed backend** — SPL label search, full XML label retrieval
- **RxNorm backend** — drug name normalization, RxCUI properties, WHO ATC class mapping
- **PubChem backend** — compound search by name, molecular properties (formula, weight, SMILES)
- **Health Canada DPD backend** — Canadian drug product search by brand/ingredient, DIN lookup
- **ClinicalTrials.gov v2 backend** — trial search by drug/condition, study detail by NCT ID
- **EMA backend** — EU-authorized medicines from downloadable XLSX (cached on startup)
- **MHRA backend** — UK Drug Safety Updates via GOV.UK search API
- **Cross-backend tools** — federated global drug search, multi-region registration status
- 19 tools total, all read-only, no credentials required
- Registry-compatible `mcp-server.toml` manifest

### Design decisions
- **No interaction checking** — RxNorm interactions API discontinued in 2024; requires licensed source
- **No clinical dosing** — requires BNF, Lexicomp, or equivalent licensed knowledge base
- **EMA via XLSX** — EMA does not expose a simple JSON API; we download and parse their official XLSX
- **All free public APIs** — zero cost, no API keys required (OpenFDA key optional for rate limits)
