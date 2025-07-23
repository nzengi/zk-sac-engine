// Very basic tests for types only - should compile and run

#[test]
fn test_basic_types() {
    // Just test that we can create and use basic types
    use zk_sac_engine::types::Address;
    
    let addr1 = Address::zero();
    let addr2 = Address::zero();
    
    // Zero addresses should be equal
    assert_eq!(addr1, addr2);
    
    // Should be 20 bytes of zeros
    assert_eq!(addr1.as_bytes().len(), 20);
    assert!(addr1.as_bytes().iter().all(|&b| b == 0));
} 