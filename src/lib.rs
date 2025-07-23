pub mod types;
pub mod consensus;
pub mod crypto;
pub mod zkvm;
pub mod performance;
pub mod serialization;
pub mod async_utils;

pub use types::*;
pub use consensus::engine::{ZkSacConsensusEngine, ConsensusEngine};

// Re-export commonly used items
pub use anyhow::Result;
pub use tracing::{info, warn, error, debug};

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    use std::time::Duration;

    #[tokio::test]
    async fn test_module_integration() {
        // Test that all modules can be imported and basic types work together
        let address = types::Address::random();
        let hasher = crypto::hash::MultiHasher::new();
        
        // Test address can be hashed
        let hash = hasher.blake3_hash(address.as_bytes());
        assert_eq!(hash.len(), 32);
        
        // Test address can be serialized
        let serialized = serialization::serialize_address(&address).unwrap();
        let deserialized = serialization::deserialize_address(&serialized).unwrap();
        assert_eq!(address, deserialized);
    }

    #[tokio::test]
    async fn test_async_utils_basic_functionality() {
        let pool = async_utils::AsyncTaskPool::new(2);
        
        let result = pool.spawn(async {
            42
        }).await.unwrap();
        
        assert_eq!(result, 42);
    }

    #[test]
    fn test_types_basic_operations() {
        let addr1 = types::Address::random();
        let addr2 = types::Address::random();
        
        assert_ne!(addr1, addr2);
        
        let zero = types::Address::zero();
        assert_eq!(zero.as_bytes(), &[0u8; 20]);
    }

    #[test]
    fn test_crypto_hash_consistency() {
        let hasher = crypto::hash::MultiHasher::new();
        let data = b"test_data";
        
        let hash1 = hasher.blake3_hash(data);
        let hash2 = hasher.blake3_hash(data);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let transaction = types::Transaction {
            from: types::Address::random(),
            to: Some(types::Address::random()),
            value: 1000,
            data: vec![1, 2, 3, 4],
            nonce: 42,
            gas_limit: 21000,
            gas_price: 20,
            signature: vec![0; 64],
        };

        let serialized = serialization::serialize_transaction(&transaction).unwrap();
        let deserialized = serialization::deserialize_transaction(&serialized).unwrap();
        
        assert_eq!(transaction.from, deserialized.from);
        assert_eq!(transaction.value, deserialized.value);
        assert_eq!(transaction.nonce, deserialized.nonce);
    }

    #[tokio::test]
    async fn test_consensus_config_creation() {
        let config = consensus::BeamChainConfig::new_for_testing();
        
        assert!(config.block_time.as_secs() > 0);
        assert!(config.max_validators > 0);
        assert!(config.min_validator_stake > 0);
    }

    #[cfg(feature = "risc0")]
    #[tokio::test]
    async fn test_zkvm_basic_creation() {
        let result = zkvm::Risc0Executor::new();
        assert!(result.is_ok());
    }
} 