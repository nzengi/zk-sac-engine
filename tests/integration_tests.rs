use std::time::Duration;
use tokio::time::timeout;
use zk_sac_engine::{
    consensus::{ConsensusEngine, BeamChainConfig},
    types::{Block, Transaction, Address, BlockHash, ValidatorSet},
    crypto::{hash::MultiHasher, signatures::QuantumResistantSigner},
    zkvm::Risc0Executor,
    async_utils::{AsyncTaskPool, BatchProcessor, ConsensusCoordinator},
};
#[cfg(feature = "risc0")]
use risc0_zkvm::{default_prover, ExecutorEnv};

#[tokio::test]
async fn test_full_consensus_flow() {
    // Test complete consensus cycle: block production -> validation -> finalization
    let config = BeamChainConfig::new_for_testing();
    let mut engine = ConsensusEngine::new(config.clone()).await.expect("Failed to create engine");
    
    // Create test transactions
    let transactions = create_test_transactions(10).await;
    
    // Process block production
    let block = engine.produce_block(transactions).await.expect("Block production failed");
    
    // Validate the block
    let is_valid = engine.validate_block(&block).await.expect("Block validation failed");
    assert!(is_valid, "Block should be valid");
    
    // Test finalization
    engine.finalize_block(block.hash).await.expect("Block finalization failed");
    
    // Verify chain state
    let chain_state = engine.get_chain_state().await;
    assert_eq!(chain_state.latest_block_height, 1);
}

#[cfg(feature = "risc0")]
#[tokio::test]
async fn test_zkvm_proof_generation_and_verification() {
    // Test complete ZK proof lifecycle
    let executor = Risc0Executor::new().expect("Failed to create Risc0 executor");
    
    // Create state transition input
    let prev_state = create_test_state();
    let transactions = create_test_transactions(5).await;
    
    // Generate proof
    let proof_result = timeout(
        Duration::from_secs(30),
        executor.generate_state_transition_proof(prev_state, transactions)
    ).await;
    
    assert!(proof_result.is_ok(), "Proof generation should complete within timeout");
    let proof = proof_result.unwrap().expect("Proof generation should succeed");
    
    // Verify proof
    let verification_result = executor.verify_proof(&proof).await;
    assert!(verification_result.is_ok(), "Proof verification should succeed");
    assert!(verification_result.unwrap(), "Proof should be valid");
}

#[tokio::test]
async fn test_multi_hash_security() {
    // Test all three hash algorithms for consistency and security
    let hasher = MultiHasher::new();
    let test_data = b"test_blockchain_data_for_hashing";
    
    // Test Blake3 (performance)
    let blake3_hash = hasher.blake3_hash(test_data);
    assert_eq!(blake3_hash.len(), 32);
    
    // Test Keccak256 (EVM compatibility)
    let keccak_hash = hasher.keccak256_hash(test_data);
    assert_eq!(keccak_hash.len(), 32);
    
    // Test SHA3-256 (post-quantum security)
    let sha3_hash = hasher.sha3_256_hash(test_data);
    assert_eq!(sha3_hash.len(), 32);
    
    // Verify deterministic behavior
    let blake3_hash2 = hasher.blake3_hash(test_data);
    assert_eq!(blake3_hash, blake3_hash2);
    
    // Verify different algorithms produce different hashes
    assert_ne!(blake3_hash, keccak_hash);
    assert_ne!(blake3_hash, sha3_hash);
    assert_ne!(keccak_hash, sha3_hash);
}

#[tokio::test]
async fn test_async_coordination_under_load() {
    // Test async utilities under high concurrency
    let pool = AsyncTaskPool::new(10);
    let coordinator = ConsensusCoordinator::new();
    let batch_processor = BatchProcessor::new(100);
    
    // Simulate high-load scenario
    let mut tasks = Vec::new();
    
    for i in 0..50 {
        let task = pool.spawn(async move {
            // Simulate variable processing time
            tokio::time::sleep(Duration::from_millis(i % 10)).await;
            format!("task_{}", i)
        });
        tasks.push(task);
    }
    
    // Wait for all tasks with timeout
    let results = timeout(
        Duration::from_secs(5),
        futures::future::join_all(tasks)
    ).await;
    
    assert!(results.is_ok(), "All tasks should complete within timeout");
    let task_results = results.unwrap();
    assert_eq!(task_results.len(), 50);
    
    // Verify task results
    for (i, result) in task_results.iter().enumerate() {
        assert!(result.is_ok());
        assert_eq!(result.as_ref().unwrap(), &format!("task_{}", i));
    }
}

#[tokio::test]
async fn test_validator_set_management() {
    // Test validator set operations and consensus participation
    let config = BeamChainConfig::new_for_testing();
    let mut engine = ConsensusEngine::new(config).await.expect("Failed to create engine");
    
    // Create test validators
    let validators = create_test_validators(5).await;
    
    // Update validator set
    engine.update_validator_set(validators.clone()).await.expect("Validator set update failed");
    
    // Test validator selection
    let selected = engine.select_block_producer().await.expect("Validator selection failed");
    assert!(validators.contains(&selected), "Selected validator should be in set");
    
    // Test slashing mechanism
    let validator_to_slash = validators[0].clone();
    engine.slash_validator(validator_to_slash.address, 1000).await.expect("Slashing failed");
    
    // Verify validator stake was reduced
    let updated_set = engine.get_validator_set().await;
    let slashed_validator = updated_set.validators.iter()
        .find(|v| v.address == validator_to_slash.address)
        .expect("Validator should still exist");
    assert!(slashed_validator.stake < validator_to_slash.stake);
}

#[tokio::test]
async fn test_evm_compatibility() {
    // Test EVM-compatible features
    let hasher = MultiHasher::new();
    
    // Test Keccak256 address generation (Ethereum style)
    let public_key = create_test_public_key();
    let address = hasher.generate_evm_address(&public_key);
    
    // Verify address format (20 bytes, 0x prefixed when displayed)
    assert_eq!(address.as_bytes().len(), 20);
    
    // Test transaction hash compatibility
    let transaction = create_test_evm_transaction();
    let tx_hash = hasher.hash_evm_transaction(&transaction);
    assert_eq!(tx_hash.len(), 32);
    
    // Test hex encoding/decoding
    let hex_address = format!("0x{}", hex::encode(address.as_bytes()));
    assert!(hex_address.starts_with("0x"));
    assert_eq!(hex_address.len(), 42); // 0x + 40 hex chars
}

#[tokio::test]
async fn test_error_recovery_and_resilience() {
    // Test system behavior under various failure conditions
    let config = BeamChainConfig::new_for_testing();
    let mut engine = ConsensusEngine::new(config).await.expect("Failed to create engine");
    
    // Test invalid block handling
    let invalid_block = create_invalid_test_block();
    let validation_result = engine.validate_block(&invalid_block).await;
    assert!(validation_result.is_ok()); // Should handle gracefully
    assert!(!validation_result.unwrap()); // But should return false
    
    // Test network partition recovery
    engine.handle_network_partition().await.expect("Network partition handling failed");
    
    // Test state corruption recovery
    let recovery_result = engine.recover_from_corruption().await;
    assert!(recovery_result.is_ok(), "Should recover from corruption");
}

// Helper functions for test data generation

async fn create_test_transactions(count: usize) -> Vec<Transaction> {
    let mut transactions = Vec::new();
    for i in 0..count {
        transactions.push(Transaction {
            from: Address::random(),
            to: Some(Address::random()),
            value: 1000 + i as u64,
            data: vec![],
            nonce: i as u64,
            gas_limit: 21000,
            gas_price: 20,
            signature: vec![0; 64], // Mock signature
        });
    }
    transactions
}

fn create_test_state() -> Vec<u8> {
    // Mock previous state
    vec![0; 32]
}

async fn create_test_validators(count: usize) -> ValidatorSet {
    use zk_sac_engine::types::Validator;
    
    let mut validators = Vec::new();
    for i in 0..count {
        validators.push(Validator {
            address: Address::random(),
            stake: 1000000 + (i as u64 * 100000),
            public_key: vec![i as u8; 32],
        });
    }
    
    ValidatorSet { validators }
}

fn create_test_public_key() -> Vec<u8> {
    vec![0x04; 65] // Mock uncompressed public key
}

fn create_test_evm_transaction() -> Transaction {
    Transaction {
        from: Address::random(),
        to: Some(Address::random()),
        value: 1000,
        data: vec![0x60, 0x60, 0x60, 0x40], // Mock EVM bytecode
        nonce: 0,
        gas_limit: 21000,
        gas_price: 20,
        signature: vec![0; 65], // Mock signature with recovery byte
    }
}

fn create_invalid_test_block() -> Block {
    Block {
        hash: BlockHash::zero(),
        parent_hash: BlockHash::zero(),
        height: u64::MAX, // Invalid height
        timestamp: 0, // Invalid timestamp
        transactions: vec![],
        state_root: vec![],
        validator: Address::zero(),
        signature: vec![],
        zk_proof: None,
    }
} 