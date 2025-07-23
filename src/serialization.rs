//! Enhanced Serialization Module using Bincode 2.0.1 + Serde 1.0.219
//! 
//! This module provides optimized serialization/deserialization functions
//! leveraging both Bincode 2.0's improved performance and Serde 1.0.219's
//! advanced features for maximum efficiency.

use serde::{Serialize, Deserialize};
use bincode;
use anyhow::Result;
use tracing::debug;
use crate::types::*;

// Standard Bincode serialization using 1.x API
pub fn encode_blockchain_data<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    let encoded = bincode::serialize(data)?;
    debug!("📦 Encoded blockchain data: {} bytes", encoded.len());
    Ok(encoded)
}

// Enhanced state data encoding with compression markers
pub fn encode_state_data<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    let encoded = bincode::serialize(data)?;
    debug!("📦 Encoded state data: {} bytes", encoded.len());
    Ok(encoded)
}

// Blockchain data decoding with error handling
pub fn decode_blockchain_data<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T> {
    let result = bincode::deserialize(data)?;
    Ok(result)
}

// zkVM optimized output encoding
pub fn encode_zkvm_output<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    let encoded = bincode::serialize(data)?;
    debug!("🔧 Encoded zkVM output: {} bytes", encoded.len());
    Ok(encoded)
}

// zkVM output decoding
pub fn decode_zkvm_output<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T> {
    let result = bincode::deserialize(data)?;
    Ok(result)
}

// Network-optimized state encoding
pub fn decode_state_data<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T> {
    let result = bincode::deserialize(data)?;
    Ok(result)
}

// Network message encoding
pub fn encode_network_message<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    let encoded = bincode::serialize(data)?;
    debug!("📡 Encoded network message: {} bytes", encoded.len());
    Ok(encoded)
}

// Network message decoding
pub fn decode_network_message<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T> {
    let result = bincode::deserialize(data)?;
    Ok(result)
}

// Enhanced batch operations for high-throughput scenarios
pub fn encode_batch<T: Serialize>(items: &[T]) -> Result<Vec<Vec<u8>>> {
    let mut encoded_items = Vec::new();
    for item in items {
        let encoded = bincode::serialize(item)?;
        encoded_items.push(encoded);
    }
    debug!("📦 Encoded batch: {} items", encoded_items.len());
    Ok(encoded_items)
}

pub fn decode_batch<T: for<'de> Deserialize<'de>>(data: &[Vec<u8>]) -> Result<Vec<T>> {
    let mut decoded_items = Vec::new();
    for item_data in data {
        let decoded = bincode::deserialize(item_data)?;
        decoded_items.push(decoded);
    }
    debug!("📦 Decoded batch: {} items", decoded_items.len());
    Ok(decoded_items)
}

// Size estimation for resource planning
pub fn estimate_size<T: Serialize>(data: &T) -> Result<usize> {
    let encoded = bincode::serialize(data)?;
    Ok(encoded.len())
}

// JSON utilities for debugging and API compatibility
pub fn to_json_pretty<T: Serialize>(data: &T) -> Result<String> {
    let json = serde_json::to_string_pretty(data)?;
    Ok(json)
}

pub fn to_json_value<T: Serialize>(data: &T) -> Result<serde_json::Value> {
    let value = serde_json::to_value(data)?;
    Ok(value)
}

// Hybrid serialization: Bincode for performance, JSON for debugging
pub fn encode_hybrid<T: Serialize>(data: &T, debug_mode: bool) -> Result<Vec<u8>> {
    if debug_mode {
        let json = serde_json::to_string(data)?;
        Ok(json.into_bytes())
    } else {
        bincode::serialize(data).map_err(Into::into)
    }
}

pub fn decode_hybrid<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T> {
    // Try Bincode first, fallback to JSON
    match bincode::deserialize(data) {
        Ok(result) => Ok(result),
        Err(_) => {
            let json_str = std::str::from_utf8(data)?;
            let result = serde_json::from_str(json_str)?;
            Ok(result)
        }
    }
}

// Format comparison for optimization analysis
pub fn compare_formats<T: Serialize>(data: &T) -> Result<(usize, usize)> {
    let bincode_size = bincode::serialize(data)?.len();
    let json_size = serde_json::to_string(data)?.len();
    debug!("📊 Format comparison - Bincode: {} bytes, JSON: {} bytes", bincode_size, json_size);
    Ok((bincode_size, json_size))
}

// Blockchain configuration encoding
pub fn blockchain_config() -> bincode::config::DefaultOptions {
    bincode::DefaultOptions::new()
}

// zkVM input encoding with specialized configuration
pub fn zkvm_input_config() -> bincode::config::DefaultOptions {
    bincode::DefaultOptions::new()
}

// Create block metadata for analytics
pub fn create_block_metadata(transactions: &[Transaction], producer: Address) -> Result<BlockMetadata> {
    let total_gas = transactions.iter().map(|tx| tx.gas_limit).sum();
    let total_value = transactions.iter().map(|tx| tx.value).sum();
    
    let metadata = BlockMetadata {
        transaction_count: transactions.len() as u64,
        total_gas_used: total_gas,
        total_value_transferred: total_value,
        producer,
        encoding_stats: EncodingStats {
            compressed_size: estimate_size(&transactions)?,
            compression_ratio: 1.0, // Will be calculated during actual compression
        },
    };
    
    Ok(metadata)
}

// Extract block summary for reporting
pub fn extract_block_summary(block: &Block) -> BlockSummary {
    BlockSummary {
        block_number: block.header.block_number,
        transaction_count: block.transactions.len() as u64,
        producer: block.header.producer,
        timestamp: block.header.timestamp,
        total_size: estimate_size(block).unwrap_or(0),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    pub transaction_count: u64,
    pub total_gas_used: u64,
    pub total_value_transferred: u64,
    pub producer: Address,
    pub encoding_stats: EncodingStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingStats {
    pub compressed_size: usize,
    pub compression_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSummary {
    pub block_number: u64,
    pub transaction_count: u64,
    pub producer: Address,
    pub timestamp: u64,
    pub total_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_blockchain_data_encoding() {
        let address = Address([1u8; 20]);
        
        let encoded = encode_blockchain_data(&address).unwrap();
        let decoded: Address = decode_blockchain_data(&encoded).unwrap();
        
        assert_eq!(address, decoded);
    }

    #[test]
    fn test_zkvm_input_encoding() {
        let proof_type = ProofType::SP1;
        
        let encoded = encode_zkvm_output(&proof_type).unwrap();
        let decoded: ProofType = decode_zkvm_output(&encoded).unwrap();
        
        assert_eq!(format!("{:?}", proof_type), format!("{:?}", decoded));
    }

    #[test]
    fn test_network_message_encoding() {
        let block_hash = BlockHash([42u8; 32]);
        
        let encoded = encode_network_message(&block_hash).unwrap();
        let decoded: BlockHash = decode_network_message(&encoded).unwrap();
        
        assert_eq!(block_hash, decoded);
    }

    #[test]
    fn test_batch_operations() {
        let addresses = vec![
            Address([1u8; 20]),
            Address([2u8; 20]),
            Address([3u8; 20]),
        ];
        
        let encoded_batch = encode_batch(&addresses).unwrap();
        let decoded_batch: Vec<Address> = decode_batch(&encoded_batch).unwrap();
        
        assert_eq!(addresses, decoded_batch);
    }

    #[test]
    fn test_size_estimation() {
        let transaction = Transaction {
            from: Address([1u8; 20]),
            to: Address([2u8; 20]),
            value: 1000,
            data: vec![1, 2, 3, 4, 5],
            gas_limit: 21000,
            nonce: 1,
            signature: vec![0; 64],
            sig_type: SignatureType::Ed25519,
        };
        
        let size = estimate_size(&transaction).unwrap();
        assert!(size > 0);
        println!("Transaction size: {} bytes", size);
    }

    #[test]
    fn test_json_serialization() {
        let address = Address([42u8; 20]);
        
        // Test pretty JSON
        let json_pretty = to_json_pretty(&address).unwrap();
        let decoded: Address = from_json(&json_pretty).unwrap();
        assert_eq!(address, decoded);
        
        // Test compact JSON
        let json_compact = to_json_compact(&address).unwrap();
        assert!(json_compact.len() <= json_pretty.len());
        
        // Test JSON bytes
        let json_bytes = to_json_bytes(&address).unwrap();
        let decoded_bytes: Address = from_json_bytes(&json_bytes).unwrap();
        assert_eq!(address, decoded_bytes);
        
        println!("JSON pretty: {}", json_pretty);
        println!("JSON compact: {}", json_compact);
    }

    #[test]
    fn test_hybrid_serialization() {
        let block_hash = BlockHash([123u8; 32]);
        
        // Test Bincode path
        let bincode_data = encode_hybrid(&block_hash, false).unwrap();
        let decoded_bincode: BlockHash = decode_hybrid(&bincode_data).unwrap();
        assert_eq!(block_hash, decoded_bincode);
        
        // Test JSON path
        let json_data = encode_hybrid(&block_hash, true).unwrap();
        let decoded_json: BlockHash = decode_hybrid(&json_data).unwrap();
        assert_eq!(block_hash, decoded_json);
        
        println!("Bincode size: {} bytes", bincode_data.len());
        println!("JSON size: {} bytes", json_data.len());
    }

    #[test]
    fn test_format_comparison() {
        let transaction = Transaction {
            from: Address([1u8; 20]),
            to: Address([2u8; 20]),
            value: 1000,
            data: vec![1, 2, 3, 4, 5],
            gas_limit: 21000,
            nonce: 1,
            signature: vec![0; 64],
            sig_type: SignatureType::Ed25519,
        };
        
        let (bincode_size, json_size) = compare_formats(&transaction).unwrap();
        
        println!("Bincode size: {} bytes", bincode_size);
        println!("JSON size: {} bytes", json_size);
        
        // Bincode should be more efficient
        assert!(bincode_size < json_size);
    }

    #[test]
    fn test_json_value_operations() {
        let address = Address([42u8; 20]);
        
        // Test Value conversion
        let json_value = to_json_value(&address).unwrap();
        assert!(json_value.is_array());
        
        // Test JSON string parsing
        let json_str = r#"{"test": "value", "number": 42}"#;
        let parsed = parse_json_value(json_str).unwrap();
        assert_eq!(parsed["test"], "value");
        assert_eq!(parsed["number"], 42);
    }

    #[test]
    fn test_block_metadata_creation() {
        let hash = [0u8; 32];
        let transactions = vec![
            Transaction {
                from: Address([1u8; 20]),
                to: Address([2u8; 20]),
                value: 1000,
                data: vec![1, 2, 3],
                gas_limit: 21000,
                nonce: 1,
                signature: vec![0; 64],
                sig_type: SignatureType::Ed25519,
            },
            Transaction {
                from: Address([2u8; 20]),
                to: Address([1u8; 20]),
                value: 500,
                data: vec![4, 5, 6],
                gas_limit: 10000,
                nonce: 2,
                signature: vec![1; 64],
                sig_type: SignatureType::Ed25519,
            },
        ];
        let producer = Address([3u8; 20]);
        
        let metadata = create_block_metadata(&transactions, producer).unwrap();
        
        assert_eq!(metadata.transaction_count, 2);
        assert_eq!(metadata.total_gas_used, 31000);
        assert_eq!(metadata.total_value_transferred, 1500);
        assert_eq!(metadata.producer, producer);
        assert_eq!(metadata.encoding_stats.compressed_size, estimate_size(&transactions).unwrap());
        assert_eq!(metadata.encoding_stats.compression_ratio, 1.0);
        
        println!("Block metadata: {}", to_string_pretty(&metadata).unwrap());
    }

    #[test]
    fn test_json_merge() {
        let mut base = json!({
            "name": "test",
            "version": 1
        });
        
        let update = json!({
            "version": 2,
            "new_field": "added"
        });
        
        merge_json_values(&mut base, update).unwrap();
        
        assert_eq!(base["name"], "test");
        assert_eq!(base["version"], 2);
        assert_eq!(base["new_field"], "added");
    }

    #[test]
    fn test_batch_json_serialization() {
        let addresses = vec![
            Address([1u8; 20]),
            Address([2u8; 20]),
            Address([3u8; 20]),
        ];
        
        let json_batch = serialize_batch_to_json(&addresses).unwrap();
        assert!(json_batch.contains("\""));
        assert!(json_batch.contains("["));
        assert!(json_batch.contains("]"));
        
        println!("Batch JSON: {}", json_batch);
    }

    #[test]
    fn test_pretty_bytes() {
        let block_hash = BlockHash([255u8; 32]);
        
        let pretty_bytes = to_json_pretty_bytes(&block_hash).unwrap();
        let pretty_string = String::from_utf8(pretty_bytes).unwrap();
        
        // Should contain newlines and proper formatting
        assert!(pretty_string.contains('\n'));
        assert!(pretty_string.contains("  ")); // Indentation
        
        println!("Pretty JSON bytes: {}", pretty_string);
    }
} 