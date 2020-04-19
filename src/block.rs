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
    id: [u8; 32],
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

        let mut id = [0u8; 32];
        id.copy_from_slice(&hash[..hash.len()]);

        BlockChain {
            id,
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

    pub fn iter(&self) -> BlockChainIter {
        let mut cur_hash = [0u8; 32];
        cur_hash.copy_from_slice(&self.db.get("last").unwrap().unwrap().to_vec());

        BlockChainIter {
            cur_hash,
            db: self.db.clone()
        }
    }

    fn get_block(&self, hash: &[u8]) -> Block {
        serde_json::from_slice(&self.db.get(hash).unwrap().unwrap()).unwrap()
    }

    pub fn print(&self) {
        let mut iter = self.iter();
        while let Some(bc) = iter.next() {
            println!("Prev Hash: {:?}", bc.pre_block_hash());
            println!("Curr Hash: {:?}", bc.cur_block_hash());
            println!("Data: {}\n", bc.data);
        }
    }
}

pub struct BlockChainIter {
    cur_hash: [u8; 32],
    db: sled::Db,
}

impl BlockChainIter {
    fn next(&mut self) -> Option<Block> {
        if self.cur_hash == [0u8; 32] {
            return None;
        }
        if let Ok(block) = serde_json::from_slice::<Block>(&self.db.get(self.cur_hash).unwrap().unwrap()) {
            self.cur_hash = block.pre_block_hash;
            Some(block)
        } else {
            None
        }
    }
}

impl Iterator for BlockChainIter {
    type Item = Block;
    fn next(&mut self) -> Option<Self::Item> {
        self.next()
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