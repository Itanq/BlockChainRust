use openssl::sha;
use serde::{ Serialize, Deserialize };
use bigint::uint;
use openssl::sha::sha256;

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    time_stamp: u64,
    data: String,
    pre_block_hash: [u8; 32],
    cur_block_hash: [u8; 32],
    target_bits: u8,
    nonce: u32,
}

impl Block {

    pub fn new_block(data: String, pre_block_hash: [u8; 32]) -> Self {
        let mut block = Block {
            time_stamp: std::time::SystemTime::now().elapsed().unwrap().as_secs(),
            data,
            pre_block_hash,
            cur_block_hash: [0;32],
            target_bits: 16,
            nonce: 0,
        };
        block.cur_block_hash = Block::proof_of_work(&mut block);
        block
    }

    pub fn cur_block_hash(&self) -> String {
        hex::encode(self.cur_block_hash.to_vec())
    }

    pub fn pre_block_hash(&self) -> String {
        hex::encode(self.pre_block_hash.to_vec())
    }

    pub fn data(&self) -> &String {
        &self.data
    }
}

pub struct BlockChain { blocks: Vec<Block> }

impl BlockChain {
    pub fn genesis_block() -> Self {
        let block = Block::new_block("Genesis Block".to_string(), [0;32]);
        BlockChain {
            blocks: vec![ block ]
        }
    }

    pub fn add_block(&mut self, data: String) {
        let pre_block = self.blocks.get(self.blocks.len() - 1).unwrap();
        let new_block = Block::new_block(data, pre_block.cur_block_hash);
        self.blocks.push(new_block);
    }

    pub fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
}

trait ProofOfWork {
    fn proof_of_work(&mut self) -> [u8;32];
}

impl ProofOfWork for Block {

    fn proof_of_work(&mut self) -> [u8; 32] {
        let one = uint::U256::one();
        let target = one << ( 256 - self.target_bits as usize );

        while self.nonce < u32::MAX {
            let value = serde_json::to_string(&self).unwrap_or("".to_string());
            let hash = sha256(value.as_bytes());
            let hashInt = uint::U256::from(hash);
            if hashInt < target {
                println!("Find the valid hash: {}", hex::encode(hash));
                return hash;
            } else {
                self.nonce += 1;
            }
        }
        [0;32]
    }
}

pub struct BlockHeader {
    version: i32,
    pre_block_hash: i32,
    cur_block_hash: i32,
    time_stamp: i32,
    difficult_target: u32,
    nonce: u32
}