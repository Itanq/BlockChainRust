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

impl ToString for Block {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

pub struct BlockChain {
    header: [u8; 32],
    db: sled::Db,
}

impl BlockChain {
    pub fn new_block_chain() -> Self {
        let db = sled::open("block_chain_db").unwrap();
        let hash: Vec<u8> = if let Some(hash) = db.get("last").unwrap() {
            hash.to_vec()
        } else {
            println!("Genesis Block....");
            let block = Block::new_block("Genesis Block".to_string(), [0;32]);
            db.insert(block.cur_block_hash, &block.to_string()[..]);
            db.insert("last", &block.cur_block_hash);
            block.cur_block_hash.to_vec()
        };

        let mut header = [0u8; 32];
        header.copy_from_slice(&hash[..hash.len()]);

        BlockChain {
            header,
            db,
        }
    }

    pub fn add_block(&mut self, data: String) {
        let mut pre_block_hash: [u8; 32] = [0u8;32];
        pre_block_hash.copy_from_slice(&self.db.get("last").unwrap().unwrap().to_vec()[..]);
        let new_block = Block::new_block(data, pre_block_hash);
        self.db.insert(new_block.cur_block_hash, &new_block.to_string()[..]);
        self.db.insert("last", &new_block.cur_block_hash);
    }

    pub fn print(&self) {
        if let Some(hash) = self.db.get("last").unwrap() {
            let mut block = self.get_block(&hash.to_vec());
            while block.pre_block_hash != [0u8; 32] {
                println!("Prev Hash: {:?}", block.pre_block_hash());
                println!("Data: {}", block.data);
                println!("Cur Hash: {:?}\n", block.cur_block_hash());
                block =  self.get_block(&block.pre_block_hash);
            }
            println!("Prev Hash: {:?}", block.pre_block_hash());
            println!("Data: {}", block.data);
            println!("Cur Hash: {:?}\n", block.cur_block_hash());
        }
    }

    fn get_block(&self, hash: &[u8]) -> Block {
        serde_json::from_slice(&self.db.get(hash).unwrap().unwrap().to_vec()).unwrap()
    }
}


trait ProofOfWork {
    fn proof_of_work(&mut self) -> [u8;32];
}

impl ProofOfWork for Block {

    fn proof_of_work(&mut self) -> [u8; 32] {
        let one = uint::U256::one();
        let target = one << ( 256 - self.target_bits as usize );

        while self.nonce < std::u32::MAX {
            let value = serde_json::to_string(&self).unwrap_or("".to_string());
            let hash = sha256(value.as_bytes());
            let hashInt = uint::U256::from(hash);
            if hashInt < target {
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