use zk_sac_engine::consensus::engine::{ZkSacConsensusEngine, ConsensusEngine};
use zk_sac_engine::types::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting ZK-SAC Engine Demo");
    
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create genesis state
    let mut accounts = HashMap::new();
    accounts.insert(
        Address::new(1),
        Account {
            balance: 1_000_000,
            nonce: 0,
            code: Vec::new(),
            storage: HashMap::new(),
        }
    );
    
    let genesis_state = WorldState {
        accounts,
        global_nonce: 0,
        state_root: BlockHash::zero(),
        block_number: 0,
    };
    
    // Create validators
    let validators = vec![
        Validator {
            address: Address::new(1),
            stake: 32_000_000_000,
            public_key: vec![1; 32],
            performance_score: 1.0,
        },
        Validator {
            address: Address::new(2), 
            stake: 16_000_000_000,
            public_key: vec![2; 32],
            performance_score: 0.9,
        },
        Validator {
            address: Address::new(3),
            stake: 8_000_000_000,
            public_key: vec![3; 32],
            performance_score: 0.8,
        },
    ];
    
    // Create consensus engine
    let mut engine = ZkSacConsensusEngine::new(
        genesis_state, 
        validators, 
        ProtocolConfig::default()
    )?;
    
    println!("✅ Consensus engine initialized");
    
    // Create test transactions
    let transactions = vec![
        Transaction {
            from: Address::new(1),
            to: Address::new(2),
            value: 1000,
            data: vec![],
            gas_limit: 21000,
            nonce: 0,
            signature: vec![0; 64],
            sig_type: SignatureType::Ed25519,
        },
        Transaction {
            from: Address::new(2),
            to: Address::new(3),
            value: 500,
            data: vec![],
            gas_limit: 21000,
            nonce: 0,
            signature: vec![0; 64],
            sig_type: SignatureType::Ed25519,
        },
    ];
    
    // Add transactions to pending pool
    for tx in transactions {
        engine.pending_transactions.push(tx);
    }
    
    println!("📝 Added {} transactions to pool", engine.pending_transactions.len());
    
    // Select block producer
    let producer = engine.select_block_producer(1)?;
    println!("🎯 Selected block producer: {:?}", producer);
    
    // Produce a block
    let block = engine.produce_block(producer)?;
    println!("🔨 Produced block {}", block.header.block_number);
    
    // Validate the block
    let is_valid = engine.validate_block(&block)?;
    println!("🔍 Block validation result: {}", is_valid);
    
    // Apply the block to the chain
    if is_valid {
        engine.apply_block(block)?;
        println!("✅ Block applied to chain");
        println!("📊 Total blocks in chain: {}", engine.blocks.len());
    }
    
    println!("🎉 ZK-SAC Engine demo completed successfully!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_full_consensus_flow() -> Result<(), String> {
        // Create a minimal test setup
        let mut accounts = HashMap::new();
        accounts.insert(Address::new(1), Account::new(1000));
        accounts.insert(Address::new(2), Account::new(1000));
        
        let genesis_state = WorldState {
            accounts,
            contract_storage: HashMap::new(),
            protocol_rules: HashMap::new(),
            state_root: BlockHash::new([0; 32]),
        };
        
        let validators = vec![
            Validator {
                address: Address::new(10),
                stake: 1000000,
                public_key: vec![0; 32],
                is_active: true,
            }
        ];
        
        let mut engine = ZkSacEngine::new(genesis_state, validators);
        
        // Add transaction
        let tx = Transaction::new(Address::new(1), Address::new(2), 100, 0);
        engine.add_transaction(tx)?;
        
        // Produce block
        let producer = engine.select_block_producer(1)?;
        let mut block = engine.produce_block(producer)?;
        
        // Collect signatures and verify
        engine.collect_validator_signatures(&mut block)?;
        engine.verify_and_add_block(block)?;
        
        assert_eq!(engine.get_block_count(), 1);
        assert_eq!(engine.get_account_balance(&Address::new(1)), 900);
        assert_eq!(engine.get_account_balance(&Address::new(2)), 1100);
        
        Ok(())
    }
    
    #[test]
    fn test_recursive_proof_chain() -> Result<(), String> {
        let mut accounts = HashMap::new();
        accounts.insert(Address::new(1), Account::new(1000));
        
        let genesis_state = WorldState {
            accounts,
            contract_storage: HashMap::new(),
            protocol_rules: HashMap::new(),
            state_root: BlockHash::new([0; 32]),
        };
        
        let validators = vec![
            Validator {
                address: Address::new(10),
                stake: 1000000,
                public_key: vec![0; 32],
                is_active: true,
            }
        ];
        
        let mut engine = ZkSacEngine::new(genesis_state, validators);
        
        // Produce multiple blocks to test recursive proofs
        for i in 0..3 {
            let tx = Transaction::new(Address::new(1), Address::new(1), 1, i);
            engine.add_transaction(tx)?;
            
            let producer = engine.select_block_producer(i + 1)?;
            let mut block = engine.produce_block(producer)?;
            engine.collect_validator_signatures(&mut block)?;
            engine.verify_and_add_block(block)?;
        }
        
        assert_eq!(engine.get_block_count(), 3);
        
        // Each block should have a recursive proof that proves the entire chain
        for (i, block) in engine.blocks.iter().enumerate() {
            assert!(!block.recursive_proof.proof_data.is_empty());
            println!("Block {} recursive proof size: {} bytes", 
                     i + 1, block.recursive_proof.proof_data.len());
        }
        
        Ok(())
    }
} 