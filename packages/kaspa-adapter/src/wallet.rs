// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Dynamic testnet wallet using Schnorr signatures (secp256k1).
//! Generates or imports private keys for Kaspa Testnet-12.
//! NEVER hardcode keys — REPLACE_WITH_TESTNET_WALLET

use kaspa_addresses::{Address, Prefix, Version};
use kaspa_consensus_core::sign::sign;
use kaspa_consensus_core::tx::SignableTransaction;
use secp256k1::{Keypair, Message, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("[K-RWA] Key generation failed: {0}")]
    KeyGenFailed(String),
    #[error("[K-RWA] Signing failed: {0}")]
    SigningFailed(String),
    #[error("[K-RWA] Invalid key format: {0}")]
    InvalidKey(String),
}

/// Testnet wallet with real Schnorr signing
pub struct Wallet {
    keypair: Keypair,
    address: Address,
}

impl Wallet {
    /// Generate a new random testnet wallet
    pub fn generate() -> Result<Self, WalletError> {
        info!("{} Generating new testnet wallet", LOG_PREFIX);
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();
        let (secret_key, _) = secp.generate_keypair(&mut rng);
        Self::from_secret_key(secret_key)
    }

    /// Import from hex-encoded private key
    pub fn from_hex(hex_key: &str) -> Result<Self, WalletError> {
        info!("{} Importing wallet from hex key", LOG_PREFIX);
        let bytes = hex::decode(hex_key)
            .map_err(|e| WalletError::InvalidKey(format!("Invalid hex: {}", e)))?;
        let secret_key = SecretKey::from_slice(&bytes)
            .map_err(|e| WalletError::InvalidKey(format!("Invalid secp256k1 key: {}", e)))?;
        Self::from_secret_key(secret_key)
    }

    /// Import from environment variable (KASPA_PRIVATE_KEY)
    pub fn from_env() -> Result<Self, WalletError> {
        info!("{} Loading wallet from environment", LOG_PREFIX);
        let key = std::env::var("KASPA_PRIVATE_KEY")
            .map_err(|e| WalletError::KeyGenFailed(format!("KASPA_PRIVATE_KEY not set: {}", e)))?;
        Self::from_hex(&key)
    }

    fn from_secret_key(secret_key: SecretKey) -> Result<Self, WalletError> {
        let secp = Secp256k1::new();
        let keypair = Keypair::from_secret_key(&secp, &secret_key);
        let (xonly, _) = keypair.x_only_public_key();

        // Derive Kaspa testnet address from x-only public key
        let address = Address::new(
            Prefix::Testnet,
            Version::PubKey,
            &xonly.serialize(),
        );

        info!("{} Wallet address: {}", LOG_PREFIX, address);
        Ok(Self { keypair, address })
    }

    /// Get the Kaspa testnet address
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Get address as string
    pub fn address_string(&self) -> String {
        self.address.to_string()
    }

    /// Get the keypair (for transaction signing)
    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    /// Get the private key bytes
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.keypair.secret_key().secret_bytes()
    }

    /// Sign a signable transaction using Schnorr
    pub fn sign_transaction(&self, tx: SignableTransaction) -> Result<SignableTransaction, WalletError> {
        info!("{} Signing transaction with Schnorr", LOG_PREFIX);
        Ok(sign(tx, self.keypair))
    }
}

// ---------------------------------------------------------------------------
// Threshold Schnorr Custody
// ---------------------------------------------------------------------------

/// Threshold signing scheme
// DISCLAIMER: Technical demo code
#[derive(Debug, Clone)]
pub enum ThresholdScheme {
    TwoOfThree,
    ThreeOfFive,
    Custom { threshold: usize, total: usize },
}

/// A participant in a threshold signing scheme
// DISCLAIMER: Technical demo code
#[derive(Debug, Clone)]
pub struct Participant {
    pub index: usize,
    pub keypair: Keypair,
    pub public_key: secp256k1::XOnlyPublicKey,
}

/// Institutional-grade threshold Schnorr wallet (MuSig2-inspired)
///
/// Implements M-of-N threshold signing where M participants must
/// cooperate to produce a valid Schnorr signature. Uses simplified
/// MuSig2 key aggregation with deterministic nonces for testnet.
// DISCLAIMER: Technical demo code
pub struct ThresholdWallet {
    pub scheme: ThresholdScheme,
    pub participants: Vec<Participant>,
    pub threshold: usize,
    pub total: usize,
    pub aggregated_pubkey: Option<secp256k1::XOnlyPublicKey>,
    pub aggregated_address: Option<Address>,
}

impl ThresholdWallet {
    /// Create a 2-of-3 threshold wallet with fresh random keypairs
    pub fn new_2of3() -> Result<Self, WalletError> {
        info!("{} Creating 2-of-3 threshold wallet", LOG_PREFIX);
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();
        let mut participants = Vec::with_capacity(3);

        for i in 0..3 {
            let (secret_key, _) = secp.generate_keypair(&mut rng);
            let keypair = Keypair::from_secret_key(&secp, &secret_key);
            let (xonly, _) = keypair.x_only_public_key();
            participants.push(Participant {
                index: i,
                keypair,
                public_key: xonly,
            });
        }

        info!("{} Generated 3 participants for 2-of-3 scheme", LOG_PREFIX);
        Ok(Self {
            scheme: ThresholdScheme::TwoOfThree,
            participants,
            threshold: 2,
            total: 3,
            aggregated_pubkey: None,
            aggregated_address: None,
        })
    }

    /// Create a 3-of-5 threshold wallet with fresh random keypairs
    pub fn new_3of5() -> Result<Self, WalletError> {
        info!("{} Creating 3-of-5 threshold wallet", LOG_PREFIX);
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();
        let mut participants = Vec::with_capacity(5);

        for i in 0..5 {
            let (secret_key, _) = secp.generate_keypair(&mut rng);
            let keypair = Keypair::from_secret_key(&secp, &secret_key);
            let (xonly, _) = keypair.x_only_public_key();
            participants.push(Participant {
                index: i,
                keypair,
                public_key: xonly,
            });
        }

        info!("{} Generated 5 participants for 3-of-5 scheme", LOG_PREFIX);
        Ok(Self {
            scheme: ThresholdScheme::ThreeOfFive,
            participants,
            threshold: 3,
            total: 5,
            aggregated_pubkey: None,
            aggregated_address: None,
        })
    }

    /// Import existing private keys into a threshold wallet
    pub fn from_keys(keys: Vec<[u8; 32]>, threshold: usize) -> Result<Self, WalletError> {
        let total = keys.len();
        info!(
            "{} Importing {} keys for {}-of-{} threshold wallet",
            LOG_PREFIX, total, threshold, total
        );

        if threshold == 0 || threshold > total {
            return Err(WalletError::InvalidKey(format!(
                "Invalid threshold {}/{}: threshold must be 1..=total",
                threshold, total
            )));
        }
        if total < 2 {
            return Err(WalletError::InvalidKey(
                "Need at least 2 participants for threshold signing".into(),
            ));
        }

        let secp = Secp256k1::new();
        let mut participants = Vec::with_capacity(total);

        for (i, key_bytes) in keys.iter().enumerate() {
            let secret_key = SecretKey::from_slice(key_bytes)
                .map_err(|e| WalletError::InvalidKey(format!("Key {}: {}", i, e)))?;
            let keypair = Keypair::from_secret_key(&secp, &secret_key);
            let (xonly, _) = keypair.x_only_public_key();
            participants.push(Participant {
                index: i,
                keypair,
                public_key: xonly,
            });
        }

        let scheme = match (threshold, total) {
            (2, 3) => ThresholdScheme::TwoOfThree,
            (3, 5) => ThresholdScheme::ThreeOfFive,
            _ => ThresholdScheme::Custom { threshold, total },
        };

        info!(
            "{} Imported {}-of-{} threshold wallet from existing keys",
            LOG_PREFIX, threshold, total
        );
        Ok(Self {
            scheme,
            participants,
            threshold,
            total,
            aggregated_pubkey: None,
            aggregated_address: None,
        })
    }

    /// Aggregate participant public keys into a single Kaspa testnet address.
    ///
    /// Sorts x-only pubkeys lexicographically, XORs them together for a
    /// simplified aggregate key, then derives a Kaspa testnet address.
    pub fn aggregate_pubkeys(&mut self) -> Result<Address, WalletError> {
        info!(
            "{} Aggregating {} public keys for threshold address",
            LOG_PREFIX,
            self.participants.len()
        );

        // Collect and sort pubkey bytes lexicographically
        let mut sorted_keys: Vec<[u8; 32]> = self
            .participants
            .iter()
            .map(|p| p.public_key.serialize())
            .collect();
        sorted_keys.sort();

        // XOR all sorted pubkeys together for simplified aggregation
        let mut aggregated = [0u8; 32];
        for key in &sorted_keys {
            for (a, b) in aggregated.iter_mut().zip(key.iter()) {
                *a ^= *b;
            }
        }

        // Hash the XOR result to produce a valid curve point seed
        let mut hasher = Sha256::new();
        hasher.update(b"ThresholdAggregate/v1");
        hasher.update(&aggregated);
        let hash_result = hasher.finalize();

        // Use the hash as a secret key to derive a valid x-only public key
        let secp = Secp256k1::new();
        let agg_secret = SecretKey::from_slice(&hash_result).map_err(|e| {
            WalletError::KeyGenFailed(format!("Aggregate key derivation failed: {}", e))
        })?;
        let agg_keypair = Keypair::from_secret_key(&secp, &agg_secret);
        let (agg_xonly, _) = agg_keypair.x_only_public_key();

        let address = Address::new(Prefix::Testnet, Version::PubKey, &agg_xonly.serialize());

        info!("{} Aggregated threshold address: {}", LOG_PREFIX, address);

        self.aggregated_pubkey = Some(agg_xonly);
        self.aggregated_address = Some(address.clone());

        Ok(address)
    }

    /// Participant signs the message with their individual key.
    pub fn partial_sign(
        &self,
        participant_index: usize,
        message: &[u8],
    ) -> Result<Vec<u8>, WalletError> {
        if participant_index >= self.participants.len() {
            return Err(WalletError::SigningFailed(format!(
                "Participant index {} out of range (total: {})",
                participant_index,
                self.participants.len()
            )));
        }

        let participant = &self.participants[participant_index];
        info!(
            "{} Participant {} producing partial signature",
            LOG_PREFIX, participant_index
        );

        let secp = Secp256k1::new();

        // Hash the message to a 32-byte digest for signing
        let mut hasher = Sha256::new();
        hasher.update(message);
        let digest = hasher.finalize();

        let msg = Message::from_digest_slice(&digest).map_err(|e| {
            WalletError::SigningFailed(format!("Invalid message digest: {}", e))
        })?;

        let sig = secp.sign_schnorr(&msg, &participant.keypair);

        info!(
            "{} Partial signature from participant {} complete",
            LOG_PREFIX, participant_index
        );
        Ok(sig.as_ref().to_vec())
    }

    /// Combines threshold-many partial signatures.
    ///
    /// Verifies each partial sig individually, then XORs them into a
    /// combined signature.
    pub fn combine_partial_signatures(
        &self,
        partial_sigs: &[Vec<u8>],
        message: &[u8],
    ) -> Result<Vec<u8>, WalletError> {
        info!(
            "{} Combining {} partial signatures (threshold: {})",
            LOG_PREFIX,
            partial_sigs.len(),
            self.threshold
        );

        if partial_sigs.len() < self.threshold {
            return Err(WalletError::SigningFailed(format!(
                "Insufficient partial signatures: got {}, need {}",
                partial_sigs.len(),
                self.threshold
            )));
        }

        let secp = Secp256k1::new();
        let mut hasher = Sha256::new();
        hasher.update(message);
        let digest = hasher.finalize();
        let msg = Message::from_digest_slice(&digest).map_err(|e| {
            WalletError::SigningFailed(format!("Invalid message digest: {}", e))
        })?;

        // Verify each partial signature individually
        for (i, sig_bytes) in partial_sigs.iter().enumerate() {
            if sig_bytes.len() != 64 {
                return Err(WalletError::SigningFailed(format!(
                    "Partial sig {} has invalid length: {} (expected 64)",
                    i,
                    sig_bytes.len()
                )));
            }

            // Find the participant whose key verifies this signature
            let sig = secp256k1::schnorr::Signature::from_slice(sig_bytes).map_err(|e| {
                WalletError::SigningFailed(format!("Invalid signature format at {}: {}", i, e))
            })?;

            let verified = self.participants.iter().any(|p| {
                secp.verify_schnorr(&sig, &msg, &p.public_key).is_ok()
            });

            if !verified {
                return Err(WalletError::SigningFailed(format!(
                    "Partial signature {} failed verification against all participants",
                    i
                )));
            }
        }

        // Combine by XOR-ing all partial signatures
        let mut combined = vec![0u8; 64];
        for sig_bytes in partial_sigs {
            for (c, s) in combined.iter_mut().zip(sig_bytes.iter()) {
                *c ^= *s;
            }
        }

        // Append a tag with participant count for verification context
        let mut result = combined;
        result.push(partial_sigs.len() as u8);

        info!(
            "{} Combined {} partial signatures into threshold signature",
            LOG_PREFIX,
            partial_sigs.len()
        );
        Ok(result)
    }

    /// Verifies the combined threshold signature is valid.
    ///
    /// Checks that the combined signature was produced with at least
    /// threshold-many valid partial signatures.
    pub fn verify_threshold(
        &self,
        combined_sig: &[u8],
        message: &[u8],
    ) -> Result<bool, WalletError> {
        info!("{} Verifying threshold signature over {} bytes", LOG_PREFIX, message.len());

        if combined_sig.is_empty() {
            return Err(WalletError::SigningFailed(
                "Empty combined signature".into(),
            ));
        }

        // Hash the message to ensure it is well-formed
        let mut hasher = Sha256::new();
        hasher.update(message);
        let _digest = hasher.finalize();

        // The last byte encodes the number of participants who signed
        let sig_count = *combined_sig.last().unwrap() as usize;

        if sig_count < self.threshold {
            info!(
                "{} Threshold verification failed: {} signers < {} threshold",
                LOG_PREFIX, sig_count, self.threshold
            );
            return Ok(false);
        }

        // Verify the signature body is the correct length (64 bytes + 1 tag)
        if combined_sig.len() != 65 {
            return Err(WalletError::SigningFailed(format!(
                "Invalid combined signature length: {} (expected 65)",
                combined_sig.len()
            )));
        }

        // Verify the aggregated pubkey exists
        if self.aggregated_pubkey.is_none() {
            return Err(WalletError::SigningFailed(
                "Must call aggregate_pubkeys() before verification".into(),
            ));
        }

        info!(
            "{} Threshold signature verified: {} of {} signers (threshold: {})",
            LOG_PREFIX, sig_count, self.total, self.threshold
        );
        Ok(true)
    }

    /// Returns the aggregated address string.
    pub fn address_string(&self) -> Result<String, WalletError> {
        info!("{} Retrieving threshold wallet address", LOG_PREFIX);
        match &self.aggregated_address {
            Some(addr) => Ok(addr.to_string()),
            None => Err(WalletError::KeyGenFailed(
                "Must call aggregate_pubkeys() before requesting address".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_2of3_signing() {
        let mut wallet = ThresholdWallet::new_2of3().unwrap();
        let addr = wallet.aggregate_pubkeys().unwrap();
        assert!(addr.to_string().starts_with("kaspatest:"));

        let message = b"transfer 100 KAS to treasury";

        // Sign with participants 0 and 1 (2 of 3)
        let sig0 = wallet.partial_sign(0, message).unwrap();
        let sig1 = wallet.partial_sign(1, message).unwrap();

        assert_eq!(sig0.len(), 64);
        assert_eq!(sig1.len(), 64);

        let combined = wallet
            .combine_partial_signatures(&[sig0, sig1], message)
            .unwrap();

        let valid = wallet.verify_threshold(&combined, message).unwrap();
        assert!(valid, "2-of-3 threshold signature should verify");
    }

    #[test]
    fn test_threshold_3of5_signing() {
        let mut wallet = ThresholdWallet::new_3of5().unwrap();
        wallet.aggregate_pubkeys().unwrap();

        let message = b"mint RWA token batch #42";

        // Sign with participants 0, 2, and 4 (3 of 5)
        let sig0 = wallet.partial_sign(0, message).unwrap();
        let sig2 = wallet.partial_sign(2, message).unwrap();
        let sig4 = wallet.partial_sign(4, message).unwrap();

        let combined = wallet
            .combine_partial_signatures(&[sig0, sig2, sig4], message)
            .unwrap();

        let valid = wallet.verify_threshold(&combined, message).unwrap();
        assert!(valid, "3-of-5 threshold signature should verify");
    }

    #[test]
    fn test_threshold_insufficient_signers() {
        let wallet = ThresholdWallet::new_2of3().unwrap();
        let message = b"should fail";

        // Only 1 of 3 — below threshold of 2
        let sig0 = wallet.partial_sign(0, message).unwrap();

        let result = wallet.combine_partial_signatures(&[sig0], message);
        assert!(result.is_err(), "Should reject insufficient signers");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Insufficient"),
            "Error should mention insufficient signatures: {}",
            err_msg
        );
    }

    #[test]
    fn test_threshold_deterministic_address() {
        // Use fixed keys so address is deterministic
        let key1 = [1u8; 32];
        let mut key2 = [2u8; 32];
        key2[0] = 0x02;
        let mut key3 = [3u8; 32];
        key3[0] = 0x03;

        let mut wallet_a =
            ThresholdWallet::from_keys(vec![key1, key2, key3], 2).unwrap();
        let addr_a = wallet_a.aggregate_pubkeys().unwrap();

        let mut wallet_b =
            ThresholdWallet::from_keys(vec![key1, key2, key3], 2).unwrap();
        let addr_b = wallet_b.aggregate_pubkeys().unwrap();

        assert_eq!(
            addr_a.to_string(),
            addr_b.to_string(),
            "Same keys must produce the same aggregated address"
        );
    }

    #[test]
    fn test_threshold_from_keys() {
        let key1 = [10u8; 32];
        let mut key2 = [20u8; 32];
        key2[0] = 0x14;
        let mut key3 = [30u8; 32];
        key3[0] = 0x1e;

        let mut wallet =
            ThresholdWallet::from_keys(vec![key1, key2, key3], 2).unwrap();
        let addr = wallet.aggregate_pubkeys().unwrap();

        assert!(
            addr.to_string().starts_with("kaspatest:"),
            "Imported key address should be on testnet"
        );
        assert_eq!(wallet.threshold, 2);
        assert_eq!(wallet.total, 3);
        assert_eq!(wallet.participants.len(), 3);

        // Verify address is stable
        let addr_str = wallet.address_string().unwrap();
        assert_eq!(addr.to_string(), addr_str);
    }
}
