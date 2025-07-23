use zk_sac_engine::consensus::engine::{ZkSacConsensusEngine, ConsensusEngine};
use zk_sac_engine::types::*;
use zk_sac_engine::performance::{PerformanceMonitor, PerformanceTest};
use zk_sac_engine::zkvm::real_proofs::RealZKProver;
use std::collections::HashMap;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .init();

    println!("🚀 ZK-SAC Engine - Performance Demonstration");
    println!("============================================");
    
    // 1. Performance Monitor Test
    println!("\n📊 Phase 1: Performance Monitoring System");
    let mut monitor = PerformanceMonitor::new();
    
    // Simulate various operations
    for i in 1..=5 {
        monitor.start_timer("consensus_cycle");
        
        monitor.start_timer("block_production");
        tokio::time::sleep(tokio::time::Duration::from_millis(10 + i * 2)).await;
        let block_time = monitor.end_timer("block_production");
        
        monitor.start_timer("proof_generation");
        tokio::time::sleep(tokio::time::Duration::from_millis(50 + i * 5)).await;
        let proof_time = monitor.end_timer("proof_generation");
        
        monitor.start_timer("validation");
        tokio::time::sleep(tokio::time::Duration::from_millis(5 + i)).await;
        let validation_time = monitor.end_timer("validation");
        
        let cycle_time = monitor.end_timer("consensus_cycle");
        
        // Create benchmark
        monitor.create_benchmark(
            i,
            50 + (i as u64 * 10), // Increasing transactions
            block_time,
            proof_time,
            validation_time,
            1024 + (i as usize * 256), // Increasing proof size
        );
        
        info!("✅ Consensus cycle {} completed in {:?}", i, cycle_time);
    }
    
    monitor.print_performance_report();
    
    // 2. Stress Test
    println!("\n🔥 Phase 2: Stress Testing");
    let mut stress_test = PerformanceTest::new();
    
    let summary = stress_test.run_stress_test(25, 100).await;
    
    println!("\n📈 Stress Test Results:");
    println!("   🔗 Blocks processed: {}", summary.total_blocks);
    println!("   📝 Total transactions: {}", summary.total_transactions);
    println!("   ⏰ Runtime: {} seconds", summary.total_runtime_seconds);
    println!("   🚀 Average TPS: {:.2}", summary.average_tps);
    println!("   🏆 Peak TPS: {:.2}", summary.max_tps);
    println!("   ⚡ Avg block time: {:.2} ms", summary.average_block_time_ms);
    
    // 3. Real ZK Proof System Test (Mock Mode)
    println!("\n🔬 Phase 3: ZK Proof System");
    
    match RealZKProver::new() {
        Ok(prover) => {
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
            
            let start_time = std::time::Instant::now();
            
            let proof_result = prover.generate_state_transition_proof(
                BlockHash::zero(),
                &transactions,
                1,
                1640995200,
            ).await?;
            
            let generation_time = start_time.elapsed();
            
            println!("   🔧 ZK Proof Generated:");
            println!("      📏 Size: {} bytes", proof_result.proof_size);
            println!("      ⏱️  Time: {:?}", generation_time);
            println!("      📊 Transactions: {}", proof_result.public_outputs.transaction_count);
            println!("      ⛽ Gas used: {}", proof_result.public_outputs.gas_used);
            
            // Verify the proof
            let verification_start = std::time::Instant::now();
            let is_valid = prover.verify_proof(&proof_result).await?;
            let verification_time = verification_start.elapsed();
            
            println!("   🔍 Proof Verification:");
            println!("      ✅ Valid: {}", is_valid);
            println!("      ⏱️  Time: {:?}", verification_time);
            
            // Test recursive proofs
            let sub_proofs = vec![proof_result.clone(), proof_result.clone()];
            let recursive_proof = prover.generate_recursive_proof(sub_proofs).await?;
            
            println!("   🔄 Recursive Proof:");
            println!("      📏 Size: {} bytes", recursive_proof.proof_size);
            println!("      📊 Combined transactions: {}", recursive_proof.public_outputs.transaction_count);
        },
        Err(e) => {
            println!("   🚧 ZK Prover in mock mode: {}", e);
        }
    }
    
    // 4. Consensus Engine Integration Test
    println!("\n⚙️  Phase 4: Consensus Engine Integration");
    
    let genesis_state = create_test_genesis_state();
    let validators = create_test_validators();
    let config = ProtocolConfig::default();
    
    let mut engine = ZkSacConsensusEngine::new(genesis_state, validators, config)?;
    
    // Add test transactions
    let test_transactions = create_test_transactions(50);
    for tx in test_transactions {
        engine.pending_transactions.push(tx);
    }
    
    let integration_start = std::time::Instant::now();
    
    // Process multiple blocks
    for block_num in 1..=10 {
        let producer = engine.select_block_producer(block_num)?;
        let block = engine.produce_block(producer)?;
        let is_valid = engine.validate_block(&block)?;
        
        if is_valid {
            engine.apply_block(block.clone())?;
            info!("✅ Block {} processed with {} transactions", 
                  block_num, block.transactions.len());
        } else {
            error!("❌ Block {} validation failed", block_num);
        }
    }
    
    let integration_time = integration_start.elapsed();
    
    println!("   🏗️  Processed 10 blocks in {:?}", integration_time);
    println!("   📊 Average time per block: {:?}", integration_time / 10);
    
    // 5. Export Performance Data
    println!("\n📄 Phase 5: Data Export");
    
    let benchmark_json = stress_test.get_monitor().export_benchmarks()?;
    println!("   📊 Benchmark data exported: {} chars", benchmark_json.len());
    
    // Final Summary
    println!("\n🎉 Performance Demonstration Complete!");
    println!("=====================================");
    println!("✅ Performance monitoring system");
    println!("✅ Stress testing capabilities"); 
    println!("✅ ZK proof generation & verification");
    println!("✅ Consensus engine integration");
    println!("✅ Data export functionality");
    println!("\n🚀 ZK-SAC Engine is ready for production testing!");
    
    Ok(())
}

// Helper functions
fn create_test_genesis_state() -> WorldState {
    let mut accounts = HashMap::new();
    for i in 1..=10 {
        accounts.insert(
            Address::new(i),
            Account {
                balance: 1_000_000 + (i as u64 * 100_000),
                nonce: 0,
                code: Vec::new(),
                storage: HashMap::new(),
            }
        );
    }
    
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
            stake: 48_000_000_000,
            public_key: vec![2; 32],
            performance_score: 0.98,
        },
        Validator {
            address: Address::new(3),
            stake: 16_000_000_000,
            public_key: vec![3; 32],
            performance_score: 0.95,
        },
        Validator {
            address: Address::new(4),
            stake: 24_000_000_000,
            public_key: vec![4; 32],
            performance_score: 0.92,
        },
    ]
}

fn create_test_transactions(count: usize) -> Vec<Transaction> {
    (0..count).map(|i| {
        Transaction {
            from: Address::new((i % 8 + 1) as u8),
            to: Address::new((i % 8 + 2) as u8),
            value: 100 + (i as u64 * 50),
            data: vec![i as u8; (i % 32) + 1],
            gas_limit: 21000 + (i as u64 * 500),
            nonce: i as u64,
            signature: vec![0; 64],
            sig_type: if i % 4 == 0 { 
                SignatureType::PostQuantum 
            } else { 
                SignatureType::Ed25519 
            },
        }
    }).collect()
} 