use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize, BenchmarkId, Throughput};
use std::time::Duration;
use zk_sac_engine::{
    crypto::{
        hash::MultiHasher,
        signatures::QuantumResistantSigner,
    },
    types::{Transaction, Address, BlockHash},
};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature};
use rand::rngs::OsRng;

fn bench_multi_hash_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_algorithms");
    
    for data_size in [32, 256, 1024, 4096, 16384, 65536].iter() {
        let data = vec![0u8; *data_size];
        group.throughput(Throughput::Bytes(*data_size as u64));
        
        // Blake3 benchmarks
        group.bench_with_input(
            BenchmarkId::new("blake3", data_size),
            &data,
            |b, data| {
                let hasher = MultiHasher::new();
                b.iter(|| black_box(hasher.blake3_hash(data)))
            },
        );
        
        // Keccak256 benchmarks
        group.bench_with_input(
            BenchmarkId::new("keccak256", data_size),
            &data,
            |b, data| {
                let hasher = MultiHasher::new();
                b.iter(|| black_box(hasher.keccak256_hash(data)))
            },
        );
        
        // SHA3-256 benchmarks
        group.bench_with_input(
            BenchmarkId::new("sha3_256", data_size),
            &data,
            |b, data| {
                let hasher = MultiHasher::new();
                b.iter(|| black_box(hasher.sha3_256_hash(data)))
            },
        );
    }
    group.finish();
}

fn bench_incremental_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_hashing");
    
    for chunk_count in [1, 5, 10, 50, 100].iter() {
        let chunks: Vec<Vec<u8>> = (0..*chunk_count)
            .map(|i| vec![i as u8; 1024])
            .collect();
        
        group.throughput(Throughput::Bytes((chunk_count * 1024) as u64));
        
        group.bench_with_input(
            BenchmarkId::new("blake3_incremental", chunk_count),
            &chunks,
            |b, chunks| {
                let hasher = MultiHasher::new();
                b.iter(|| {
                    let mut incremental = hasher.new_incremental_blake3();
                    for chunk in chunks {
                        incremental.update(chunk);
                    }
                    black_box(incremental.finalize())
                })
            },
        );
    }
    group.finish();
}

fn bench_extended_output_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("extended_output");
    let input_data = vec![0u8; 1024];
    
    for output_size in [64, 128, 256, 512, 1024].iter() {
        group.throughput(Throughput::Bytes(*output_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("shake128", output_size),
            output_size,
            |b, &output_size| {
                let hasher = MultiHasher::new();
                b.iter(|| black_box(hasher.shake128_hash(&input_data, output_size)))
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("shake256", output_size),
            output_size,
            |b, &output_size| {
                let hasher = MultiHasher::new();
                b.iter(|| black_box(hasher.shake256_hash(&input_data, output_size)))
            },
        );
    }
    group.finish();
}

fn bench_signature_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("signature_operations");
    group.measurement_time(Duration::from_secs(5));
    
    // Ed25519 signature benchmarks
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    let message = b"benchmark_message_for_signing_performance_test";
    let signature = signing_key.sign(message);
    
    group.bench_function("ed25519_sign", |b| {
        b.iter(|| black_box(signing_key.sign(message)))
    });
    
    group.bench_function("ed25519_verify", |b| {
        b.iter(|| black_box(verifying_key.verify(message, &signature).is_ok()))
    });
    
    // Batch signature verification
    for batch_size in [10, 50, 100, 500, 1000].iter() {
        let signatures_and_messages: Vec<(VerifyingKey, Vec<u8>, Signature)> = (0..*batch_size)
            .map(|i| {
                let key = SigningKey::generate(&mut OsRng);
                let message = format!("message_{}", i).into_bytes();
                let signature = key.sign(&message);
                (key.verifying_key(), message, signature)
            })
            .collect();
        
        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("ed25519_batch_verify", batch_size),
            &signatures_and_messages,
            |b, batch| {
                b.iter(|| {
                    let mut all_valid = true;
                    for (verifying_key, message, signature) in batch {
                        if verifying_key.verify(message, signature).is_err() {
                            all_valid = false;
                            break;
                        }
                    }
                    black_box(all_valid)
                })
            },
        );
    }
    group.finish();
}

fn bench_evm_compatibility(c: &mut Criterion) {
    let mut group = c.benchmark_group("evm_compatibility");
    let hasher = MultiHasher::new();
    
    // EVM address generation benchmark
    let public_key = vec![0x04u8; 65]; // Uncompressed public key
    group.bench_function("evm_address_generation", |b| {
        b.iter(|| black_box(hasher.generate_evm_address(&public_key)))
    });
    
    // EVM transaction hashing
    for tx_data_size in [0, 100, 1000, 10000].iter() {
        let transaction = Transaction {
            from: Address::random(),
            to: Some(Address::random()),
            value: 1000,
            data: vec![0u8; *tx_data_size],
            nonce: 0,
            gas_limit: 21000,
            gas_price: 20,
            signature: vec![0; 65],
        };
        
        group.throughput(Throughput::Bytes(*tx_data_size as u64));
        group.bench_with_input(
            BenchmarkId::new("evm_transaction_hash", tx_data_size),
            &transaction,
            |b, tx| {
                b.iter(|| black_box(hasher.hash_evm_transaction(tx)))
            },
        );
    }
    
    // Hex encoding/decoding performance
    for data_size in [20, 32, 64, 256, 1024].iter() {
        let data = vec![0u8; *data_size];
        group.throughput(Throughput::Bytes(*data_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("hex_encode", data_size),
            &data,
            |b, data| {
                b.iter(|| black_box(hex::encode(data)))
            },
        );
        
        let hex_string = hex::encode(&data);
        group.bench_with_input(
            BenchmarkId::new("hex_decode", data_size),
            &hex_string,
            |b, hex_str| {
                b.iter(|| black_box(hex::decode(hex_str).unwrap()))
            },
        );
    }
    group.finish();
}

fn bench_hash_comparison_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_comparison");
    let hasher = MultiHasher::new();
    
    // Generate test hashes
    let test_data = vec![0u8; 1024];
    let blake3_hash = hasher.blake3_hash(&test_data);
    let keccak_hash = hasher.keccak256_hash(&test_data);
    let sha3_hash = hasher.sha3_256_hash(&test_data);
    
    group.bench_function("blake3_vs_blake3", |b| {
        b.iter(|| black_box(hasher.compare_hashes(&blake3_hash, &blake3_hash)))
    });
    
    group.bench_function("keccak_vs_keccak", |b| {
        b.iter(|| black_box(hasher.compare_hashes(&keccak_hash, &keccak_hash)))
    });
    
    group.bench_function("sha3_vs_sha3", |b| {
        b.iter(|| black_box(hasher.compare_hashes(&sha3_hash, &sha3_hash)))
    });
    
    group.bench_function("mixed_hash_comparison", |b| {
        b.iter(|| {
            black_box(hasher.compare_hashes(&blake3_hash, &keccak_hash));
            black_box(hasher.compare_hashes(&keccak_hash, &sha3_hash));
            black_box(hasher.compare_hashes(&sha3_hash, &blake3_hash));
        })
    });
    
    group.finish();
}

fn bench_merkle_tree_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle_tree");
    let hasher = MultiHasher::new();
    
    for leaf_count in [8, 16, 32, 64, 128, 256, 512, 1024].iter() {
        let leaves: Vec<Vec<u8>> = (0..*leaf_count)
            .map(|i| format!("leaf_{}", i).into_bytes())
            .collect();
        
        group.throughput(Throughput::Elements(*leaf_count as u64));
        
        // Blake3 Merkle tree
        group.bench_with_input(
            BenchmarkId::new("blake3_merkle", leaf_count),
            &leaves,
            |b, leaves| {
                b.iter(|| black_box(hasher.compute_merkle_root_blake3(leaves)))
            },
        );
        
        // Keccak256 Merkle tree (EVM compatible)
        group.bench_with_input(
            BenchmarkId::new("keccak256_merkle", leaf_count),
            &leaves,
            |b, leaves| {
                b.iter(|| black_box(hasher.compute_merkle_root_keccak256(leaves)))
            },
        );
        
        // SHA3-256 Merkle tree (post-quantum)
        group.bench_with_input(
            BenchmarkId::new("sha3_256_merkle", leaf_count),
            &leaves,
            |b, leaves| {
                b.iter(|| black_box(hasher.compute_merkle_root_sha3_256(leaves)))
            },
        );
    }
    group.finish();
}

fn bench_post_quantum_signatures(c: &mut Criterion) {
    let mut group = c.benchmark_group("post_quantum_signatures");
    group.measurement_time(Duration::from_secs(8));
    
    let signer = QuantumResistantSigner::new();
    let message = b"post_quantum_signature_benchmark_message";
    
    // Mock post-quantum signature operations
    group.bench_function("pq_sign", |b| {
        b.iter(|| black_box(signer.sign_post_quantum(message)))
    });
    
    let signature = signer.sign_post_quantum(message);
    group.bench_function("pq_verify", |b| {
        b.iter(|| black_box(signer.verify_post_quantum(message, &signature)))
    });
    
    // Aggregate signature benchmarks
    for aggregation_count in [5, 10, 20, 50].iter() {
        let signatures: Vec<Vec<u8>> = (0..*aggregation_count)
            .map(|i| {
                let msg = format!("message_{}", i).into_bytes();
                signer.sign_post_quantum(&msg)
            })
            .collect();
        
        group.throughput(Throughput::Elements(*aggregation_count as u64));
        group.bench_with_input(
            BenchmarkId::new("pq_aggregate", aggregation_count),
            &signatures,
            |b, sigs| {
                b.iter(|| black_box(signer.aggregate_signatures(sigs)))
            },
        );
    }
    
    group.finish();
}

fn bench_cryptographic_nonce_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("nonce_generation");
    let hasher = MultiHasher::new();
    
    // VDF-based nonce generation
    for difficulty in [10, 15, 20, 25].iter() {
        group.bench_with_input(
            BenchmarkId::new("vdf_nonce", difficulty),
            difficulty,
            |b, &difficulty| {
                b.iter(|| black_box(hasher.generate_vdf_nonce(difficulty)))
            },
        );
    }
    
    // Random beacon nonce
    group.bench_function("random_beacon_nonce", |b| {
        b.iter(|| black_box(hasher.generate_random_beacon_nonce()))
    });
    
    // Proof-of-work style nonce
    for target_zeros in [4, 8, 12, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("pow_nonce", target_zeros),
            target_zeros,
            |b, &target_zeros| {
                b.iter(|| black_box(hasher.generate_pow_nonce(target_zeros)))
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_multi_hash_algorithms,
    bench_incremental_hashing,
    bench_extended_output_functions,
    bench_signature_operations,
    bench_evm_compatibility,
    bench_hash_comparison_performance,
    bench_merkle_tree_operations,
    bench_post_quantum_signatures,
    bench_cryptographic_nonce_generation
);

criterion_main!(benches); 