// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//! Quick UTXO diagnostic for all wallets

use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::wallet::Wallet;

const RPC_URL: &str = "ws://127.0.0.1:17210";

#[tokio::test]
async fn check_all_utxos() {
    let client = KaspaClient::new(RPC_URL).unwrap();
    client.connect().await.unwrap();

    let wallets = [
        (
            "Issuer",
            "91149facb865c1f35b4cdec412caef7cd41191372024cd37cf9fd4a9b6bf686d",
        ),
        (
            "Bob",
            "37df3703a12b02b3d0a16efa38ca53cda2ee5e9eaa3b8861dc8e04383fb3fecc",
        ),
        (
            "Alice",
            "ab08984d79824336161553b77e366abde831ebde78d78f0440e6833b2f2e2f92",
        ),
    ];

    for (name, key) in wallets {
        let w = Wallet::from_hex(key).unwrap();
        let addr = w.address_string();
        let utxos = client.get_spendable_utxos(&addr).await.unwrap();
        let total: u64 = utxos.iter().map(|u| u.amount).sum();
        println!(
            "\n[K-RWA] {} ({}) — {} UTXOs, total {:.4} KAS",
            name,
            addr,
            utxos.len(),
            total as f64 / 1e8
        );
        for (i, u) in utxos.iter().enumerate() {
            if i < 10 || utxos.len() <= 15 {
                println!(
                    "  [{:2}] txid={}..{} idx={} amount={:.4} KAS",
                    i,
                    &u.txid.to_string()[..8],
                    &u.txid.to_string()[56..],
                    u.index,
                    u.amount as f64 / 1e8
                );
            }
        }
        if utxos.len() > 15 {
            println!("  ... and {} more", utxos.len() - 10);
        }
    }

    client.disconnect().await.unwrap();
}
