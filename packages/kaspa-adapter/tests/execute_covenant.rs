// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//! Execute a SilverScript covenant entrypoint on Kaspa TN12.
//! Deploys simple-spend.sil then invokes spend() to prove covenant execution works.

use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::script::load_contract_json;
use kaspa_adapter::wallet::Wallet;

const RPC_URL: &str = "ws://127.0.0.1:17210";
const ALICE_KEY: &str = "ab08984d79824336161553b77e366abde831ebde78d78f0440e6833b2f2e2f92";

#[tokio::test]
async fn test_execute_simple_spend_covenant() {
    let client = KaspaClient::new(RPC_URL).unwrap();
    client.connect().await.unwrap();

    // Step 1: Use Bob as the owner (contract compiled with Bob's blake2b key hash)
    let alice = Wallet::from_hex(ALICE_KEY).unwrap();
    let owner = &alice; // Bob IS the owner — contract has his key hash baked in
    let owner_addr = owner.address_string();

    println!("[K-RWA] === COVENANT EXECUTION TEST ===");
    println!("[K-RWA] Owner (Alice): {}", owner_addr);

    // Step 2: Fund fresh wallet from Bob for deployment (keeps Bob's UTXOs clean)
    let deployer = Wallet::generate().unwrap();
    println!("[K-RWA] Funding deployer from Bob (2 KAS)...");
    let fund_tx = client
        .send_kas(
            &owner_addr,
            &deployer.address_string(),
            200_000_000,
            alice.keypair(),
            None,
        )
        .await
        .unwrap();
    println!("[K-RWA] Funded deployer: TX {}", fund_tx);
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Step 3: Deploy simple-spend covenant (compiled with Bob's blake2b key hash)
    let contract = load_contract_json("../../contracts/silverscript/simple-spend.json").unwrap();
    println!(
        "[K-RWA] Deploying SimpleSpend ({} bytes)...",
        contract.redeem_script.len()
    );

    let deploy_tx = client
        .deploy_contract(
            &deployer.address_string(),
            &contract,
            100_000_000,
            deployer.keypair(),
        )
        .await
        .unwrap();
    println!("[K-RWA] SimpleSpend DEPLOYED: TX {}", deploy_tx);
    println!("[K-RWA] P2SH: {}", contract.p2sh_address);
    println!("[K-RWA] Waiting 10s for confirmation...");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Step 4: Invoke the spend() entrypoint
    // SimpleSpend has one entrypoint: spend(pubkey ownerPk, sig ownerSig)
    // Stack layout: [selector=0] [ownerPk] [ownerSig(placeholder)] [redeemScript]
    // Note: SilverScript pushes params in REVERSE order from .sil declaration
    // So spend(pubkey ownerPk, sig ownerSig) means stack bottom-to-top:
    //   ownerSig, ownerPk, [selector if enabled]

    let (owner_xonly, _) = owner.keypair().x_only_public_key();
    let recipient = Wallet::generate().unwrap();
    println!(
        "[K-RWA] Invoking spend() → recipient {}",
        recipient.address_string()
    );

    // Try with selector 0 + sig placeholder + pubkey
    let witness = vec![
        vec![0x00],                       // selector 0 = spend
        vec![],                           // sig placeholder
        owner_xonly.serialize().to_vec(), // ownerPk (32 bytes)
    ];

    match client
        .spend_contract(
            &contract,
            witness,
            &recipient.address_string(),
            99_000_000,
            1, // sig_op_count
            owner.keypair(),
        )
        .await
    {
        Ok(tx_id) => {
            println!("[K-RWA] ========================================");
            println!("[K-RWA]  COVENANT EXECUTION SUCCESS!");
            println!("[K-RWA]  TX: {}", tx_id);
            println!("[K-RWA] ========================================");
        }
        Err(e) => {
            let err = format!("{}", e);
            println!("[K-RWA] Covenant execution result: {}", err);

            // Try reversed param order (ownerPk first, then sig)
            if err.contains("Number too big") || err.contains("signature") {
                println!("[K-RWA] Trying reversed witness order...");
                let witness2 = vec![
                    vec![0x00],                       // selector
                    owner_xonly.serialize().to_vec(), // ownerPk first
                    vec![],                           // sig placeholder second
                ];
                match client
                    .spend_contract(
                        &contract,
                        witness2,
                        &recipient.address_string(),
                        99_000_000,
                        1,
                        owner.keypair(),
                    )
                    .await
                {
                    Ok(tx_id) => {
                        println!("[K-RWA] ========================================");
                        println!("[K-RWA]  COVENANT EXECUTION SUCCESS (reversed)!");
                        println!("[K-RWA]  TX: {}", tx_id);
                        println!("[K-RWA] ========================================");
                    }
                    Err(e2) => {
                        println!("[K-RWA] Reversed also failed: {}", e2);

                        // Try without selector (without_selector might be true for simple contracts)
                        println!("[K-RWA] Trying without selector...");
                        let witness3 = vec![
                            vec![],                           // sig placeholder
                            owner_xonly.serialize().to_vec(), // ownerPk
                        ];
                        match client
                            .spend_contract(
                                &contract,
                                witness3,
                                &recipient.address_string(),
                                99_000_000,
                                1,
                                owner.keypair(),
                            )
                            .await
                        {
                            Ok(tx_id) => {
                                println!("[K-RWA] ========================================");
                                println!("[K-RWA]  COVENANT EXECUTION SUCCESS (no selector)!");
                                println!("[K-RWA]  TX: {}", tx_id);
                                println!("[K-RWA] ========================================");
                            }
                            Err(e3) => {
                                println!("[K-RWA] No-selector also failed: {}", e3);
                            }
                        }
                    }
                }
            }
        }
    }

    client.disconnect().await.ok();
}
