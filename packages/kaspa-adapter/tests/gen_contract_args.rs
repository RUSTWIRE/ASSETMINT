// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Generate real SilverScript constructor argument files from testnet wallet keys.
//!
//! This test derives x-only public keys from the 3 testnet private keys (alice, bob, issuer),
//! computes `blake2b(x_only_pubkey)` using the same algorithm as Kaspa's `OP_BLAKE2B` opcode
//! (blake2b-256, no key/domain separation), and outputs hex values for use in .sil contracts.
//!
//! Run: cargo test -p kaspa-adapter --test gen_contract_args -- --nocapture

use kaspa_adapter::wallet::Wallet;
use blake2b_simd::Params;
use secp256k1::{Secp256k1, SecretKey, Keypair};
use std::io::Write;

const ALICE_KEY: &str = "ab08984d79824336161553b77e366abde831ebde78d78f0440e6833b2f2e2f92";
const BOB_KEY: &str = "37df3703a12b02b3d0a16efa38ca53cda2ee5e9eaa3b8861dc8e04383fb3fecc";
const ISSUER_KEY: &str = "91149facb865c1f35b4cdec412caef7cd41191372024cd37cf9fd4a9b6bf686d";

/// Compute blake2b-256 hash matching Kaspa's OP_BLAKE2B (no key, no domain separation).
fn kaspa_blake2b(data: &[u8]) -> [u8; 32] {
    let hash = Params::new()
        .hash_length(32)
        .to_state()
        .update(data)
        .finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(hash.as_bytes());
    out
}

/// Derive x-only public key bytes from a hex private key.
fn x_only_pubkey_bytes(hex_key: &str) -> [u8; 32] {
    let secp = Secp256k1::new();
    let secret = SecretKey::from_slice(&hex::decode(hex_key).unwrap()).unwrap();
    let keypair = Keypair::from_secret_key(&secp, &secret);
    let (xonly, _) = keypair.x_only_public_key();
    xonly.serialize()
}

/// Build a SilverScript args JSON array of byte[32] entries from a list of 32-byte arrays,
/// with optional trailing `int` values.
fn build_args_json(byte_arrays: &[[u8; 32]], ints: &[i64]) -> serde_json::Value {
    let mut arr = Vec::new();
    for bytes in byte_arrays {
        let byte_entries: Vec<serde_json::Value> = bytes
            .iter()
            .map(|&b| serde_json::json!({"kind": "byte", "data": b as u64}))
            .collect();
        arr.push(serde_json::json!({"kind": "array", "data": byte_entries}));
    }
    for &val in ints {
        arr.push(serde_json::json!({"kind": "int", "data": val}));
    }
    serde_json::Value::Array(arr)
}

#[test]
fn generate_contract_args() {
    println!("\n=== AssetMint Testnet Key Derivation ===\n");

    // Create wallets and derive key hashes
    let alice_wallet = Wallet::from_hex(ALICE_KEY).expect("alice wallet");
    let bob_wallet = Wallet::from_hex(BOB_KEY).expect("bob wallet");
    let issuer_wallet = Wallet::from_hex(ISSUER_KEY).expect("issuer wallet");

    let alice_xonly = x_only_pubkey_bytes(ALICE_KEY);
    let bob_xonly = x_only_pubkey_bytes(BOB_KEY);
    let issuer_xonly = x_only_pubkey_bytes(ISSUER_KEY);

    let alice_keyhash = kaspa_blake2b(&alice_xonly);
    let bob_keyhash = kaspa_blake2b(&bob_xonly);
    let issuer_keyhash = kaspa_blake2b(&issuer_xonly);

    println!("Alice:");
    println!("  Address:    {}", alice_wallet.address_string());
    println!("  X-only PK:  {}", hex::encode(alice_xonly));
    println!("  blake2b():  {}", hex::encode(alice_keyhash));

    println!("\nBob:");
    println!("  Address:    {}", bob_wallet.address_string());
    println!("  X-only PK:  {}", hex::encode(bob_xonly));
    println!("  blake2b():  {}", hex::encode(bob_keyhash));

    println!("\nIssuer:");
    println!("  Address:    {}", issuer_wallet.address_string());
    println!("  X-only PK:  {}", hex::encode(issuer_xonly));
    println!("  blake2b():  {}", hex::encode(issuer_keyhash));

    // --- Generate constructor arg JSON files ---
    // Use issuer_keyhash as the "admin" / "issuer" / "stateManager" key hash.
    // Use alice_keyhash as the initial "oracle" key hash (alice acts as oracle in testnet).
    // Use bob_keyhash as the initial "custodian" key hash (bob acts as custodian in testnet).

    // For initial merkle roots / attestation hashes / DKG UAL hashes, use sha256("AssetMint-init-<field>")
    // as a deterministic placeholder that is non-zero.
    use sha2::{Sha256, Digest};

    let init_merkle_root = {
        let mut h = Sha256::new();
        h.update(b"AssetMint-init-merkleRoot");
        let r: [u8; 32] = h.finalize().into();
        r
    };
    let init_zk_verifier_key_hash = {
        let mut h = Sha256::new();
        h.update(b"AssetMint-init-zkVerifierKeyHash");
        let r: [u8; 32] = h.finalize().into();
        r
    };
    let init_dkg_ual_hash = {
        let mut h = Sha256::new();
        h.update(b"AssetMint-init-dkgUalHash");
        let r: [u8; 32] = h.finalize().into();
        r
    };
    let init_oracle_attestation_hash = {
        let mut h = Sha256::new();
        h.update(b"AssetMint-init-oracleAttestationHash");
        let r: [u8; 32] = h.finalize().into();
        r
    };
    let init_compliance_merkle_root = {
        let mut h = Sha256::new();
        h.update(b"AssetMint-init-complianceMerkleRoot");
        let r: [u8; 32] = h.finalize().into();
        r
    };

    let args_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .join("contracts/silverscript/args");

    // 1. rwa-core-args.json
    //    RwaCore(byte[32] initMerkleRoot, byte[32] initZkVerifierKeyHash, byte[32] initIssuerKeyHash)
    let rwa_core = build_args_json(&[init_merkle_root, init_zk_verifier_key_hash, issuer_keyhash], &[]);
    write_json(&args_dir.join("rwa-core-args.json"), &rwa_core);
    println!("\nWrote rwa-core-args.json");

    // 2. clawback-args.json
    //    Clawback(pubkey owner, byte[32] issuerKeyHash)
    //    NOTE: `pubkey owner` is a raw x-only pubkey, not a hash. We use alice as the owner.
    //    But the arg format uses byte[32] arrays for everything. A pubkey is 32 bytes (x-only).
    let clawback = build_args_json(&[alice_xonly, issuer_keyhash], &[]);
    write_json(&args_dir.join("clawback-args.json"), &clawback);
    println!("Wrote clawback-args.json");

    // 3. reserves-args.json
    //    Reserves(byte[32] initOracleKeyHash, byte[32] initCustodianKeyHash, int initMinReserveRatio)
    //    minReserveRatio = 15000 means 150% (RATIO_DENOMINATOR = 10000)
    let reserves = build_args_json(&[alice_keyhash, bob_keyhash], &[15000]);
    write_json(&args_dir.join("reserves-args.json"), &reserves);
    println!("Wrote reserves-args.json");

    // 4. zkkyc-verifier-args.json
    //    ZkKycVerifier(byte[32] initVerifierKeyHash, byte[32] initApprovedMerkleRoot, byte[32] initAdminKeyHash)
    let zkkyc = build_args_json(&[init_zk_verifier_key_hash, init_merkle_root, issuer_keyhash], &[]);
    write_json(&args_dir.join("zkkyc-verifier-args.json"), &zkkyc);
    println!("Wrote zkkyc-verifier-args.json");

    // 5. state-verity-args.json
    //    StateVerity(byte[32] initDkgUalHash, byte[32] initOracleAttestationHash,
    //                byte[32] initComplianceMerkleRoot, byte[32] initStateManagerKeyHash)
    let state_verity = build_args_json(
        &[init_dkg_ual_hash, init_oracle_attestation_hash, init_compliance_merkle_root, issuer_keyhash],
        &[],
    );
    write_json(&args_dir.join("state-verity-args.json"), &state_verity);
    println!("Wrote state-verity-args.json");

    println!("\n=== All 5 constructor arg files generated ===");
}

fn write_json(path: &std::path::Path, value: &serde_json::Value) {
    let json = serde_json::to_string_pretty(value).expect("serialize json");
    let mut f = std::fs::File::create(path).expect(&format!("create {}", path.display()));
    f.write_all(json.as_bytes()).expect("write json");
    f.write_all(b"\n").expect("write newline");
}
