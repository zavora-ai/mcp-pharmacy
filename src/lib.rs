//! # mcp-pharmacy
//!
//! Pharmacy MCP server — 19 tools for drug information, interactions,
//! clinical trials, FDA data, and international drug databases.
pub mod server;
pub mod types;
pub mod clinicaltrials;
pub mod dailymed;
pub mod ema;
pub mod health_canada;
pub mod openfda;
pub mod pubchem;
pub mod rxnorm;
