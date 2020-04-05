use openssl::sha;
use serde::{ Serialize, Deserialize };
use openssl::version::version;

#[derive(Serialize, Deserialize)]
pub struct Block {
    time_stamp: u64,
    data: String,
    pre_block_hash: [u8; 32],
    cur_block_hash: [u8; 32],
}

impl Block {

    pub fn new_block(data: String, pre_block_hash: [u8; 32]) -> Self {
        let mut block = Block {
            time_stamp: std::time::SystemTime::now().elapsed().unwrap().as_secs(),
            data,
            pre_block_hash,
            cur_block_hash: [0;32]
        };
        block.set_hash();
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

    fn set_hash(&mut self) {
        let value = serde_json::to_string(self).unwrap();
        self.cur_block_hash = sha::sha256(value.as_bytes());
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

pub struct BlockHeader {
    version: i32,
    pre_block_hash: i32,
    cur_block_hash: i32,
    time_stamp: i32,
    difficult_target: u32,
    nonce: u32
}