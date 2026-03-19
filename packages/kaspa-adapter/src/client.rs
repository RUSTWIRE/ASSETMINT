// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Kaspa Testnet-12 wRPC client.
//! Connects to local kaspad via Borsh-encoded wRPC for UTXO queries and tx broadcast.

use kaspa_addresses::Address;
use kaspa_consensus_core::hashing::sighash::{
    calc_schnorr_signature_hash, SigHashReusedValuesUnsync,
};
use kaspa_consensus_core::hashing::sighash_type::SIG_HASH_ALL;
use kaspa_consensus_core::network::NetworkType;
use kaspa_consensus_core::sign::sign;
use kaspa_consensus_core::subnets::SUBNETWORK_ID_NATIVE;
use kaspa_consensus_core::tx::{
    MutableTransaction, ScriptPublicKey, Transaction, TransactionId, TransactionInput,
    TransactionOutpoint, TransactionOutput, UtxoEntry,
};
use kaspa_rpc_core::api::rpc::RpcApi;
use kaspa_rpc_core::model::tx::RpcTransaction;
use kaspa_txscript::pay_to_address_script;
use kaspa_txscript::script_builder::ScriptBuilder;

use kaspa_wrpc_client::client::{ConnectOptions, ConnectStrategy};
use kaspa_wrpc_client::prelude::NetworkId;
use kaspa_wrpc_client::{KaspaRpcClient, WrpcEncoding};
use secp256k1::Keypair;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tracing::info;

use crate::tx_builder::{self, SpendableUtxo, TransferParams};
use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("[K-RWA] Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("[K-RWA] RPC error: {0}")]
    RpcError(String),
    #[error("[K-RWA] Address parse error: {0}")]
    AddressError(String),
}

/// Unspent transaction output
#[derive(Debug, Clone)]
pub struct Utxo {
    pub txid: String,
    pub index: u32,
    pub amount: u64,
    pub script_pubkey: String,
}

/// Server info from kaspad
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub server_version: String,
    pub is_synced: bool,
    pub virtual_daa_score: u64,
    pub network_id: String,
}

/// Kaspa Testnet-12 RPC client wrapping kaspa-wrpc-client
pub struct KaspaClient {
    endpoint: String,
    rpc: Arc<KaspaRpcClient>,
}

impl KaspaClient {
    /// Create a new client targeting Testnet-12
    pub fn new(endpoint: &str) -> Result<Self, ClientError> {
        info!("{} Initializing Kaspa client: {}", LOG_PREFIX, endpoint);

        let network_id = NetworkId::with_suffix(NetworkType::Testnet, 12);

        let rpc = KaspaRpcClient::new(
            WrpcEncoding::Borsh,
            Some(endpoint),
            None,
            Some(network_id),
            None,
        )
        .map_err(|e| {
            ClientError::ConnectionFailed(format!("Failed to create RPC client: {}", e))
        })?;

        Ok(Self {
            endpoint: endpoint.to_string(),
            rpc: Arc::new(rpc),
        })
    }

    /// Connect to Testnet-12 kaspad
    pub async fn connect(&self) -> Result<(), ClientError> {
        info!("{} Connecting to {}", LOG_PREFIX, self.endpoint);

        let options = ConnectOptions {
            block_async_connect: true,
            connect_timeout: Some(Duration::from_secs(10)),
            strategy: ConnectStrategy::Fallback,
            ..Default::default()
        };

        self.rpc
            .connect(Some(options))
            .await
            .map_err(|e| ClientError::ConnectionFailed(format!("{}", e)))?;

        info!("{} Connected to kaspad", LOG_PREFIX);
        Ok(())
    }

    /// Disconnect from kaspad
    pub async fn disconnect(&self) -> Result<(), ClientError> {
        self.rpc
            .disconnect()
            .await
            .map_err(|e| ClientError::RpcError(format!("{}", e)))?;
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.rpc.is_connected()
    }

    /// Get server info (version, sync status, DAA score)
    pub async fn get_server_info(&self) -> Result<ServerInfo, ClientError> {
        info!("{} Querying server info", LOG_PREFIX);
        let info = self
            .rpc
            .get_server_info()
            .await
            .map_err(|e| ClientError::RpcError(format!("{}", e)))?;

        Ok(ServerInfo {
            server_version: info.server_version,
            is_synced: info.is_synced,
            virtual_daa_score: info.virtual_daa_score,
            network_id: format!("{}", info.network_id),
        })
    }

    /// Get balance for an address (in sompis)
    pub async fn get_balance(&self, address: &str) -> Result<u64, ClientError> {
        info!("{} Querying balance for {}", LOG_PREFIX, address);
        let addr: Address = address
            .try_into()
            .map_err(|e: kaspa_addresses::AddressError| {
                ClientError::AddressError(format!("{}", e))
            })?;

        let balance = self
            .rpc
            .get_balance_by_address(addr)
            .await
            .map_err(|e| ClientError::RpcError(format!("{}", e)))?;

        info!(
            "{} Balance: {} sompis ({:.4} KAS)",
            LOG_PREFIX,
            balance,
            balance as f64 / 1e8
        );
        Ok(balance)
    }

    /// Get UTXOs for an address
    pub async fn get_utxos(&self, address: &str) -> Result<Vec<Utxo>, ClientError> {
        info!("{} Querying UTXOs for {}", LOG_PREFIX, address);
        let addr: Address = address
            .try_into()
            .map_err(|e: kaspa_addresses::AddressError| {
                ClientError::AddressError(format!("{}", e))
            })?;

        let entries = self
            .rpc
            .get_utxos_by_addresses(vec![addr])
            .await
            .map_err(|e| ClientError::RpcError(format!("{}", e)))?;

        let utxos: Vec<Utxo> = entries
            .iter()
            .map(|e| Utxo {
                txid: format!("{}", e.outpoint.transaction_id),
                index: e.outpoint.index,
                amount: e.utxo_entry.amount,
                script_pubkey: hex::encode(e.utxo_entry.script_public_key.script()),
            })
            .collect();

        info!("{} Found {} UTXOs", LOG_PREFIX, utxos.len());
        Ok(utxos)
    }

    /// Get current block DAG info
    pub async fn get_block_dag_info(&self) -> Result<(u64, u64, f64), ClientError> {
        let dag = self
            .rpc
            .get_block_dag_info()
            .await
            .map_err(|e| ClientError::RpcError(format!("{}", e)))?;

        Ok((dag.block_count, dag.virtual_daa_score, dag.difficulty))
    }

    /// Get spendable UTXOs for an address (with full entry data for signing).
    /// Filters out UTXOs that are already being spent by mempool transactions.
    pub async fn get_spendable_utxos(
        &self,
        address: &str,
    ) -> Result<Vec<SpendableUtxo>, ClientError> {
        info!("{} Querying spendable UTXOs for {}", LOG_PREFIX, address);
        let addr: Address = address
            .try_into()
            .map_err(|e: kaspa_addresses::AddressError| {
                ClientError::AddressError(format!("{}", e))
            })?;

        let entries = self
            .rpc
            .get_utxos_by_addresses(vec![addr.clone()])
            .await
            .map_err(|e| ClientError::RpcError(format!("{}", e)))?;

        // Query mempool to find which UTXOs are already being spent
        let mempool_entries = self
            .rpc
            .get_mempool_entries_by_addresses(vec![addr], false, false)
            .await
            .unwrap_or_default();

        // Collect all outpoints being spent by mempool transactions
        let mut mempool_spent: std::collections::HashSet<(
            kaspa_consensus_core::tx::TransactionId,
            u32,
        )> = std::collections::HashSet::new();
        for entry_by_addr in &mempool_entries {
            for mempool_entry in &entry_by_addr.sending {
                for input in &mempool_entry.transaction.inputs {
                    mempool_spent.insert((
                        input.previous_outpoint.transaction_id,
                        input.previous_outpoint.index,
                    ));
                }
            }
        }

        if !mempool_spent.is_empty() {
            info!(
                "{} Excluding {} outpoints spent in mempool",
                LOG_PREFIX,
                mempool_spent.len()
            );
        }

        let utxos: Vec<SpendableUtxo> = entries
            .iter()
            .filter(|e| !mempool_spent.contains(&(e.outpoint.transaction_id, e.outpoint.index)))
            .map(|e| SpendableUtxo {
                txid: e.outpoint.transaction_id,
                index: e.outpoint.index,
                amount: e.utxo_entry.amount,
                script_public_key: e.utxo_entry.script_public_key.clone(),
            })
            .collect();

        info!(
            "{} Found {} spendable UTXOs (after mempool filter)",
            LOG_PREFIX,
            utxos.len()
        );
        Ok(utxos)
    }

    /// Submit a signed transaction to the network
    pub async fn submit_transaction(&self, tx: &Transaction) -> Result<TransactionId, ClientError> {
        info!("{} Broadcasting transaction {}", LOG_PREFIX, tx.id());

        let rpc_tx = RpcTransaction::from(tx);
        let tx_id = self
            .rpc
            .submit_transaction(rpc_tx, false)
            .await
            .map_err(|e| ClientError::RpcError(format!("Submit failed: {}", e)))?;

        info!("{} Transaction accepted: {}", LOG_PREFIX, tx_id);
        Ok(tx_id)
    }

    /// Send KAS from one address to another (full flow: select UTXOs → build → sign → broadcast)
    pub async fn send_kas(
        &self,
        from_address: &str,
        to_address: &str,
        amount_sompis: u64,
        keypair: &Keypair,
        op_return_data: Option<Vec<u8>>,
    ) -> Result<TransactionId, ClientError> {
        info!(
            "{} Sending {} sompis from {} to {}",
            LOG_PREFIX, amount_sompis, from_address, to_address
        );

        // 1. Get spendable UTXOs
        let utxos = self.get_spendable_utxos(from_address).await?;
        if utxos.is_empty() {
            return Err(ClientError::RpcError("No UTXOs available".to_string()));
        }

        // 2. Parse addresses
        let to_addr: Address =
            to_address
                .try_into()
                .map_err(|e: kaspa_addresses::AddressError| {
                    ClientError::AddressError(format!("{}", e))
                })?;
        let change_addr: Address =
            from_address
                .try_into()
                .map_err(|e: kaspa_addresses::AddressError| {
                    ClientError::AddressError(format!("{}", e))
                })?;

        // 3. Build unsigned transaction
        let params = TransferParams {
            to_address: to_addr,
            amount: amount_sompis,
            change_address: change_addr,
            op_return_data,
        };

        let (unsigned_tx, utxo_entries) = tx_builder::build_transfer(&utxos, &params)
            .map_err(|e| ClientError::RpcError(format!("Build failed: {}", e)))?;

        // 4. Create signable transaction (MutableTransaction with populated UTXO entries)
        let signable = MutableTransaction::with_entries(
            unsigned_tx,
            utxo_entries
                .into_iter()
                .map(|(_outpoint, entry)| kaspa_consensus_core::tx::UtxoEntry {
                    amount: entry.amount,
                    script_public_key: entry.script_public_key,
                    block_daa_score: entry.block_daa_score,
                    is_coinbase: entry.is_coinbase,
                    covenant_id: entry.covenant_id,
                })
                .collect(),
        );

        // 5. Sign with Schnorr
        info!("{} Signing transaction with Schnorr keypair", LOG_PREFIX);
        let signed = sign(signable, *keypair);

        // 6. Submit
        let tx_id = self.submit_transaction(&signed.tx).await?;

        info!("{} Transfer complete! TX: {}", LOG_PREFIX, tx_id);

        Ok(tx_id)
    }

    /// Deploy a SilverScript contract by funding its P2SH address
    ///
    /// Creates a UTXO locked by the covenant's P2SH script.
    /// The contract is "live" once this transaction confirms.
    pub async fn deploy_contract(
        &self,
        from_address: &str,
        contract: &crate::script::CompiledContract,
        funding_sompis: u64,
        keypair: &Keypair,
    ) -> Result<TransactionId, ClientError> {
        info!(
            "{} Deploying contract '{}' with {} sompis to P2SH {}",
            LOG_PREFIX, contract.contract_name, funding_sompis, contract.p2sh_address
        );

        // Fund the P2SH address — this creates the covenant UTXO
        let p2sh_addr_str = contract.p2sh_address.to_string();
        self.send_kas(from_address, &p2sh_addr_str, funding_sompis, keypair, None)
            .await
    }

    /// Commit a compliance audit hash to the Kaspa DAG via OP_RETURN.
    /// Creates a minimal self-send transaction with the hash embedded in the output.
    pub async fn commit_audit_hash(
        &self,
        from_address: &str,
        audit_hash: [u8; 32],
        keypair: &Keypair,
    ) -> Result<TransactionId, ClientError> {
        info!(
            "{} Committing audit hash to DAG: {}",
            LOG_PREFIX,
            hex::encode(&audit_hash)
        );
        // Send minimal amount (1000 sompis) to self with audit hash as OP_RETURN
        self.send_kas(
            from_address,
            from_address,
            1000,
            keypair,
            Some(audit_hash.to_vec()),
        )
        .await
    }

    /// Consolidate fragmented UTXOs into fewer, larger outputs.
    /// Sends batches of MAX_INPUTS UTXOs to self, waiting between batches.
    /// Handles storage mass errors by reducing the send amount and retrying.
    /// Consolidate fragmented UTXOs by sending small batches to self.
    /// Uses only a subset of UTXOs per batch to stay under storage mass limit.
    pub async fn consolidate_utxos(
        &self,
        address: &str,
        keypair: &Keypair,
    ) -> Result<usize, ClientError> {
        info!("{} Consolidating UTXOs for {}", LOG_PREFIX, address);
        let mut total_consolidated = 0;
        // Use small batches (10 UTXOs) to stay well under mass limit
        const CONSOLIDATION_BATCH: usize = 10;

        loop {
            let utxos = self.get_spendable_utxos(address).await?;
            if utxos.len() <= 3 {
                info!(
                    "{} Consolidation complete: {} UTXOs remaining",
                    LOG_PREFIX,
                    utxos.len()
                );
                break;
            }

            // Sort largest-first, take only CONSOLIDATION_BATCH
            let mut sorted = utxos.clone();
            sorted.sort_by(|a, b| b.amount.cmp(&a.amount));
            let batch: Vec<_> = sorted.iter().take(CONSOLIDATION_BATCH).collect();
            let batch_total: u64 = batch.iter().map(|u| u.amount).sum();
            let fee_estimate = (batch.len() as u64) * 27_000 + 5_000;

            if batch_total <= fee_estimate + 10_000 {
                info!("{} Batch too small for consolidation", LOG_PREFIX);
                break;
            }

            // Send just under what this batch can cover — coin selection will pick these
            let send_amount = batch_total - fee_estimate;
            info!(
                "{} Consolidation batch: {} of {} UTXOs, {} sompis → {} sompis",
                LOG_PREFIX,
                batch.len(),
                utxos.len(),
                batch_total,
                send_amount
            );

            match self
                .send_kas(address, address, send_amount, keypair, None)
                .await
            {
                Ok(tx_id) => {
                    info!("{} Consolidation TX: {}", LOG_PREFIX, tx_id);
                    total_consolidated += 1;
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                }
                Err(e) => {
                    info!("{} Consolidation batch failed: {}", LOG_PREFIX, e);
                    break;
                }
            }
        }

        Ok(total_consolidated)
    }

    /// Get the inner RPC client (for advanced operations)
    pub fn rpc(&self) -> &KaspaRpcClient {
        &self.rpc
    }

    /// Spend a P2SH UTXO by providing witness parameters and the redeem script.
    ///
    /// This invokes a SilverScript covenant entrypoint. The scriptSig is built as:
    ///   `[param1_push] [param2_push] ... [redeem_script_push]`
    /// where each element is canonically length-prefixed using ScriptBuilder::add_data.
    ///
    /// # P2SH execution model (from rusty-kaspa txscript)
    ///
    /// 1. The signature_script (scriptSig) is executed with verify_only_push=true
    ///    — all elements are pushed to the stack.
    /// 2. The script_public_key (`OP_BLAKE2B OP_DATA32 <hash> OP_EQUAL`) is executed,
    ///    which blake2b-hashes the top stack element (the redeem script) and checks
    ///    it equals the embedded hash.
    /// 3. The stack is restored to its state after step 1, the top element (redeem
    ///    script) is popped and executed as a third script — the covenant logic.
    ///
    /// # Arguments
    /// * `p2sh_utxo_txid` - Transaction ID of the P2SH UTXO
    /// * `p2sh_utxo_index` - Output index of the P2SH UTXO
    /// * `p2sh_utxo_amount` - Amount (in sompis) locked in the P2SH UTXO
    /// * `p2sh_script_public_key` - The P2SH locking script (OP_BLAKE2B OP_DATA32 <hash> OP_EQUAL)
    /// * `redeem_script` - The compiled covenant bytecode (raw bytes)
    /// * `witness_params` - Stack parameters for the entrypoint, in push order.
    ///   Each param is raw bytes; this function wraps them with canonical push opcodes.
    /// * `outputs` - Transaction outputs to create (must satisfy covenant require() checks)
    /// * `sig_op_count` - Number of signature operations in the redeem script path
    ///   being invoked (e.g. 1 for a single checkSig)
    /// * `keypair` - Keypair for Schnorr signing (used to produce the witness signature)
    pub async fn spend_p2sh(
        &self,
        p2sh_utxo_txid: TransactionId,
        p2sh_utxo_index: u32,
        p2sh_utxo_amount: u64,
        p2sh_script_public_key: ScriptPublicKey,
        redeem_script: &[u8],
        witness_params: Vec<Vec<u8>>,
        outputs: Vec<TransactionOutput>,
        sig_op_count: u8,
        keypair: &Keypair,
    ) -> Result<TransactionId, ClientError> {
        info!(
            "{} Spending P2SH UTXO {}:{} ({} sompis) with {} witness params",
            LOG_PREFIX,
            p2sh_utxo_txid,
            p2sh_utxo_index,
            p2sh_utxo_amount,
            witness_params.len()
        );

        // Build the signature_script (scriptSig) for P2SH spending.
        //
        // Format: [witness_param1] [witness_param2] ... [redeem_script]
        //
        // BUT: we need to sign the transaction first to produce the signature,
        // which is itself one of the witness params. So we do a two-pass approach:
        //
        // Pass 1: Build the transaction with an empty signature_script, compute sighash
        // Pass 2: Sign, build the real signature_script, update the input

        let outpoint = TransactionOutpoint::new(p2sh_utxo_txid, p2sh_utxo_index);
        let input = TransactionInput::new(
            outpoint,
            vec![], // Placeholder — filled after signing
            0,      // sequence
            sig_op_count,
        );

        let tx = Transaction::new(
            0,                    // version
            vec![input],          // inputs
            outputs,              // outputs
            0,                    // lock_time
            SUBNETWORK_ID_NATIVE, // subnetwork_id
            0,                    // gas
            vec![],               // payload
        );

        // Create a signable transaction with the P2SH UTXO entry.
        // The sighash includes the script_public_key of the UTXO being spent,
        // which for P2SH is the hash-wrapper script, NOT the redeem script.
        let utxo_entry = UtxoEntry {
            amount: p2sh_utxo_amount,
            script_public_key: p2sh_script_public_key,
            block_daa_score: 0,
            is_coinbase: false,
            covenant_id: None,
        };

        let signable = MutableTransaction::with_entries(tx, vec![utxo_entry]);

        // Compute the sighash for input 0
        let reused_values = SigHashReusedValuesUnsync::new();
        let sig_hash =
            calc_schnorr_signature_hash(&signable.as_verifiable(), 0, SIG_HASH_ALL, &reused_values);

        // Sign the sighash with Schnorr
        let msg = secp256k1::Message::from_digest_slice(sig_hash.as_bytes().as_slice())
            .map_err(|e| ClientError::RpcError(format!("Sighash message error: {}", e)))?;
        let sig: [u8; 64] = *keypair.sign_schnorr(msg).as_ref();

        // Build the Schnorr signature with sighash type as a push-data element:
        // OP_DATA_65 <64-byte sig> <SIGHASH_ALL>
        let sig_with_hashtype: Vec<u8> = std::iter::once(65u8)
            .chain(sig.iter().copied())
            .chain(std::iter::once(SIG_HASH_ALL.to_u8()))
            .collect();

        // Now build the complete signature_script.
        //
        // The witness_params may contain a placeholder for the signature.
        // The caller is responsible for inserting the signature into the
        // correct position in witness_params. For flexibility, this function
        // takes the already-built witness_params and appends the redeem script.
        //
        // However, we need a special approach: the caller passes in raw bytes
        // for each witness param, but the FIRST param that is a signature
        // needs to be the actual Schnorr signature we just computed.
        //
        // Design decision: The caller provides witness_params where each entry
        // is the raw bytes to push. If a param is empty (zero-length), it is
        // replaced with the computed Schnorr signature.

        let mut script_sig = Vec::new();

        for param in &witness_params {
            if param.is_empty() {
                // Empty param = placeholder for Schnorr signature.
                // Push the raw signature bytes (already includes OP_DATA_65 prefix).
                script_sig.extend_from_slice(&sig_with_hashtype);
            } else if param.len() == 1 && param[0] <= 16 {
                // MINIMALDATA encoding for small integers (function selectors).
                // 0 → OP_FALSE (0x00), 1-16 → OP_1..OP_16 (0x51..0x60)
                if param[0] == 0 {
                    script_sig.push(0x00); // OP_FALSE
                } else {
                    script_sig.push(0x50 + param[0]); // OP_1..OP_16
                }
            } else {
                // Canonically encode the param as a push-data element.
                let push = ScriptBuilder::new()
                    .add_data(param)
                    .map_err(|e| ClientError::RpcError(format!("Script builder error: {}", e)))?
                    .drain();
                script_sig.extend_from_slice(&push);
            }
        }

        // Append the redeem script as the final push-data element.
        let redeem_push = ScriptBuilder::new()
            .add_data(redeem_script)
            .map_err(|e| ClientError::RpcError(format!("Redeem script push error: {}", e)))?
            .drain();
        script_sig.extend_from_slice(&redeem_push);

        info!(
            "{} Built P2SH signature_script: {} bytes ({} witness params + redeem script)",
            LOG_PREFIX,
            script_sig.len(),
            witness_params.len()
        );

        // Update the transaction with the real signature_script
        let mut final_tx = signable.tx;
        final_tx.inputs[0].signature_script = script_sig;

        // Submit to the network
        let tx_id = self.submit_transaction(&final_tx).await?;

        info!("{} P2SH covenant spend complete! TX: {}", LOG_PREFIX, tx_id);

        Ok(tx_id)
    }

    /// Helper: Spend a P2SH UTXO using a CompiledContract and a simplified interface.
    ///
    /// Looks up the UTXO at the contract's P2SH address, builds appropriate outputs,
    /// and calls `spend_p2sh()`.
    ///
    /// # Arguments
    /// * `contract` - The compiled SilverScript contract
    /// * `witness_params` - Witness parameters for the entrypoint. Use an empty vec
    ///   as a placeholder for the Schnorr signature (it will be computed and inserted).
    /// * `output_address` - Where to send the funds
    /// * `output_amount` - How much to send (must satisfy covenant's require() checks)
    /// * `sig_op_count` - Number of checkSig ops in the invoked entrypoint
    /// * `keypair` - Keypair for signing
    pub async fn spend_contract(
        &self,
        contract: &crate::script::CompiledContract,
        witness_params: Vec<Vec<u8>>,
        output_address: &str,
        output_amount: u64,
        sig_op_count: u8,
        keypair: &Keypair,
    ) -> Result<TransactionId, ClientError> {
        info!(
            "{} Invoking covenant '{}' -> {} ({} sompis)",
            LOG_PREFIX, contract.contract_name, output_address, output_amount
        );

        // Find the P2SH UTXO
        let p2sh_addr_str = contract.p2sh_address.to_string();
        let utxos = self.get_utxos(&p2sh_addr_str).await?;
        if utxos.is_empty() {
            return Err(ClientError::RpcError(format!(
                "No UTXOs found at P2SH address {} for contract '{}'",
                p2sh_addr_str, contract.contract_name
            )));
        }

        // Use the first (largest) UTXO
        let utxo = &utxos[0];
        let txid: TransactionId = utxo
            .txid
            .parse()
            .map_err(|e| ClientError::RpcError(format!("Invalid txid: {}", e)))?;

        // Build the output
        let out_addr: Address =
            output_address
                .try_into()
                .map_err(|e: kaspa_addresses::AddressError| {
                    ClientError::AddressError(format!("{}", e))
                })?;
        let out_script = pay_to_address_script(&out_addr);
        let outputs = vec![TransactionOutput::new(output_amount, out_script)];

        self.spend_p2sh(
            txid,
            utxo.index,
            utxo.amount,
            contract.script_public_key.clone(),
            &contract.redeem_script,
            witness_params,
            outputs,
            sig_op_count,
            keypair,
        )
        .await
    }
}
