// Basic tests that should compile and run
use zk_sac_engine::types::{Address, Transaction};

#[test]
fn test_address_creation() {
    let addr1 = Address::random();
    let addr2 = Address::random();
    
    // Addresses should be different
    assert_ne!(addr1, addr2);
    
    // Zero address should be all zeros
    let zero = Address::zero();
    assert_eq!(zero.as_bytes(), &[0u8; 20]);
}

#[test] 
fn test_transaction_creation() {
    let tx = Transaction {
        from: Address::random(),
        to: Some(Address::random()),
        value: 1000,
        data: vec![1, 2, 3],
        nonce: 5,
        gas_limit: 21000,
        gas_price: 20,
        signature: vec![0; 64],
    };
    
    assert_eq!(tx.value, 1000);
    assert_eq!(tx.nonce, 5);
    assert_eq!(tx.data.len(), 3);
}

#[test]
fn test_basic_hashing() {
    use zk_sac_engine::crypto::hash::MultiHasher;
    
    let hasher = MultiHasher::new();
    let data = b"test_data";
    
    // Blake3 test
    let hash1 = hasher.blake3_hash(data);
    let hash2 = hasher.blake3_hash(data);
    
    // Same data should produce same hash
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 32);
} 