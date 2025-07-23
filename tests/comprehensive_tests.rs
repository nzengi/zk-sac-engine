use zk_sac_engine::consensus::engine::{ZkSacConsensusEngine, ConsensusEngine};
use zk_sac_engine::types::*;
use zk_sac_engine::zkvm::real_proofs::{RealZKProver, ZKProofResult};
use zk_sac_engine::performance::{PerformanceMonitor, PerformanceTest};
use std::collections::HashMap;
use tokio::time::{timeout, Duration};
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn test_real_zk_proof_generation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Real ZK Proof Generation");
    
    let prover = RealZKProver::new()?;
    
    // Create test transactions
    let transactions = vec![
        Transaction {
            from: Address::new(1),
            to: Address::new(2),
            value: 1000,
            data: vec![0x01, 0x02, 0x03],
            gas_limit: 21000,
            nonce: 0,
            signature: vec![0; 64],
            sig_type: SignatureType::Ed25519,
        },
        Transaction {
            from: Address::new(2),
            to: Address::new(3),
            value: 500,
            data: vec![0x04, 0x05, 0x06],
            gas_limit: 21000,
            nonce: 1,
            signature: vec![0; 64],
            sig_type: SignatureType::Ed25519,
        },
    ];
    
    let prev_state_root = BlockHash::zero();
    let block_number = 1;
    let timestamp = 1640995200; // 2022-01-01
    
    // Generate proof with timeout
    let proof_result = timeout(
        Duration::from_secs(30),
        prover.generate_state_transition_proof(
            prev_state_root,
            &transactions,
            block_number,
            timestamp,
        ),
    ).await??;
    
    // Verify proof
    let is_valid = prover.verify_proof(&proof_result).await?;
    
    assert!(is_valid, "ZK proof should be valid");
    assert!(proof_result.proof_size > 0, "Proof should have non-zero size");
    assert_eq!(proof_result.public_outputs.transaction_count, 2);
    assert!(proof_result.public_outputs.success);
    
    println!("✅ Real ZK proof test passed");
    println!("   📏 Proof size: {} bytes", proof_result.proof_size);
    println!("   ⏱️  Generation time: {} ms", proof_result.generation_time_ms);
    
    Ok(())
}

#[tokio::test]
#[traced_test]
async fn test_consensus_engine_with_performance_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Consensus Engine with Performance Monitoring");
    
    let mut monitor = PerformanceMonitor::new();
    
    // Setup consensus engine
    let genesis_state = create_test_genesis_state();
    let validators = create_test_validators();
    let config = ProtocolConfig::default();
    
    let mut engine = ZkSacConsensusEngine::new(genesis_state, validators, config)?;
    
    // Create test transactions
    let transactions = create_large_transaction_set(100);
    for tx in transactions {
        engine.pending_transactions.push(tx);
    }
    
    // Test multiple block production cycles
    for block_num in 1..=5 {
        monitor.start_timer("full_cycle");
        
        // Select producer
        monitor.start_timer("producer_selection");
        let producer = engine.select_block_producer(block_num)?;
        let selection_time = monitor.end_timer("producer_selection");
        
        // Produce block
        monitor.start_timer("block_production");
        let block = engine.produce_block(producer)?;
        let production_time = monitor.end_timer("block_production");
        
        // Validate block
        monitor.start_timer("validation");
        let is_valid = engine.validate_block(&block)?;
        let validation_time = monitor.end_timer("validation");
        
        assert!(is_valid, "Block should be valid");
        
        // Apply block
        monitor.start_timer("application");
        engine.apply_block(block.clone())?;
        let application_time = monitor.end_timer("application");
        
        let full_cycle_time = monitor.end_timer("full_cycle");
        
        // Create benchmark
        monitor.create_benchmark(
            block_num,
            block.transactions.len() as u64,
            production_time,
            Duration::from_millis(0), // No real proof generation in this test
            validation_time,
            block.recursive_proof.proof_data.len(),
        );
        
        println!("✅ Block {} completed in {:?}", block_num, full_cycle_time);
        println!("   🏗️  Production: {:?}", production_time);
        println!("   🔍 Validation: {:?}", validation_time);
        println!("   📝 Application: {:?}", application_time);
        println!("   📊 Transactions: {}", block.transactions.len());
    }
    
    // Print performance report
    monitor.print_performance_report();
    
    let summary = monitor.get_performance_summary();
    assert_eq!(summary.total_blocks, 5);
    assert!(summary.average_tps > 0.0);
    
    Ok(())
}

#[tokio::test]
#[traced_test]
async fn test_stress_test_small() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Running Small Stress Test");
    
    let mut performance_test = PerformanceTest::new();
    
    // Run a small stress test
    let summary = performance_test.run_stress_test(20, 50).await;
    
    assert_eq!(summary.total_blocks, 20);
    assert_eq!(summary.total_transactions, 20 * 50);
    assert!(summary.average_tps > 0.0);
    assert!(summary.max_tps >= summary.average_tps);
    
    println!("✅ Stress test completed");
    println!("   🚀 Average TPS: {:.2}", summary.average_tps);
    println!("   🏆 Peak TPS: {:.2}", summary.max_tps);
    println!("   ⏰ Total runtime: {} seconds", summary.total_runtime_seconds);
    
    Ok(())
}

#[tokio::test]
#[traced_test]
async fn test_recursive_proof_generation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Recursive Proof Generation");
    
    let prover = RealZKProver::new()?;
    
    // Generate multiple sub-proofs
    let mut sub_proofs = Vec::new();
    
    for i in 0..3 {
        let transactions = vec![
            Transaction {
                from: Address::new(i + 1),
                to: Address::new(i + 2),
                value: 1000 * (i + 1) as u64,
                data: vec![i as u8; 10],
                gas_limit: 21000,
                nonce: i as u64,
                signature: vec![0; 64],
                sig_type: SignatureType::Ed25519,
            }
        ];
        
        let proof_result = prover.generate_state_transition_proof(
            BlockHash::random(),
            &transactions,
            (i + 1) as u64,
            1640995200 + (i as u64),
        ).await?;
        
        sub_proofs.push(proof_result);
    }
    
    // Generate recursive proof
    let recursive_proof = timeout(
        Duration::from_secs(60),
        prover.generate_recursive_proof(sub_proofs.clone()),
    ).await??;
    
    // Verify recursive proof
    let is_valid = prover.verify_proof(&recursive_proof).await?;
    
    assert!(is_valid, "Recursive proof should be valid");
    assert_eq!(recursive_proof.public_outputs.transaction_count, 3);
    assert!(recursive_proof.proof_size >= sub_proofs[0].proof_size);
    
    println!("✅ Recursive proof test passed");
    println!("   📏 Recursive proof size: {} bytes", recursive_proof.proof_size);
    println!("   🔧 Sub-proofs combined: {}", sub_proofs.len());
    println!("   📊 Total transactions: {}", recursive_proof.public_outputs.transaction_count);
    
    Ok(())
}

#[tokio::test]
#[traced_test]
async fn test_validator_selection_fairness() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Validator Selection Fairness");
    
    let genesis_state = create_test_genesis_state();
    let validators = create_diverse_validators(); // Different stake amounts
    let config = ProtocolConfig::default();
    
    let engine = ZkSacConsensusEngine::new(genesis_state, validators.clone(), config)?;
    
    let mut selection_counts = HashMap::new();
    let total_selections = 1000;
    
    // Test many selections
    for block_num in 1..=total_selections {
        let selected = engine.select_block_producer(block_num)?;
        *selection_counts.entry(selected).or_insert(0) += 1;
    }
    
    println!("📊 Validator selection results:");
    for (validator_addr, count) in &selection_counts {
        let percentage = (*count as f64 / total_selections as f64) * 100.0;
        println!("   Validator {:?}: {} selections ({:.1}%)", 
                 &validator_addr.0[..4], count, percentage);
    }
    
    // Check that all validators were selected at least once
    assert_eq!(selection_counts.len(), validators.len(), 
               "All validators should be selected at least once");
    
    // Check basic fairness (no validator should have >60% of selections)
    for count in selection_counts.values() {
        let percentage = (*count as f64 / total_selections as f64) * 100.0;
        assert!(percentage < 60.0, "No validator should dominate selection");
    }
    
    println!("✅ Validator selection fairness test passed");
    
    Ok(())
}

#[tokio::test]
#[traced_test]
async fn test_error_handling_and_recovery() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Error Handling and Recovery");
    
    let mut monitor = PerformanceMonitor::new();
    
    // Simulate various error conditions
    monitor.record_error("network_timeout");
    monitor.record_error("proof_generation_failed");
    monitor.record_error("network_timeout"); // Duplicate
    monitor.record_error("invalid_signature");
    
    let summary = monitor.get_performance_summary();
    assert_eq!(summary.total_errors, 4);
    
    // Test consensus engine error recovery
    let genesis_state = create_test_genesis_state();
    let validators = create_test_validators();
    let config = ProtocolConfig::default();
    
    let mut engine = ZkSacConsensusEngine::new(genesis_state, validators, config)?;
    
    // Try to produce block with empty transaction pool
    let producer = engine.select_block_producer(1)?;
    let block = engine.produce_block(producer)?;
    
    // Should succeed even with no transactions
    assert_eq!(block.transactions.len(), 0);
    assert!(engine.validate_block(&block)?);
    
    println!("✅ Error handling test passed");
    
    Ok(())
}

#[tokio::test]
#[traced_test]
async fn test_performance_benchmark_export() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Performance Benchmark Export");
    
    let mut performance_test = PerformanceTest::new();
    
    // Run a small test to generate data
    let _summary = performance_test.run_stress_test(5, 10).await;
    
    // Export benchmarks
    let json_data = performance_test.get_monitor().export_benchmarks()?;
    
    // Verify JSON is valid
    let parsed: serde_json::Value = serde_json::from_str(&json_data)?;
    assert!(parsed.is_array());
    
    let benchmarks_array = parsed.as_array().unwrap();
    assert_eq!(benchmarks_array.len(), 5); // Should have 5 benchmarks
    
    // Check structure of first benchmark
    let first_benchmark = &benchmarks_array[0];
    assert!(first_benchmark["timestamp"].is_number());
    assert!(first_benchmark["block_number"].is_number());
    assert!(first_benchmark["transaction_count"].is_number());
    assert!(first_benchmark["metrics"].is_object());
    
    println!("✅ Benchmark export test passed");
    println!("   📄 Exported {} benchmarks", benchmarks_array.len());
    println!("   📏 JSON size: {} bytes", json_data.len());
    
    Ok(())
}

// Helper functions
fn create_test_genesis_state() -> WorldState {
    let mut accounts = HashMap::new();
    accounts.insert(
        Address::new(1),
        Account {
            balance: 1_000_000,
            nonce: 0,
            code: Vec::new(),
            storage: HashMap::new(),
        }
    );
    
    WorldState {
        accounts,
        global_nonce: 0,
        state_root: BlockHash::zero(),
        block_number: 0,
    }
}

fn create_test_validators() -> Vec<Validator> {
    vec![
        Validator {
            address: Address::new(1),
            stake: 32_000_000_000,
            public_key: vec![1; 32],
            performance_score: 1.0,
        },
        Validator {
            address: Address::new(2),
            stake: 32_000_000_000,
            public_key: vec![2; 32],
            performance_score: 0.95,
        },
        Validator {
            address: Address::new(3),
            stake: 32_000_000_000,
            public_key: vec![3; 32],
            performance_score: 0.90,
        },
    ]
}

fn create_diverse_validators() -> Vec<Validator> {
    vec![
        Validator {
            address: Address::new(1),
            stake: 64_000_000_000, // 2x stake
            public_key: vec![1; 32],
            performance_score: 1.0,
        },
        Validator {
            address: Address::new(2),
            stake: 32_000_000_000, // 1x stake
            public_key: vec![2; 32],
            performance_score: 0.95,
        },
        Validator {
            address: Address::new(3),
            stake: 16_000_000_000, // 0.5x stake
            public_key: vec![3; 32],
            performance_score: 0.90,
        },
    ]
}

fn create_large_transaction_set(count: usize) -> Vec<Transaction> {
    (0..count).map(|i| {
        Transaction {
            from: Address::new((i % 10 + 1) as u8),
            to: Address::new((i % 10 + 2) as u8),
            value: 100 + (i as u64 * 10),
            data: vec![i as u8; i % 20 + 1],
            gas_limit: 21000 + (i as u64 * 100),
            nonce: i as u64,
            signature: vec![0; 64],
            sig_type: if i % 3 == 0 { 
                SignatureType::PostQuantum 
            } else { 
                SignatureType::Ed25519 
            },
        }
    }).collect()
} 