// Quick test: deploy a fresh Clawback then spend it via ownerSpend
use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::script::load_contract_json;
use kaspa_adapter::wallet::Wallet;

const RPC_URL: &str = "ws://127.0.0.1:17210";
const BOB_KEY: &str = "37df3703a12b02b3d0a16efa38ca53cda2ee5e9eaa3b8861dc8e04383fb3fecc";

#[tokio::test]
async fn test_deploy_then_spend_clawback() {
    let client = KaspaClient::new(RPC_URL).unwrap();
    client.connect().await.unwrap();

    // Generate fresh wallet, fund from Bob
    let fresh = Wallet::generate().unwrap();
    let fresh_addr = fresh.address_string();
    let bob = Wallet::from_hex(BOB_KEY).unwrap();

    println!("[K-RWA] Funding fresh wallet from Bob (3 KAS)...");
    let _fund = client
        .send_kas(
            &bob.address_string(),
            &fresh_addr,
            300_000_000,
            bob.keypair(),
            None,
        )
        .await
        .unwrap();
    println!("[K-RWA] Waiting 10s for confirmation...");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Deploy clawback with 1 KAS
    let contract = load_contract_json("../../contracts/silverscript/clawback.json").unwrap();
    println!("[K-RWA] Deploying Clawback...");
    let deploy_tx = client
        .deploy_contract(&fresh_addr, &contract, 100_000_000, fresh.keypair())
        .await
        .unwrap();
    println!("[K-RWA] Clawback deployed: TX {}", deploy_tx);
    println!("[K-RWA] P2SH: {}", contract.p2sh_address);

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Now spend it via ownerSpend!
    // SilverScript uses function selectors (without_selector: false)
    // ownerSpend = selector 0, issuerClawback = selector 1
    // Stack layout: [selector] [ownerSig] [recipientPk] [redeemScript]
    let recipient = Wallet::generate().unwrap();
    let (recipient_xonly, _) = recipient.keypair().x_only_public_key();
    let witness = vec![
        vec![0x00],                           // Function selector 0 = ownerSpend (OP_FALSE / OP_0)
        vec![], // ownerSig placeholder — will be filled with Schnorr sig
        recipient_xonly.serialize().to_vec(), // recipientPk (32 bytes x-only)
    ];
    println!(
        "[K-RWA] Invoking ownerSpend → recipient {}",
        recipient.address_string()
    );

    match client
        .spend_contract(
            &contract,
            witness,
            &recipient.address_string(),
            50_000_000,
            1,
            fresh.keypair(),
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
            println!("[K-RWA] Covenant execution failed: {}", e);
        }
    }

    client.disconnect().await.ok();
}
