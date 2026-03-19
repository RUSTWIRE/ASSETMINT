// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! AssetMint CLI — command-line interface to the AssetMint compliance API.
//! Communicates with the running Axum HTTP server (default http://localhost:3001).

use clap::{Parser, Subcommand};
use colored::Colorize;
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde_json::Value;
use std::process;

// ── CLI Structure ─────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "assetmint",
    about = "AssetMint RWA Compliance CLI — interact with the AssetMint API",
    version,
    propagate_version = true
)]
struct Cli {
    /// Base URL of the AssetMint API server
    #[arg(long, default_value = "http://localhost:3001", global = true)]
    api_url: String,

    /// API key for authenticated (write) endpoints
    #[arg(long, global = true)]
    api_key: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check API health status
    Health,

    /// Display Kaspa network information
    Network,

    /// Identity management
    Identity {
        #[command(subcommand)]
        action: IdentityAction,
    },

    /// Issue or manage verifiable claims
    Claim {
        #[command(subcommand)]
        action: ClaimAction,
    },

    /// Evaluate compliance for a transfer
    Compliance {
        #[command(subcommand)]
        action: ComplianceAction,
    },

    /// Query address balance on Kaspa
    Balance {
        /// Kaspa address to query
        #[arg(long)]
        address: String,
    },

    /// Execute a compliant transfer
    Transfer {
        /// Sender DID
        #[arg(long)]
        sender_did: String,

        /// Receiver DID
        #[arg(long)]
        receiver_did: String,

        /// Receiver Kaspa address
        #[arg(long)]
        receiver_address: String,

        /// Amount in sompis
        #[arg(long)]
        amount: u64,

        /// Asset identifier
        #[arg(long)]
        asset: String,
    },

    /// Query the current Merkle root of approved addresses
    MerkleRoot,
}

#[derive(Subcommand)]
enum IdentityAction {
    /// Register a new identity
    Register {
        /// Decentralized identifier
        #[arg(long)]
        did: String,

        /// Primary public key (hex)
        #[arg(long)]
        key: String,
    },

    /// Look up an identity by DID
    Get {
        /// Decentralized identifier to look up
        #[arg(long)]
        did: String,
    },
}

#[derive(Subcommand)]
enum ClaimAction {
    /// Issue a new claim
    Issue {
        /// Subject DID
        #[arg(long)]
        subject: String,

        /// Claim type (KycVerified, AccreditedInvestor, AmlClear, ExemptedEntity, JurisdictionAllowed)
        #[arg(long, name = "type")]
        claim_type: String,

        /// Expiry timestamp (Unix seconds, 0 = never)
        #[arg(long)]
        expiry: u64,

        /// Jurisdiction (required for JurisdictionAllowed claims)
        #[arg(long)]
        jurisdiction: Option<String>,
    },
}

#[derive(Subcommand)]
enum ComplianceAction {
    /// Evaluate whether a transfer is compliant
    Check {
        /// Sender DID
        #[arg(long)]
        sender: String,

        /// Receiver DID
        #[arg(long)]
        receiver: String,

        /// Asset identifier
        #[arg(long)]
        asset: String,

        /// Transfer amount
        #[arg(long)]
        amount: u64,
    },
}

// ── HTTP Helpers ──────────────────────────────────────────────────────

fn build_client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_else(|e| {
            eprintln!("{} Failed to create HTTP client: {}", "[ERROR]".red(), e);
            process::exit(1);
        })
}

fn handle_response(resp: reqwest::blocking::Response) {
    let status = resp.status();
    let body: Value = resp.json().unwrap_or_else(|e| {
        eprintln!("{} Failed to parse response body: {}", "[ERROR]".red(), e);
        process::exit(1);
    });

    if status.is_success() {
        let pretty = serde_json::to_string_pretty(&body).unwrap_or_default();
        println!("{} {}", "[OK]".green(), status);
        println!("{}", pretty);
    } else {
        let pretty = serde_json::to_string_pretty(&body).unwrap_or_default();
        eprintln!("{} {}", "[FAIL]".red(), status);
        eprintln!("{}", pretty);
        process::exit(1);
    }
}

fn handle_request_error(e: reqwest::Error) -> ! {
    if e.is_connect() {
        eprintln!(
            "{} Connection refused — is the AssetMint API running?",
            "[ERROR]".red()
        );
        eprintln!("  Start it with: make backend");
    } else if e.is_timeout() {
        eprintln!("{} Request timed out", "[ERROR]".red());
    } else if let Some(StatusCode::UNAUTHORIZED) = e.status() {
        eprintln!(
            "{} Unauthorized — provide a valid --api-key",
            "[ERROR]".red()
        );
    } else {
        eprintln!("{} Request failed: {}", "[ERROR]".red(), e);
    }
    process::exit(1);
}

// ── Main ──────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let client = build_client();
    let base = cli.api_url.trim_end_matches('/');

    match cli.command {
        Commands::Health => {
            let resp = client
                .get(format!("{}/health", base))
                .send()
                .unwrap_or_else(|e| handle_request_error(e));
            handle_response(resp);
        }

        Commands::Network => {
            let resp = client
                .get(format!("{}/network", base))
                .send()
                .unwrap_or_else(|e| handle_request_error(e));
            handle_response(resp);
        }

        Commands::Identity { action } => match action {
            IdentityAction::Register { did, key } => {
                let mut req = client
                    .post(format!("{}/identity", base))
                    .json(&serde_json::json!({
                        "did": did,
                        "primary_key": key,
                    }));
                if let Some(ref api_key) = cli.api_key {
                    req = req.header("X-API-Key", api_key);
                }
                let resp = req.send().unwrap_or_else(|e| handle_request_error(e));
                handle_response(resp);
            }
            IdentityAction::Get { did } => {
                let resp = client
                    .get(format!("{}/identity", base))
                    .query(&[("did", &did)])
                    .send()
                    .unwrap_or_else(|e| handle_request_error(e));
                handle_response(resp);
            }
        },

        Commands::Claim { action } => match action {
            ClaimAction::Issue {
                subject,
                claim_type,
                expiry,
                jurisdiction,
            } => {
                let mut body = serde_json::json!({
                    "subject_did": subject,
                    "claim_type": claim_type,
                    "expiry": expiry,
                });
                if let Some(j) = jurisdiction {
                    body["jurisdiction"] = serde_json::Value::String(j);
                }
                let mut req = client.post(format!("{}/claim", base)).json(&body);
                if let Some(ref api_key) = cli.api_key {
                    req = req.header("X-API-Key", api_key);
                }
                let resp = req.send().unwrap_or_else(|e| handle_request_error(e));
                handle_response(resp);
            }
        },

        Commands::Compliance { action } => match action {
            ComplianceAction::Check {
                sender,
                receiver,
                asset,
                amount,
            } => {
                let resp = client
                    .get(format!("{}/compliance/evaluate", base))
                    .query(&[
                        ("sender_did", sender),
                        ("receiver_did", receiver),
                        ("asset_id", asset),
                        ("amount", amount.to_string()),
                    ])
                    .send()
                    .unwrap_or_else(|e| handle_request_error(e));
                handle_response(resp);
            }
        },

        Commands::Balance { address } => {
            let resp = client
                .get(format!("{}/balance", base))
                .query(&[("address", &address)])
                .send()
                .unwrap_or_else(|e| handle_request_error(e));
            handle_response(resp);
        }

        Commands::Transfer {
            sender_did,
            receiver_did,
            receiver_address,
            amount,
            asset,
        } => {
            // Note: the API requires zk_proof and zk_public_inputs fields.
            // In a real workflow, the CLI would first call /zk-proof/{address}
            // to generate a proof, then include it here. For now we pass
            // empty values so the server can validate and return an error
            // if proofs are missing.
            let body = serde_json::json!({
                "sender_did": sender_did,
                "receiver_did": receiver_did,
                "receiver_address": receiver_address,
                "amount_sompis": amount,
                "asset_id": asset,
                "zk_proof": "",
                "zk_public_inputs": [],
            });
            let mut req = client.post(format!("{}/transfer", base)).json(&body);
            if let Some(ref api_key) = cli.api_key {
                req = req.header("X-API-Key", api_key);
            }
            let resp = req.send().unwrap_or_else(|e| handle_request_error(e));
            handle_response(resp);
        }

        Commands::MerkleRoot => {
            let resp = client
                .get(format!("{}/merkle-root", base))
                .send()
                .unwrap_or_else(|e| handle_request_error(e));
            handle_response(resp);
        }
    }
}
