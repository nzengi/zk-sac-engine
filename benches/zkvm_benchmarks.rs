use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize, BenchmarkId, Throughput};
use tokio::runtime::Runtime;
use std::time::Duration;
use zk_sac_engine::{
    zkvm::{Risc0Executor, ZKVMConfig},
    types::{Transaction, Address, Block},
    crypto::hash::MultiHasher,
};
use sp1_sdk::{ProverClient, SP1Stdin, SP1PublicValues};

fn bench_sp1_proof_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("sp1_proof_generation");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10); // Reduce sample size for expensive operations
    
    for tx_count in [1, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("transactions", tx_count),
            tx_count,
            |b, &tx_count| {
                b.to_async(&rt).iter_batched(
                    || setup_zkvm_proof_input(tx_count),
                    |(executor, prev_state, transactions)| async move {
                        black_box(
                            executor.generate_state_transition_proof(prev_state, transactions).await.unwrap()
                        )
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_sp1_proof_verification(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("sp1_proof_verification");
    group.measurement_time(Duration::from_secs(15));
    
    for tx_count in [1, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("transactions", tx_count),
            tx_count,
            |b, &tx_count| {
                b.to_async(&rt).iter_batched(
                    || setup_zkvm_proof_for_verification(tx_count),
                    |(executor, proof)| async move {
                        black_box(executor.verify_proof(&proof).await.unwrap())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_recursive_proof_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("recursive_proof");
    group.measurement_time(Duration::from_secs(45));
    group.sample_size(5); // Very expensive operations
    
    for depth in [2, 3, 4, 5].iter() {
        group.bench_with_input(
            BenchmarkId::new("depth", depth),
            depth,
            |b, &depth| {
                b.to_async(&rt).iter_batched(
                    || setup_recursive_proof_chain(depth),
                    |(executor, proof_chain)| async move {
                        black_box(executor.generate_recursive_proof(proof_chain).await.unwrap())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_zkvm_circuit_compilation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("circuit_compilation");
    group.measurement_time(Duration::from_secs(20));
    
    for circuit_complexity in ["simple", "medium", "complex"].iter() {
        group.bench_with_input(
            BenchmarkId::new("complexity", circuit_complexity),
            circuit_complexity,
            |b, &complexity| {
                b.to_async(&rt).iter_batched(
                    || setup_circuit_for_complexity(complexity),
                    |circuit_source| async move {
                        let executor = Risc0Executor::new().unwrap();
                        black_box(executor.compile_circuit(circuit_source).await.unwrap())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_witness_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("witness_generation");
    
    for data_size in [100, 500, 1000, 5000, 10000].iter() {
        group.throughput(Throughput::Bytes(*data_size as u64));
        group.bench_with_input(
            BenchmarkId::new("data_size", data_size),
            data_size,
            |b, &data_size| {
                b.to_async(&rt).iter_batched(
                    || setup_witness_data(data_size),
                    |(executor, witness_data)| async move {
                        black_box(executor.generate_witness(witness_data).await.unwrap())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_zkvm_state_transition_validation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("state_transition_validation");
    
    for state_size in [32, 64, 128, 256, 512].iter() {
        group.throughput(Throughput::Bytes(*state_size as u64));
        group.bench_with_input(
            BenchmarkId::new("state_size", state_size),
            state_size,
            |b, &state_size| {
                b.to_async(&rt).iter_batched(
                    || setup_state_transition(state_size),
                    |(executor, prev_state, new_state, proof)| async move {
                        black_box(
                            executor.validate_state_transition(prev_state, new_state, proof).await.unwrap()
                        )
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_parallel_proof_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("parallel_proof_generation");
    group.measurement_time(Duration::from_secs(40));
    
    for parallel_count in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("parallel", parallel_count),
            parallel_count,
            |b, &parallel_count| {
                b.to_async(&rt).iter_batched(
                    || setup_parallel_proof_inputs(parallel_count),
                    |(executor, proof_inputs)| async move {
                        black_box(executor.generate_proofs_parallel(proof_inputs).await.unwrap())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_zkvm_memory_optimization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("memory_optimization");
    
    for memory_mode in ["standard", "optimized", "streaming"].iter() {
        group.bench_with_input(
            BenchmarkId::new("mode", memory_mode),
            memory_mode,
            |b, &mode| {
                b.to_async(&rt).iter_batched(
                    || setup_memory_optimized_zkvm(mode),
                    |(executor, large_input)| async move {
                        black_box(executor.process_large_input(large_input).await.unwrap())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_proof_aggregation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("proof_aggregation");
    group.measurement_time(Duration::from_secs(25));
    
    for proof_count in [2, 5, 10, 20, 50].iter() {
        group.throughput(Throughput::Elements(*proof_count as u64));
        group.bench_with_input(
            BenchmarkId::new("proofs", proof_count),
            proof_count,
            |b, &proof_count| {
                b.to_async(&rt).iter_batched(
                    || setup_proofs_for_aggregation(proof_count),
                    |(executor, proofs)| async move {
                        black_box(executor.aggregate_proofs(proofs).await.unwrap())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_zkvm_constraint_optimization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("constraint_optimization");
    
    for constraint_count in [1000, 5000, 10000, 50000, 100000].iter() {
        group.throughput(Throughput::Elements(*constraint_count as u64));
        group.bench_with_input(
            BenchmarkId::new("constraints", constraint_count),
            constraint_count,
            |b, &constraint_count| {
                b.to_async(&rt).iter_batched(
                    || setup_constraint_system(constraint_count),
                    |(executor, constraints)| async move {
                        black_box(executor.optimize_constraints(constraints).await.unwrap())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

// Setup functions for benchmarks

fn setup_zkvm_proof_input(tx_count: usize) -> (Risc0Executor, Vec<u8>, Vec<Transaction>) {
    let executor = Risc0Executor::new().expect("Failed to create Risc0 executor");
    let prev_state = vec![0u8; 32]; // Mock previous state
    let transactions = generate_test_transactions(tx_count);
    (executor, prev_state, transactions)
}

fn setup_zkvm_proof_for_verification(tx_count: usize) -> (Risc0Executor, Vec<u8>) {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let executor = Risc0Executor::new().expect("Failed to create Risc0 executor");
        let prev_state = vec![0u8; 32];
        let transactions = generate_test_transactions(tx_count);
        
        let proof = executor
            .generate_state_transition_proof(prev_state, transactions)
            .await
            .expect("Failed to generate proof");
        
        (executor, proof)
    })
}

fn setup_recursive_proof_chain(depth: usize) -> (Risc0Executor, Vec<Vec<u8>>) {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let executor = Risc0Executor::new().expect("Failed to create Risc0 executor");
        let mut proofs = Vec::new();
        
        for i in 0..depth {
            let prev_state = vec![i as u8; 32];
            let transactions = generate_test_transactions(1);
            
            let proof = executor
                .generate_state_transition_proof(prev_state, transactions)
                .await
                .expect("Failed to generate proof");
            proofs.push(proof);
        }
        
        (executor, proofs)
    })
}

fn setup_circuit_for_complexity(complexity: &str) -> String {
    match complexity {
        "simple" => {
            r#"
            fn main() {
                let a = sp1_zkvm::io::read::<u32>();
                let b = sp1_zkvm::io::read::<u32>();
                let c = a + b;
                sp1_zkvm::io::commit(&c);
            }
            "#.to_string()
        }
        "medium" => {
            r#"
            fn main() {
                let values = sp1_zkvm::io::read::<Vec<u32>>();
                let mut result = 0;
                for (i, value) in values.iter().enumerate() {
                    result += value * (i as u32 + 1);
                }
                sp1_zkvm::io::commit(&result);
            }
            "#.to_string()
        }
        "complex" => {
            r#"
            fn main() {
                let matrix = sp1_zkvm::io::read::<Vec<Vec<u32>>>();
                let mut result = vec![vec![0u32; matrix[0].len()]; matrix.len()];
                
                for i in 0..matrix.len() {
                    for j in 0..matrix[0].len() {
                        for k in 0..matrix.len() {
                            result[i][j] += matrix[i][k] * matrix[k][j];
                        }
                    }
                }
                
                sp1_zkvm::io::commit(&result);
            }
            "#.to_string()
        }
        _ => panic!("Unknown complexity level"),
    }
}

fn setup_witness_data(data_size: usize) -> (Risc0Executor, Vec<u8>) {
    let executor = Risc0Executor::new().expect("Failed to create Risc0 executor");
    let data = (0..data_size).map(|i| (i % 256) as u8).collect();
    (executor, data)
}

fn setup_state_transition(state_size: usize) -> (Risc0Executor, Vec<u8>, Vec<u8>, Vec<u8>) {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let executor = Risc0Executor::new().expect("Failed to create Risc0 executor");
        let prev_state = vec![0u8; state_size];
        let new_state = vec![1u8; state_size];
        let transactions = generate_test_transactions(1);
        
        let proof = executor
            .generate_state_transition_proof(prev_state.clone(), transactions)
            .await
            .expect("Failed to generate proof");
            
        (executor, prev_state, new_state, proof)
    })
}

fn setup_parallel_proof_inputs(parallel_count: usize) -> (Risc0Executor, Vec<(Vec<u8>, Vec<Transaction>)>) {
    let executor = Risc0Executor::new().expect("Failed to create Risc0 executor");
    let mut inputs = Vec::new();
    
    for i in 0..parallel_count {
        let prev_state = vec![i as u8; 32];
        let transactions = generate_test_transactions(2);
        inputs.push((prev_state, transactions));
    }
    
    (executor, inputs)
}

fn setup_memory_optimized_zkvm(mode: &str) -> (Risc0Executor, Vec<u8>) {
    let config = ZKVMConfig {
        memory_optimization: mode.to_string(),
        prover_mode: "cpu".to_string(),
        parallel_execution: true,
    };
    let executor = Risc0Executor::with_config(config).expect("Failed to create Risc0 executor");
    let large_input = vec![0u8; 1_000_000]; // 1MB input
    (executor, large_input)
}

fn setup_proofs_for_aggregation(proof_count: usize) -> (Risc0Executor, Vec<Vec<u8>>) {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let executor = Risc0Executor::new().expect("Failed to create Risc0 executor");
        let mut proofs = Vec::new();
        
        for i in 0..proof_count {
            let prev_state = vec![i as u8; 32];
            let transactions = generate_test_transactions(1);
            
            let proof = executor
                .generate_state_transition_proof(prev_state, transactions)
                .await
                .expect("Failed to generate proof");
            proofs.push(proof);
        }
        
        (executor, proofs)
    })
}

fn setup_constraint_system(constraint_count: usize) -> (Risc0Executor, Vec<String>) {
    let executor = Risc0Executor::new().expect("Failed to create Risc0 executor");
    let constraints = (0..constraint_count)
        .map(|i| format!("constraint_{}", i))
        .collect();
    (executor, constraints)
}

fn generate_test_transactions(count: usize) -> Vec<Transaction> {
    (0..count)
        .map(|i| Transaction {
            from: Address::random(),
            to: Some(Address::random()),
            value: 1000 + i as u64,
            data: vec![],
            nonce: i as u64,
            gas_limit: 21000,
            gas_price: 20,
            signature: vec![0; 64],
        })
        .collect()
}

criterion_group!(
    benches,
    bench_sp1_proof_generation,
    bench_sp1_proof_verification,
    bench_recursive_proof_generation,
    bench_zkvm_circuit_compilation,
    bench_witness_generation,
    bench_zkvm_state_transition_validation,
    bench_parallel_proof_generation,
    bench_zkvm_memory_optimization,
    bench_proof_aggregation,
    bench_zkvm_constraint_optimization
);

criterion_main!(benches); 