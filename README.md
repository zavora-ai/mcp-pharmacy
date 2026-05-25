# mcp-pharmacy

Global drug reference and regulatory intelligence MCP server. Provides federated access to drug labels, safety alerts, adverse events, product registrations, clinical trials, and chemical identity across US, EU, UK, and Canada — using only free public APIs.

![Architecture](https://raw.githubusercontent.com/zavora-ai/mcp-pharmacy/main/docs/assets/architecture.svg)

## What this is

A **regulatory reference and safety intelligence** platform — not clinical decision support. It answers:

- "What is this drug registered as in the US/EU/Canada?"
- "Are there FDA recalls or safety alerts for this drug?"
- "What adverse events have been reported?"
- "What's the WHO ATC classification?"
- "Are there clinical trials for this condition?"
- "What's the chemical structure?"

## What this is NOT

This server does **not** provide:
- Drug-drug interaction checking (RxNav interactions API was discontinued in 2024)
- Clinical dosing recommendations
- Pregnancy/lactation safety ratings
- Therapeutic alternatives
- Formulary or pricing information

Those require licensed clinical knowledge bases (DrugBank, Lexicomp, BNF, etc.).

## Backends (7 validated public APIs)

| Backend | Region | Tools | What it provides |
|---------|--------|:-----:|-----------------|
| **OpenFDA** | US | 4 | Drug labels, FAERS adverse events, recalls, NDC directory |
| **DailyMed** | US | 2 | Structured product labels (SPL), full label XML |
| **RxNorm** | US/Global | 3 | Drug name normalization, RxCUI mapping, WHO ATC classes |
| **PubChem** | Global | 2 | Chemical compound identity, molecular properties |
| **Health Canada DPD** | Canada | 2 | Canadian drug product database |
| **ClinicalTrials.gov** | Global | 2 | Clinical trial search and detail (v2 API) |
| **EMA** | EU | 1 | EU-authorized medicines (XLSX data, cached on startup) |
| **MHRA** | UK | 1 | Drug Safety Updates |
| **Cross-backend** | Multi-region | 2 | Federated search, registration status comparison |

**Total: 19 tools**

## Quick start

```bash
# No credentials required — all APIs are free and public
cargo install mcp-pharmacy

# Run (downloads EMA data on first start, ~890KB)
mcp-pharmacy

# Skip EMA download for faster startup
EMA_SKIP=1 mcp-pharmacy

# Optional: OpenFDA API key for higher rate limits
OPENFDA_API_KEY=your_key mcp-pharmacy
```

## Tools

### OpenFDA (US)
| Tool | Description |
|------|-------------|
| `openfda_search_labels` | Search drug labels by generic/brand name |
| `openfda_get_adverse_events` | Search FAERS adverse event reports |
| `openfda_search_recalls` | Search drug recalls and enforcement |
| `openfda_get_ndc` | Look up NDC directory entries |

### DailyMed (US)
| Tool | Description |
|------|-------------|
| `dailymed_search_labels` | Search structured product labels |
| `dailymed_get_label_xml` | Get full SPL XML by setid |

### RxNorm (Normalization)
| Tool | Description |
|------|-------------|
| `rxnorm_normalize` | Normalize drug name → RxCUI |
| `rxnorm_get_properties` | Get drug properties by RxCUI |
| `rxnorm_get_atc_classes` | Get WHO ATC codes via RxCUI |

### PubChem (Chemistry)
| Tool | Description |
|------|-------------|
| `pubchem_search_compound` | Search compound by name → CID |
| `pubchem_get_properties` | Get molecular formula, weight, SMILES |

### Health Canada (Canada)
| Tool | Description |
|------|-------------|
| `health_canada_search_products` | Search by brand name or ingredient |
| `health_canada_get_product` | Get product details by DIN |

### ClinicalTrials.gov (Global)
| Tool | Description |
|------|-------------|
| `clinicaltrials_search` | Search trials by drug/condition |
| `clinicaltrials_get_study` | Get study details by NCT ID |

### EMA (EU)
| Tool | Description |
|------|-------------|
| `ema_search_medicines` | Search EU-authorized medicines |

### MHRA (UK)
| Tool | Description |
|------|-------------|
| `mhra_search_safety_updates` | Search UK Drug Safety Updates |

### Cross-backend
| Tool | Description |
|------|-------------|
| `search_drug_global` | Federated search across US, EU, Canada |
| `get_registration_status` | Compare registration across regions |

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    MCP Client (Agent)                     │
└─────────────────────┬───────────────────────────────────┘
                      │ stdio (JSON-RPC)
┌─────────────────────▼───────────────────────────────────┐
│                  mcp-pharmacy server                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐  │
│  │ OpenFDA  │ │ DailyMed │ │  RxNorm  │ │  PubChem  │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └─────┬─────┘  │
│  ┌────┴─────┐ ┌────┴──────┐ ┌───┴──────┐ ┌────┴─────┐  │
│  │Health CA │ │ClinTrials │ │   EMA    │ │   MHRA   │  │
│  └────┬─────┘ └────┬──────┘ └───┬──────┘ └────┬─────┘  │
└───────┼─────────────┼────────────┼─────────────┼────────┘
        │             │            │             │
   health-products  clinicaltrials  ema.europa   gov.uk
   .canada.ca       .gov/api/v2    .eu (XLSX)   /api
```

## Roadmap

### v1.1 — More regions
- SFDA (Saudi Arabia) drug list connector
- SAHPRA (South Africa) registered products
- NAFDAC (Nigeria) Greenbook connector
- CDSCO (India) approvals

### v2.0 — Licensed clinical tier
- DrugBank Clinical API (interactions, mechanisms)
- BNF/BNFC (UK dosing, licensed)
- VigiAccess (global pharmacovigilance signals)

## License

Apache-2.0
