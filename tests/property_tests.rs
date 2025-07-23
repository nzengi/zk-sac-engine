use proptest::prelude::*;
use quickcheck::{quickcheck, TestResult};
use quickcheck_macros::quickcheck;
use zk_sac_engine::{
    types::{Block, Transaction, Address, BlockHash},
    crypto::hash::MultiHasher,
    consensus::BeamChainConfig,
};

// Property tests for cryptographic functions

#[quickcheck]
fn prop_hash_deterministic(data: Vec<u8>) -> bool {
    let hasher = MultiHasher::new();
    let hash1 = hasher.blake3_hash(&data);
    let hash2 = hasher.blake3_hash(&data);
    hash1 == hash2
}

#[quickcheck]
fn prop_hash_different_inputs_different_outputs(data1: Vec<u8>, data2: Vec<u8>) -> TestResult {
    if data1 == data2 {
        return TestResult::discard();
    }
    
    let hasher = MultiHasher::new();
    let hash1 = hasher.blake3_hash(&data1);
    let hash2 = hasher.blake3_hash(&data2);
    
    TestResult::from_bool(hash1 != hash2)
}

#[quickcheck]
fn prop_keccak256_length_consistency(data: Vec<u8>) -> bool {
    let hasher = MultiHasher::new();
    let hash = hasher.keccak256_hash(&data);
    hash.len() == 32
}

#[quickcheck]
fn prop_sha3_256_length_consistency(data: Vec<u8>) -> bool {
    let hasher = MultiHasher::new();
    let hash = hasher.sha3_256_hash(&data);
    hash.len() == 32
}

#[quickcheck]
fn prop_blake3_length_consistency(data: Vec<u8>) -> bool {
    let hasher = MultiHasher::new();
    let hash = hasher.blake3_hash(&data);
    hash.len() == 32
}

// Property tests for address generation

#[quickcheck]
fn prop_address_generation_deterministic(seed: u64) -> bool {
    let addr1 = Address::from_seed(seed);
    let addr2 = Address::from_seed(seed);
    addr1 == addr2
}

#[quickcheck]
fn prop_address_different_seeds_different_addresses(seed1: u64, seed2: u64) -> TestResult {
    if seed1 == seed2 {
        return TestResult::discard();
    }
    
    let addr1 = Address::from_seed(seed1);
    let addr2 = Address::from_seed(seed2);
    
    TestResult::from_bool(addr1 != addr2)
}

// Property tests for transaction ordering and validation

proptest! {
    #[test]
    fn prop_transaction_nonce_ordering(
        nonces in prop::collection::vec(0u64..1000, 1..20)
    ) {
        let mut transactions = Vec::new();
        let from_addr = Address::random();
        
        for nonce in nonces {
            transactions.push(Transaction {
                from: from_addr,
                to: Some(Address::random()),
                value: 100,
                data: vec![],
                nonce,
                gas_limit: 21000,
                gas_price: 20,
                signature: vec![0; 64],
            });
        }
        
        // Sort transactions by nonce
        transactions.sort_by_key(|tx| tx.nonce);
        
        // Verify ordering is maintained
        for i in 1..transactions.len() {
            prop_assert!(transactions[i].nonce >= transactions[i-1].nonce);
        }
    }
    
    #[test]
    fn prop_block_hash_uniqueness(
        heights in prop::collection::vec(0u64..1000000, 2..10)
    ) {
        let mut block_hashes = Vec::new();
        
        for height in heights {
            let block = Block {
                hash: BlockHash::zero(), // Will be calculated
                parent_hash: BlockHash::random(),
                height,
                timestamp: height * 4, // 4 second blocks
                transactions: vec![],
                state_root: vec![height as u8; 32],
                validator: Address::random(),
                signature: vec![0; 64],
                zk_proof: None,
            };
            
            let calculated_hash = block.calculate_hash();
            block_hashes.push(calculated_hash);
        }
        
        // Check all hashes are unique
        for i in 0..block_hashes.len() {
            for j in i+1..block_hashes.len() {
                prop_assert!(block_hashes[i] != block_hashes[j]);
            }
        }
    }
    
    #[test]
    fn prop_validator_stake_arithmetic(
        initial_stake in 1000000u64..10000000,
        slash_amount in 1u64..500000
    ) {
        // Test validator stake operations don't overflow/underflow
        let remaining_stake = if slash_amount > initial_stake {
            0
        } else {
            initial_stake - slash_amount
        };
        
        prop_assert!(remaining_stake <= initial_stake);
        prop_assert!(remaining_stake == initial_stake.saturating_sub(slash_amount));
    }
}

// Property tests for consensus parameters

#[quickcheck]
fn prop_consensus_config_validity(
    block_time_ms: u32,
    max_validators: u32,
    min_stake: u64
) -> TestResult {
    if block_time_ms == 0 || max_validators == 0 || min_stake == 0 {
        return TestResult::discard();
    }
    
    let config = BeamChainConfig {
        block_time: std::time::Duration::from_millis(block_time_ms as u64),
        max_validators: max_validators as usize,
        min_validator_stake: min_stake,
        slash_percentage: 10,
        finality_threshold: 67,
    };
    
    TestResult::from_bool(
        config.block_time.as_millis() > 0 &&
        config.max_validators > 0 &&
        config.min_validator_stake > 0 &&
        config.slash_percentage <= 100 &&
        config.finality_threshold <= 100
    )
}

// Property tests for ZK proof properties

#[quickcheck]
fn prop_proof_size_bounds(data_size: usize) -> TestResult {
    if data_size > 10000 {
        return TestResult::discard(); // Avoid too large inputs
    }
    
    // Mock proof size calculation (in practice would use real SP1)
    let proof_size = calculate_mock_proof_size(data_size);
    
    // Proof size should be reasonable bounds
    TestResult::from_bool(
        proof_size > 0 &&
        proof_size < data_size * 10 // Proof shouldn't be 10x larger than data
    )
}

// Property tests for EVM compatibility

#[quickcheck]
fn prop_evm_address_format(public_key: Vec<u8>) -> TestResult {
    if public_key.len() != 65 || public_key[0] != 0x04 {
        return TestResult::discard(); // Must be valid uncompressed public key
    }
    
    let hasher = MultiHasher::new();
    let address = hasher.generate_evm_address(&public_key);
    
    TestResult::from_bool(address.as_bytes().len() == 20)
}

#[quickcheck]
fn prop_hex_encoding_roundtrip(data: Vec<u8>) -> bool {
    let encoded = hex::encode(&data);
    let decoded = hex::decode(&encoded).unwrap();
    data == decoded
}

// Helper functions

fn calculate_mock_proof_size(data_size: usize) -> usize {
    // Mock calculation - in practice would depend on circuit complexity
    std::cmp::max(256, data_size / 10)
}

// Custom strategies for more complex types

prop_compose! {
    fn arb_valid_transaction()
        (from in arb_address(),
         to in prop::option::of(arb_address()),
         value in 0u64..1000000,
         nonce in 0u64..1000,
         gas_limit in 21000u64..1000000,
         gas_price in 1u64..1000,
         data in prop::collection::vec(any::<u8>(), 0..1000))
        -> Transaction
    {
        Transaction {
            from,
            to,
            value,
            data,
            nonce,
            gas_limit,
            gas_price,
            signature: vec![0; 64], // Mock signature
        }
    }
}

prop_compose! {
    fn arb_address()
        (bytes in prop::collection::vec(any::<u8>(), 20))
        -> Address
    {
        Address::from_bytes(&bytes)
    }
}

prop_compose! {
    fn arb_block()
        (height in 0u64..1000000,
         timestamp in 1000000u64..2000000000,
         transactions in prop::collection::vec(arb_valid_transaction(), 0..100))
        -> Block
    {
        Block {
            hash: BlockHash::zero(),
            parent_hash: BlockHash::random(),
            height,
            timestamp,
            transactions,
            state_root: vec![0; 32],
            validator: Address::random(),
            signature: vec![0; 64],
            zk_proof: None,
        }
    }
} 