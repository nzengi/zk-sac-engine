use serde::{Deserialize, Serialize, Deserializer, Serializer};
// Removed bincode derive - using regular serde
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(pub [u8; 20]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockHash(pub [u8; 32]);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub balance: u64,
    pub nonce: u64,
    pub code: Vec<u8>,
    pub storage: HashMap<[u8; 32], [u8; 32]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub accounts: HashMap<Address, Account>,
    pub global_nonce: u64,
    pub state_root: BlockHash,
    pub block_number: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub previous_hash: BlockHash,
    pub merkle_root: BlockHash,
    pub state_root: BlockHash,
    pub timestamp: u64,
    pub block_number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub producer: Address,
    pub extra_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub address: Address,
    pub stake: u64,
    pub public_key: Vec<u8>,
    pub performance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSet {
    pub validators: Vec<Validator>,
    pub total_stake: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProof {
    pub proof_data: Vec<u8>,
    pub public_inputs: Vec<u8>,
    pub verification_key: Vec<u8>,
    pub proof_type: ProofType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofType {
    SP1,
    Risc0,
    Plonky3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub value: u64,
    pub data: Vec<u8>,
    pub gas_limit: u64,
    pub nonce: u64,
    pub signature: Vec<u8>,
    pub sig_type: SignatureType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureType {
    Ed25519,
    Secp256k1,
    PostQuantum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolRule {
    pub rule_id: u32,
    pub rule_data: Vec<u8>,
    pub validity_proof: ZkProof,
    pub activation_epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSignature {
    pub validator_address: Address,
    pub stake_weight: u64,
    pub signature: Vec<u8>,
    pub sig_type: SignatureType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub validator_signatures: Vec<ValidatorSignature>,
    pub recursive_proof: ZkProof,
    pub protocol_updates: Vec<ProtocolRule>,
}

#[derive(Debug, Clone)]
pub struct ProtocolConfig {
    pub block_time: tokio::time::Duration,
    pub max_block_size: usize,
    pub max_transactions_per_block: usize,
    pub min_stake_threshold: u64,
    pub slashing_rate: f64,
    pub reward_rate: f64,
    pub zkvm_config: ZkVMConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkVMConfig {
    pub memory_limit: usize,
    pub execution_timeout: tokio::time::Duration,
    pub proof_compression: bool,
    pub parallel_execution: bool,
    pub max_circuits: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkVMContext {
    pub previous_state_root: BlockHash,
    pub transactions: Vec<Transaction>,
    pub block_number: u64,
    pub timestamp: u64,
    pub gas_limit: u64,
}

// Helper implementations
impl BlockHash {
    pub fn new(hash: [u8; 32]) -> Self {
        BlockHash(hash)
    }

    pub fn zero() -> Self {
        BlockHash([0; 32])
    }
    
    pub fn random() -> Self {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        BlockHash(bytes)
    }
}

impl Address {
    pub fn new(id: u8) -> Self {
        let mut addr = [0u8; 20];
        addr[19] = id;
        Address(addr)
    }

    pub fn zero() -> Self {
        Address([0; 20])
    }
    
    pub fn random() -> Self {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 20];
        rng.fill_bytes(&mut bytes);
        Address(bytes)
    }

    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        Address(bytes)
    }
}

impl Account {
    pub fn new(balance: u64) -> Self {
        Account {
            balance,
            nonce: 0,
            code: Vec::new(),
            storage: HashMap::new(),
        }
    }
}

impl Transaction {
    pub fn new(from: Address, to: Address, value: u64, nonce: u64) -> Self {
        Transaction {
            from,
            to,
            value,
            data: Vec::new(),
            gas_limit: 21000,
            nonce,
            signature: vec![0; 64],
            sig_type: SignatureType::Ed25519,
        }
    }

    pub fn with_post_quantum(from: Address, to: Address, value: u64, nonce: u64) -> Self {
        Transaction {
            from,
            to,
            value,
            data: Vec::new(),
            gas_limit: 21000,
            nonce,
            signature: Vec::new(), // LMS signatures vary in size
            sig_type: SignatureType::PostQuantum,
        }
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            accounts: HashMap::new(),
            global_nonce: 0,
            state_root: BlockHash::zero(),
            block_number: 0,
        }
    }
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            block_time: tokio::time::Duration::from_secs(4),
            max_block_size: 1_000_000, // 1MB
            max_transactions_per_block: 10_000,
            min_stake_threshold: 32_000_000_000, // 32 ETH equivalent
            slashing_rate: 0.05, // 5%
            reward_rate: 0.04, // 4% annual
            zkvm_config: ZkVMConfig::default(),
        }
    }
}

impl Default for ZkVMConfig {
    fn default() -> Self {
        Self {
            memory_limit: 1024 * 1024 * 1024, // 1GB
            execution_timeout: tokio::time::Duration::from_secs(30),
            proof_compression: true,
            parallel_execution: true,
            max_circuits: 16,
        }
    }
}



// Custom serialization for Duration to handle UNIX timestamps
impl Serialize for ProtocolConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ProtocolConfig", 8)?;
        state.serialize_field("block_time_secs", &self.block_time.as_secs())?;
        state.serialize_field("max_block_size", &self.max_block_size)?;
        state.serialize_field("max_transactions_per_block", &self.max_transactions_per_block)?;
        state.serialize_field("min_stake_threshold", &self.min_stake_threshold)?;
        state.serialize_field("slashing_rate", &self.slashing_rate)?;
        state.serialize_field("reward_rate", &self.reward_rate)?;
        state.serialize_field("zkvm_config", &self.zkvm_config)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ProtocolConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            BlockTimeSecs,
            MaxBlockSize,
            MaxTransactionsPerBlock,
            MinStakeThreshold,
            SlashingRate,
            RewardRate,
            ZkvmConfig,
        }

        struct ProtocolConfigVisitor;

        impl<'de> Visitor<'de> for ProtocolConfigVisitor {
            type Value = ProtocolConfig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ProtocolConfig")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ProtocolConfig, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut block_time_secs = None;
                let mut max_block_size = None;
                let mut max_transactions_per_block = None;
                let mut min_stake_threshold = None;
                let mut slashing_rate = None;
                let mut reward_rate = None;
                let mut zkvm_config = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::BlockTimeSecs => {
                            if block_time_secs.is_some() {
                                return Err(de::Error::duplicate_field("block_time_secs"));
                            }
                            block_time_secs = Some(map.next_value()?);
                        }
                        Field::MaxBlockSize => {
                            if max_block_size.is_some() {
                                return Err(de::Error::duplicate_field("max_block_size"));
                            }
                            max_block_size = Some(map.next_value()?);
                        }
                        Field::MaxTransactionsPerBlock => {
                            if max_transactions_per_block.is_some() {
                                return Err(de::Error::duplicate_field("max_transactions_per_block"));
                            }
                            max_transactions_per_block = Some(map.next_value()?);
                        }
                        Field::MinStakeThreshold => {
                            if min_stake_threshold.is_some() {
                                return Err(de::Error::duplicate_field("min_stake_threshold"));
                            }
                            min_stake_threshold = Some(map.next_value()?);
                        }
                        Field::SlashingRate => {
                            if slashing_rate.is_some() {
                                return Err(de::Error::duplicate_field("slashing_rate"));
                            }
                            slashing_rate = Some(map.next_value()?);
                        }
                        Field::RewardRate => {
                            if reward_rate.is_some() {
                                return Err(de::Error::duplicate_field("reward_rate"));
                            }
                            reward_rate = Some(map.next_value()?);
                        }
                        Field::ZkvmConfig => {
                            if zkvm_config.is_some() {
                                return Err(de::Error::duplicate_field("zkvm_config"));
                            }
                            zkvm_config = Some(map.next_value()?);
                        }
                    }
                }

                let block_time_secs = block_time_secs.ok_or_else(|| de::Error::missing_field("block_time_secs"))?;
                let max_block_size = max_block_size.ok_or_else(|| de::Error::missing_field("max_block_size"))?;
                let max_transactions_per_block = max_transactions_per_block.ok_or_else(|| de::Error::missing_field("max_transactions_per_block"))?;
                let min_stake_threshold = min_stake_threshold.ok_or_else(|| de::Error::missing_field("min_stake_threshold"))?;
                let slashing_rate = slashing_rate.ok_or_else(|| de::Error::missing_field("slashing_rate"))?;
                let reward_rate = reward_rate.ok_or_else(|| de::Error::missing_field("reward_rate"))?;
                let zkvm_config = zkvm_config.ok_or_else(|| de::Error::missing_field("zkvm_config"))?;

                Ok(ProtocolConfig {
                    block_time: tokio::time::Duration::from_secs(block_time_secs),
                    max_block_size,
                    max_transactions_per_block,
                    min_stake_threshold,
                    slashing_rate,
                    reward_rate,
                    zkvm_config,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["block_time_secs", "max_block_size", "max_transactions_per_block", "min_stake_threshold", "slashing_rate", "reward_rate", "zkvm_config"];
        deserializer.deserialize_struct("ProtocolConfig", FIELDS, ProtocolConfigVisitor)
    }
} 