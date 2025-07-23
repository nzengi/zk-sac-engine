use blake3::{self, Hasher as Blake3Hasher};
use sha3::{Digest, Sha3_256, Sha3_512, Keccak256, Shake128, Shake256, digest::{ExtendableOutput, XofReader, Update}};
use hex;

/// Enhanced Blake3 hash using version 1.8.2 features
pub fn blake3_hash(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

/// Blake3 hash with extended output for ZK proofs
pub fn blake3_hash_extended(data: &[u8], output_len: usize) -> Vec<u8> {
    let mut hasher = Blake3Hasher::new();
    hasher.update(data);
    let mut output_reader = hasher.finalize_xof();
    let mut output = vec![0u8; output_len];
    output_reader.fill(&mut output);
    output
}

/// Blake3 keyed hash for authenticated operations
pub fn blake3_keyed_hash(key: &[u8; 32], data: &[u8]) -> [u8; 32] {
    *blake3::keyed_hash(key, data).as_bytes()
}

/// Blake3 key derivation for generating multiple keys from one master key
pub fn blake3_derive_key(context: &str, key_material: &[u8], output_len: usize) -> Vec<u8> {
    let derived = blake3::derive_key(context, key_material);
    if output_len == 32 {
        derived.to_vec()
    } else {
        // Use extended output for non-standard lengths
        let mut hasher = Blake3Hasher::new();
        hasher.update(&derived);
        let mut output_reader = hasher.finalize_xof();
        let mut output = vec![0u8; output_len];
        output_reader.fill(&mut output);
        output
    }
}

/// Incremental Blake3 hasher for streaming data
pub struct IncrementalHasher {
    hasher: Blake3Hasher,
}

impl IncrementalHasher {
    pub fn new() -> Self {
        Self {
            hasher: Blake3Hasher::new(),
        }
    }
    
    pub fn new_keyed(key: &[u8; 32]) -> Self {
        Self {
            hasher: Blake3Hasher::new_keyed(key),
        }
    }
    
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }
    
    pub fn finalize(self) -> [u8; 32] {
        *self.hasher.finalize().as_bytes()
    }
    
    pub fn finalize_extended(self, output_len: usize) -> Vec<u8> {
        let mut output_reader = self.hasher.finalize_xof();
        let mut output = vec![0u8; output_len];
        output_reader.fill(&mut output);
        output
    }
}

/// SHA3-256 hash for post-quantum security
pub fn sha3_256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    Digest::update(&mut hasher, data);
    hasher.finalize().into()
}

/// SHA3-512 hash for enhanced security
pub fn sha3_512_hash(data: &[u8]) -> [u8; 64] {
    let mut hasher = Sha3_512::new();
    Digest::update(&mut hasher, data);
    hasher.finalize().into()
}

/// Keccak256 hash for EVM compatibility
pub fn keccak256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    Digest::update(&mut hasher, data);
    hasher.finalize().into()
}

/// SHAKE128 with extendable output for ZK applications
pub fn shake128_hash(data: &[u8], output_len: usize) -> Vec<u8> {
    let mut hasher = Shake128::default();
    hasher.update(data);
    let mut reader = hasher.finalize_xof();
    let mut output = vec![0u8; output_len];
    reader.read(&mut output);
    output
}

/// SHAKE256 with extendable output for enhanced security
pub fn shake256_hash(data: &[u8], output_len: usize) -> Vec<u8> {
    let mut hasher = Shake256::default();
    hasher.update(data);
    let mut reader = hasher.finalize_xof();
    let mut output = vec![0u8; output_len];
    reader.read(&mut output);
    output
}

/// Enhanced Merkle tree using Blake3 1.8.2 incremental hashing
pub fn merkle_root(leaves: &[Vec<u8>]) -> [u8; 32] {
    if leaves.is_empty() {
        return [0; 32];
    }
    
    if leaves.len() == 1 {
        return blake3_hash(&leaves[0]);
    }
    
    // Optimized merkle tree using incremental hashing
    let mut level = leaves.iter().map(|leaf| blake3_hash(leaf)).collect::<Vec<_>>();
    
    while level.len() > 1 {
        let mut next_level = Vec::new();
        
        for chunk in level.chunks(2) {
            if chunk.len() == 2 {
                // Use incremental hasher for better performance
                let mut hasher = IncrementalHasher::new();
                hasher.update(&chunk[0]);
                hasher.update(&chunk[1]);
                next_level.push(hasher.finalize());
            } else {
                next_level.push(chunk[0]);
            }
        }
        
        level = next_level;
    }
    
    level[0]
}

/// Enhanced state root computation using Keccak256 for EVM compatibility
pub fn compute_state_root_enhanced(updates: &[([u8; 32], [u8; 32])], prev_root: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    
    // Start with previous root
    Digest::update(&mut hasher, prev_root);
    
    // Apply updates in a deterministic way
    for (key, value) in updates {
        Digest::update(&mut hasher, key);
        Digest::update(&mut hasher, value);
    }
    
    hasher.finalize().into()
}

/// EVM-compatible address generation from public key
pub fn public_key_to_address(public_key: &[u8; 64]) -> [u8; 20] {
    let hash = keccak256_hash(public_key);
    let mut address = [0u8; 20];
    address.copy_from_slice(&hash[12..32]); // Last 20 bytes
    address
}

/// EVM-compatible transaction hash
pub fn compute_transaction_hash_evm(
    nonce: u64,
    gas_price: u64, 
    gas_limit: u64,
    to: &[u8; 20],
    value: u64,
    data: &[u8]
) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    Digest::update(&mut hasher, &nonce.to_be_bytes());
    Digest::update(&mut hasher, &gas_price.to_be_bytes());
    Digest::update(&mut hasher, &gas_limit.to_be_bytes());
    Digest::update(&mut hasher, to);
    Digest::update(&mut hasher, &value.to_be_bytes());
    Digest::update(&mut hasher, data);
    hasher.finalize().into()
}

/// ZK-friendly hash function using SHAKE256 for optimal ZK performance
pub fn zk_hash_extended(data: &[u8], field_size_bits: usize) -> Vec<u8> {
    // Calculate required bytes for field elements
    let output_bytes = (field_size_bits + 7) / 8;
    shake256_hash(data, output_bytes)
}

/// Poseidon-like hash using SHAKE128 for ZK circuits
pub fn zk_poseidon_like_hash(inputs: &[[u8; 32]], output_len: usize) -> Vec<u8> {
    let mut hasher = Shake128::default();
    
    // Add all inputs
    for input in inputs {
        hasher.update(input);
    }
    
    let mut reader = hasher.finalize_xof();
    let mut output = vec![0u8; output_len];
    reader.read(&mut output);
    output
}

/// Multi-hash verification for consensus
pub fn compute_consensus_hash(data: &[u8]) -> ([u8; 32], [u8; 32], [u8; 32]) {
    (
        blake3_hash(data),        // Performance
        keccak256_hash(data),     // EVM compatibility  
        sha3_256_hash(data),      // Post-quantum security
    )
}

/// Enhanced hex utilities using hex 0.4.3 with serde support
pub mod hex_utils {
    use super::*;
    use anyhow::{Result, anyhow};

    /// Encode hash to lowercase hex string
    pub fn hash_to_hex(hash: &[u8]) -> String {
        hex::encode(hash)
    }

    /// Encode hash to uppercase hex string (for display)
    pub fn hash_to_hex_upper(hash: &[u8]) -> String {
        hex::encode_upper(hash)
    }

    /// Decode hex string to hash bytes
    pub fn hex_to_hash(hex_str: &str) -> Result<Vec<u8>> {
        hex::decode(hex_str)
            .map_err(|e| anyhow!("Invalid hex string: {}", e))
    }

    /// Decode hex string to fixed-size hash
    pub fn hex_to_hash_32(hex_str: &str) -> Result<[u8; 32]> {
        let bytes = hex_to_hash(hex_str)?;
        if bytes.len() != 32 {
            return Err(anyhow!("Hash must be 32 bytes, got {}", bytes.len()));
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        Ok(hash)
    }

    /// Decode hex string to 20-byte address
    pub fn hex_to_address(hex_str: &str) -> Result<[u8; 20]> {
        let bytes = hex_to_hash(hex_str)?;
        if bytes.len() != 20 {
            return Err(anyhow!("Address must be 20 bytes, got {}", bytes.len()));
        }
        let mut address = [0u8; 20];
        address.copy_from_slice(&bytes);
        Ok(address)
    }

    /// Format hash with 0x prefix for EVM compatibility
    pub fn hash_to_hex_prefixed(hash: &[u8]) -> String {
        format!("0x{}", hex::encode(hash))
    }

    /// Parse EVM-style hex string (with or without 0x prefix)
    pub fn parse_evm_hex(hex_str: &str) -> Result<Vec<u8>> {
        let cleaned = if hex_str.starts_with("0x") || hex_str.starts_with("0X") {
            &hex_str[2..]
        } else {
            hex_str
        };
        hex_to_hash(cleaned)
    }

    /// Encode multiple hashes as hex array
    pub fn hashes_to_hex_array(hashes: &[[u8; 32]]) -> Vec<String> {
        hashes.iter().map(|h| hash_to_hex(h)).collect()
    }

    /// Encode hash with checksum (simple checksum for validation)
    pub fn hash_to_hex_with_checksum(hash: &[u8]) -> String {
        let hex_str = hex::encode(hash);
        let checksum = blake3_hash(hex_str.as_bytes());
        format!("{}:{}", hex_str, hex::encode(&checksum[0..4]))
    }

    /// Validate and decode hex with checksum
    pub fn hex_with_checksum_to_hash(hex_str: &str) -> Result<Vec<u8>> {
        let parts: Vec<&str> = hex_str.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid checksum format"));
        }

        let hash = hex_to_hash(parts[0])?;
        let provided_checksum = hex_to_hash(parts[1])?;
        
        let expected_checksum = blake3_hash(parts[0].as_bytes());
        if provided_checksum != expected_checksum[0..4] {
            return Err(anyhow!("Checksum validation failed"));
        }

        Ok(hash)
    }
}

/// Hash comparison utilities
pub mod hash_compare {
    use super::*;

    /// Compare two hashes in constant time
    pub fn constant_time_compare(a: &[u8; 32], b: &[u8; 32]) -> bool {
        let mut result = 0u8;
        for i in 0..32 {
            result |= a[i] ^ b[i];
        }
        result == 0
    }

    /// Find the closest hash in a set (for DHT-like operations)
    pub fn find_closest_hash(target: &[u8; 32], candidates: &[[u8; 32]]) -> Option<usize> {
        if candidates.is_empty() {
            return None;
        }

        let mut best_index = 0;
        let mut best_distance = hash_distance(target, &candidates[0]);

        for (i, candidate) in candidates.iter().enumerate().skip(1) {
            let distance = hash_distance(target, candidate);
            if distance < best_distance {
                best_distance = distance;
                best_index = i;
            }
        }

        Some(best_index)
    }

    /// Calculate XOR distance between two hashes
    fn hash_distance(a: &[u8; 32], b: &[u8; 32]) -> u32 {
        let mut distance = 0u32;
        for i in 0..32 {
            distance += (a[i] ^ b[i]).count_ones();
        }
        distance
    }

    /// Check if hash meets difficulty target (leading zeros)
    pub fn meets_difficulty(hash: &[u8; 32], difficulty: usize) -> bool {
        if difficulty == 0 {
            return true;
        }

        let bytes_to_check = difficulty / 8;
        let bits_to_check = difficulty % 8;

        // Check full bytes
        for i in 0..bytes_to_check {
            if hash[i] != 0 {
                return false;
            }
        }

        // Check remaining bits
        if bits_to_check > 0 && bytes_to_check < 32 {
            let mask = 0xFF << (8 - bits_to_check);
            if (hash[bytes_to_check] & mask) != 0 {
                return false;
            }
        }

        true
    }
} 