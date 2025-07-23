use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize, BenchmarkId, Throughput};
use tokio::runtime::Runtime;
use std::time::Duration;
use zk_sac_engine::{
    consensus::{ConsensusEngine, BeamChainConfig},
    types::{Transaction, Address, Block},
    async_utils::{AsyncTaskPool, BatchProcessor, ConsensusCoordinator},
};

fn bench_block_production(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("block_production");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(Duration::from_secs(10));
    
    for tx_count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("transactions", tx_count),
            tx_count,
            |b, &tx_count| {
                b.to_async(&rt).iter_batched(
                    || setup_consensus_engine_with_transactions(tx_count),
                    |(mut engine, transactions)| async move {
                        black_box(engine.produce_block(transactions).await.unwrap())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_block_validation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("block_validation");
    group.throughput(Throughput::Elements(1));
    
    for tx_count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("transactions", tx_count),
            tx_count,
            |b, &tx_count| {
                b.to_async(&rt).iter_batched(
                    || setup_block_for_validation(tx_count),
                    |(mut engine, block)| async move {
                        black_box(engine.validate_block(&block).await.unwrap())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_parallel_block_production(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("parallel_block_production");
    group.throughput(Throughput::Elements(1));
    
    for concurrent_blocks in [1, 2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent", concurrent_blocks),
            concurrent_blocks,
            |b, &concurrent_blocks| {
                b.to_async(&rt).iter_batched(
                    || setup_parallel_consensus(concurrent_blocks),
                    |coordinator| async move {
                        black_box(coordinator.produce_blocks_parallel().await.unwrap())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_transaction_batching(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("transaction_batching");
    
    for batch_size in [10, 50, 100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_size", batch_size),
            batch_size,
            |b, &batch_size| {
                b.to_async(&rt).iter_batched(
                    || setup_batch_processor(batch_size),
                    |(processor, transactions)| async move {
                        black_box(processor.process_batch(transactions).await.unwrap())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_async_task_pool_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("async_task_pool");
    
    for pool_size in [1, 2, 4, 8, 16, 32].iter() {
        for task_count in [10, 50, 100].iter() {
            group.bench_with_input(
                BenchmarkId::from_parameter(format!("pool{}_tasks{}", pool_size, task_count)),
                &(*pool_size, *task_count),
                |b, &(pool_size, task_count)| {
                    b.to_async(&rt).iter_batched(
                        || setup_task_pool(pool_size, task_count),
                        |(pool, tasks)| async move {
                            black_box(pool.execute_all(tasks).await)
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }
    group.finish();
}

fn bench_validator_selection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("validator_selection");
    group.throughput(Throughput::Elements(1));
    
    for validator_count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("validators", validator_count),
            validator_count,
            |b, &validator_count| {
                b.to_async(&rt).iter_batched(
                    || setup_validator_set(validator_count),
                    |mut engine| async move {
                        black_box(engine.select_block_producer().await.unwrap())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_consensus_finalization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("consensus_finalization");
    group.throughput(Throughput::Elements(1));
    
    for block_count in [1, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("blocks", block_count),
            block_count,
            |b, &block_count| {
                b.to_async(&rt).iter_batched(
                    || setup_blocks_for_finalization(block_count),
                    |(mut engine, blocks)| async move {
                        for block in blocks {
                            black_box(engine.finalize_block(block.hash).await.unwrap());
                        }
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

// Benchmark memory usage and allocation patterns
fn bench_memory_efficiency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("memory_efficiency");
    
    for data_size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        group.throughput(Throughput::Bytes(*data_size as u64));
        group.bench_with_input(
            BenchmarkId::new("data_size", data_size),
            data_size,
            |b, &data_size| {
                b.to_async(&rt).iter_batched(
                    || setup_large_block(data_size),
                    |(mut engine, block)| async move {
                        // Test both production and validation for memory efficiency
                        let validation_result = engine.validate_block(&block).await.unwrap();
                        black_box(validation_result)
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

// Setup functions for benchmarks

fn setup_consensus_engine_with_transactions(tx_count: usize) -> (ConsensusEngine, Vec<Transaction>) {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let config = BeamChainConfig::new_for_testing();
        let engine = ConsensusEngine::new(config).await.unwrap();
        let transactions = generate_test_transactions(tx_count);
        (engine, transactions)
    })
}

fn setup_block_for_validation(tx_count: usize) -> (ConsensusEngine, Block) {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let config = BeamChainConfig::new_for_testing();
        let mut engine = ConsensusEngine::new(config).await.unwrap();
        let transactions = generate_test_transactions(tx_count);
        let block = engine.produce_block(transactions).await.unwrap();
        (engine, block)
    })
}

fn setup_parallel_consensus(concurrent_blocks: usize) -> ConsensusCoordinator {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let coordinator = ConsensusCoordinator::new();
        coordinator.set_concurrency_level(concurrent_blocks);
        coordinator
    })
}

fn setup_batch_processor(batch_size: usize) -> (BatchProcessor<Transaction>, Vec<Transaction>) {
    let processor = BatchProcessor::new(batch_size);
    let transactions = generate_test_transactions(batch_size);
    (processor, transactions)
}

fn setup_task_pool(pool_size: usize, task_count: usize) -> (AsyncTaskPool, Vec<Box<dyn Fn() -> u64 + Send>>) {
    let pool = AsyncTaskPool::new(pool_size);
    let tasks: Vec<Box<dyn Fn() -> u64 + Send>> = (0..task_count)
        .map(|i| {
            Box::new(move || {
                // Simulate some CPU work
                (0..1000).fold(i as u64, |acc, x| acc.wrapping_add(x))
            }) as Box<dyn Fn() -> u64 + Send>
        })
        .collect();
    (pool, tasks)
}

fn setup_validator_set(validator_count: usize) -> ConsensusEngine {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let config = BeamChainConfig::new_for_testing();
        let mut engine = ConsensusEngine::new(config).await.unwrap();
        
        // Create validator set
        let validators = (0..validator_count)
            .map(|i| zk_sac_engine::types::Validator {
                address: Address::random(),
                stake: 1000000 + (i as u64 * 100000),
                public_key: vec![i as u8; 32],
            })
            .collect();
        
        let validator_set = zk_sac_engine::types::ValidatorSet { validators };
        engine.update_validator_set(validator_set).await.unwrap();
        engine
    })
}

fn setup_blocks_for_finalization(block_count: usize) -> (ConsensusEngine, Vec<Block>) {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let config = BeamChainConfig::new_for_testing();
        let mut engine = ConsensusEngine::new(config).await.unwrap();
        
        let mut blocks = Vec::new();
        for i in 0..block_count {
            let transactions = generate_test_transactions(10);
            let block = engine.produce_block(transactions).await.unwrap();
            blocks.push(block);
        }
        
        (engine, blocks)
    })
}

fn setup_large_block(data_size: usize) -> (ConsensusEngine, Block) {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let config = BeamChainConfig::new_for_testing();
        let mut engine = ConsensusEngine::new(config).await.unwrap();
        
        // Create large transaction with specified data size
        let large_transaction = Transaction {
            from: Address::random(),
            to: Some(Address::random()),
            value: 1000,
            data: vec![0u8; data_size],
            nonce: 0,
            gas_limit: 1000000,
            gas_price: 20,
            signature: vec![0; 64],
        };
        
        let block = engine.produce_block(vec![large_transaction]).await.unwrap();
        (engine, block)
    })
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
    bench_block_production,
    bench_block_validation,
    bench_parallel_block_production,
    bench_transaction_batching,
    bench_async_task_pool_performance,
    bench_validator_selection,
    bench_consensus_finalization,
    bench_memory_efficiency
);

criterion_main!(benches); 