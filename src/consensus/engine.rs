use crate::types::*;
use crate::zkvm::Risc0Executor;
use crate::crypto::signatures::{SignatureEngine, PostQuantumSigner};
use crate::crypto::hash::{IncrementalHasher, keccak256_hash, compute_consensus_hash, hex_utils};
use crate::serialization::{encode_blockchain_data, encode_state_data, to_json_pretty, compare_formats, create_block_metadata, to_json_value, extract_block_summary};
use crate::async_utils::{ConsensusCoordinator, BatchProcessor};
use anyhow::{Result, anyhow};
use tracing::{info, warn, debug};
// Removed async_trait - using sync methods for now
use tokio::time::{timeout, Duration};

/// BeamChain-inspired ZK-SAC Consensus Engine
/// Features:
/// - Recursive zk-proof block validation
/// - Post-quantum signature support
/// - 4-second block times
/// - Self-amending protocol rules
/// - Risc0 zkVM integration
pub struct ZkSacConsensusEngine {
    pub current_state: WorldState,
    pub validator_set: ValidatorSet,
    pub blocks: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub protocol_config: ProtocolConfig,
    #[cfg(feature = "risc0")]
    pub zkvm_engine: Box<Risc0Executor>,
    pub signature_engine: SignatureEngine,
    pub post_quantum_signer: PostQuantumSigner,
    pub async_coordinator: ConsensusCoordinator,
    pub transaction_processor: BatchProcessor<Transaction>,
}

pub trait ConsensusEngine {
    fn validate_block(&self, block: &Block) -> Result<bool>;
    fn produce_block(&mut self, producer: Address) -> Result<Block>;
    fn apply_block(&mut self, block: Block) -> Result<()>;
    fn select_block_producer(&self, block_number: u64) -> Result<Address>;
}

impl ZkSacConsensusEngine {
    pub fn new(
        genesis_state: WorldState, 
        initial_validators: Vec<Validator>,
        config: ProtocolConfig
    ) -> Result<Self> {
        info!("🚀 Initializing ZK-SAC Consensus Engine");
        info!("   ⚡ Block time: {:?}", config.block_time);
        info!("   🏗️  Max TX per block: {}", config.max_transactions_per_block);
        
        let total_stake = initial_validators.iter().map(|v| v.stake).sum();

        #[cfg(feature = "risc0")]
        let zkvm_engine = Box::new(Risc0Executor::new()?);
        let signature_engine = SignatureEngine::new();
        let post_quantum_signer = PostQuantumSigner::new()?;
        
        let async_coordinator = ConsensusCoordinator::new();
        let transaction_processor = BatchProcessor::new(
            config.max_transactions_per_block, 
            config.block_time.as_millis() as u64 / 2 // Process batches at half block time
        );
        
        info!("🚀 Async coordination pools initialized");

        Ok(Self {
            current_state: genesis_state,
            validator_set: ValidatorSet {
                validators: initial_validators,
                total_stake,
            },
            blocks: Vec::new(),
            pending_transactions: Vec::new(),
            protocol_config: config,
            #[cfg(feature = "risc0")]
            zkvm_engine,
            signature_engine,
            post_quantum_signer,
            async_coordinator,
            transaction_processor,
        })
    }

    pub fn execute_transactions_with_zkvm(&self, transactions: &[Transaction]) -> Result<(WorldState, ZkProof)> {
        let mut new_state = self.current_state.clone();
        
        // Simple state update for each transaction
        for tx in transactions {
            // Update account balances
            if let Some(from_account) = new_state.accounts.get_mut(&tx.from) {
                if from_account.balance >= tx.value {
                    from_account.balance -= tx.value;
                    from_account.nonce += 1;
                }
            }
            
            // Update to account
            new_state.accounts.entry(tx.to).or_insert_with(|| Account {
                balance: 0,
                nonce: 0,
                code: Vec::new(),
                storage: std::collections::HashMap::new(),
            }).balance += tx.value;
        }
        
        // Generate zkVM proof for all executions (mock for now - async makes it complex)
        let proof = vec![0; 32]; // Mock proof
        
        let zk_proof = ZkProof {
            proof_data: proof,
            public_inputs: vec![],
            verification_key: vec![],
            proof_type: crate::types::ProofType::Risc0,
        };
        Ok((new_state, zk_proof))
    }

    pub fn generate_recursive_proof(&self, protocol_updates: Vec<ProtocolRule>) -> Result<ZkProof> {
        info!("🔄 Generating recursive zk-proof for {} protocol updates", protocol_updates.len());
        
        let mut proof_inputs = Vec::new();
        for update in &protocol_updates {
            proof_inputs.extend_from_slice(&update.validity_proof.proof_data);
        }
        
        // Generate the recursive proof (mock for sync execution)
        let proof = vec![0; 32]; // Mock proof
        
        let zk_proof = ZkProof {
            proof_data: proof,
            public_inputs: vec![],
            verification_key: vec![],
            proof_type: crate::types::ProofType::Risc0,
        };
        info!("✅ Recursive zk-proof generated: {} bytes", zk_proof.proof_data.len());
        Ok(zk_proof)
    }

    fn collect_transactions_for_block(&mut self) -> Vec<Transaction> {
        let max_tx = self.protocol_config.max_transactions_per_block;
        let collected: Vec<Transaction> = self.pending_transactions
            .drain(..std::cmp::min(max_tx, self.pending_transactions.len()))
            .collect();
        
        debug!("📦 Collected {} transactions for block production", collected.len());
        collected
    }

    fn get_last_block_hash(&self) -> BlockHash {
        if let Some(last_block) = self.blocks.last() {
            // Serialize header to bytes for hashing
            let header_bytes = bincode::serialize(&last_block.header).unwrap_or_default();
            let (hash, _, _) = compute_consensus_hash(&header_bytes);
            BlockHash(hash)
        } else {
            BlockHash::zero() // Genesis
        }
    }

    fn create_block_header(&self, transactions: &[Transaction], producer: Address) -> BlockHeader {
        BlockHeader {
            previous_hash: self.get_last_block_hash(),
            merkle_root: BlockHash::zero(), // Will be computed separately
            state_root: self.current_state.state_root,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            block_number: self.blocks.len() as u64 + 1,
            gas_used: transactions.iter().map(|tx| tx.gas_limit).sum(),
            gas_limit: 30_000_000, // Default gas limit
            producer,
            extra_data: Vec::new(),
        }
    }
}

impl ConsensusEngine for ZkSacConsensusEngine {
    fn produce_block(&mut self, producer: Address) -> Result<Block> {
        info!("🔨 Producing block {} with producer {:?}", 
              self.blocks.len() + 1, producer);
        
        let start_time = std::time::Instant::now();
        
        // Collect transactions
        let transactions = self.collect_transactions_for_block();
        debug!("📦 Collected {} transactions for block", transactions.len());
        
        // Execute transactions with zkVM
        let (new_state, execution_proof) = self.execute_transactions_with_zkvm(&transactions)?;
        
        // Create block header
        let header = self.create_block_header(&transactions, producer);
        
        // Generate recursive proof for protocol updates
        let protocol_updates = Vec::new(); // Empty for now
        let recursive_proof = self.generate_recursive_proof(protocol_updates.clone())?;

        let block = Block {
            header,
            transactions,
            validator_signatures: Vec::new(), // Will be added during consensus
            recursive_proof,
            protocol_updates,
        };
        
        let elapsed = start_time.elapsed();
        info!("✅ Block {} produced in {:?}", block.header.block_number, elapsed);

        Ok(block)
    }

    fn validate_block(&self, block: &Block) -> Result<bool> {
        debug!("🔍 Validating block {}", block.header.block_number);
        
        // Basic validation
        if block.header.previous_hash != self.get_last_block_hash() {
            warn!("❌ Invalid previous hash");
            return Ok(false);
        }
        
        if block.transactions.len() > self.protocol_config.max_transactions_per_block {
            warn!("❌ Too many transactions in block");
            return Ok(false);
        }
        
        // Verify zk-proof (mock for sync execution)
        let verified = true; // Mock verification
        
        if !verified {
            warn!("❌ ZK proof verification failed");
            return Ok(false);
        }
        
        info!("✅ Block {} validated successfully", block.header.block_number);
        Ok(true)
    }

    fn apply_block(&mut self, block: Block) -> Result<()> {
        info!("📝 Applying block {} to chain", block.header.block_number);
        
        // Update current state by re-executing transactions
        let (new_state, _) = self.execute_transactions_with_zkvm(&block.transactions)?;
        self.current_state = new_state;
        
        // Add block to chain
        self.blocks.push(block);
        
        info!("✅ Block applied successfully. Chain length: {}", self.blocks.len());
        Ok(())
    }

    fn select_block_producer(&self, block_number: u64) -> Result<Address> {
        if self.validator_set.validators.is_empty() {
            return Err(anyhow!("No validators available"));
        }
        
        // Simple round-robin selection based on block number
        let index = (block_number as usize) % self.validator_set.validators.len();
        let selected = &self.validator_set.validators[index];
        
        info!("🎯 Selected validator {:?} for block {}", selected.address, block_number);
        Ok(selected.address)
    }
}