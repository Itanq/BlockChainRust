use openssl::sha;
use serde::{ Serialize, Deserialize };
use bigint::uint;
use openssl::sha::sha256;
use openssl::version::version;
use std::collections::HashMap;
use std::path::Display;
use crate::utils::*;
use sled::open;
use crate::wallet::Wallets;

const blockchain_db: &str = "block_chain.db";
const genesis_coinbase_data: &str = "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TXInput {
    tx_id: [u8;32],
    vout: i32,
    signature: Vec<u8>,
    pub_key: Vec<u8>,
}

impl TXInput {

    pub fn uses_key(&self, pub_key_hash: &[u8]) -> bool {
        let lock_hash = Utils::hash_pub_key(&self.pub_key);
        lock_hash == pub_key_hash
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: i32,
    pub_key_hash: Vec<u8>
}

impl TXOutput {

    pub fn new(value: i32, address: &str) -> Self {
        let mut out = TXOutput{
            value,
            pub_key_hash: vec![],
        };
        out.lock(address);

        out
    }

    pub fn lock(&mut self, address: &str) {
        let address_payload = openssl::base64::decode_block(address).unwrap();
        let pub_key_hash = &address_payload[1..address_payload.len() - address_checksum_len];
        self.pub_key_hash = pub_key_hash.to_vec();
    }

    pub fn is_locked_with_key(&self, key: &[u8]) -> bool {
        self.pub_key_hash == key
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    id: Vec<u8>,
    vin: Vec<TXInput>,
    vout: Vec<TXOutput>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    time_stamp: u64,
    transaction: Vec<Transaction>,
    pre_block_hash: [u8; 32],
    cur_block_hash: [u8; 32],
    target_bits: u8,
    nonce: u32,
}

impl Block {

    pub fn genesis_block(coinbase: Transaction) -> Self {
        Block::new_block(vec![coinbase], [0u8;32])
    }

    pub fn new_block(transaction: Vec<Transaction>, pre_block_hash: [u8; 32]) -> Self {
        let mut block = Block {
            time_stamp: std::time::SystemTime::now().elapsed().unwrap().as_secs(),
            transaction,
            pre_block_hash,
            cur_block_hash: [0;32],
            target_bits: 16,
            nonce: 0,
        };
        block.cur_block_hash = block.proof_of_work();
        block
    }

    pub fn cur_block_hash(&self) -> String {
        hex::encode(self.cur_block_hash.to_vec())
    }

    pub fn pre_block_hash(&self) -> String {
        hex::encode(self.pre_block_hash.to_vec())
    }

    pub fn transaction(&self) -> &Vec<Transaction> {
        &self.transaction
    }

    pub fn print(&self) {
        println!("Prev Hash: {:?}", self.pre_block_hash());
        println!("Curr Hash: {:?}", self.cur_block_hash());
        println!("Data: {:?}\n", self.transaction);
    }
}

impl ToString for Block {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

pub struct BlockChain {
    pub tip: [u8; 32],
    db: sled::Db,
}

impl BlockChain {
    pub fn create_blockchain(address: &str) -> Option<Self> {
        if std::path::Path::new(blockchain_db).exists() {
            println!("BlockChain already exists.");
            return None;
        }

        let genesis_tx = Transaction::new_coinbase_tx(address, genesis_coinbase_data.to_string());
        let genesis_block = Block::genesis_block(genesis_tx);

        let db = sled::open(blockchain_db).unwrap();
        db.insert(genesis_block.cur_block_hash, &genesis_block.to_string()[..]);
        db.insert("last", &genesis_block.cur_block_hash);

        let tip = genesis_block.cur_block_hash;

        Some(BlockChain {
            tip,
            db
        })
    }

    pub fn new_block_chain(address: &str) -> Option<Self> {
        if !std::path::Path::new(blockchain_db).exists() {
            println!("No existing blockchain found, please create one first!!!");
            return None;
        }

        let db = sled::open(blockchain_db).unwrap();
        let hash = db.get("last").unwrap().unwrap().to_vec();
        let last_hash = hash.as_slice();

        let mut tip = [0u8; 32];
        tip.copy_from_slice(last_hash);

        Some(BlockChain {
            tip,
            db,
        })
    }

    pub fn mine_block(&mut self, transactions: Vec<Transaction>) {
        let mut pre_block_hash: [u8; 32] = [0u8;32];
        pre_block_hash.copy_from_slice(&self.db.get("last").unwrap().unwrap().to_vec()[..]);
        let new_block = Block::new_block(transactions, pre_block_hash);
        self.db.insert(new_block.cur_block_hash, &new_block.to_string()[..]);
        self.db.insert("last", &new_block.cur_block_hash);
        println!("{}", hex::encode(new_block.cur_block_hash));
    }

    pub fn find_utxo(&self, pub_key_hash: &[u8]) -> Vec<TXOutput> {
        let mut utxo = Vec::<TXOutput>::new();
        let unspent_txs = self.find_unspent_transaction(pub_key_hash);

        for tx in unspent_txs {
            for out in tx.vout {
                if out.is_locked_with_key(pub_key_hash) {
                    utxo.push(out);
                }
            }
        }
        utxo
    }

    pub fn find_spendable_outputs(&self, pub_key_hash: &[u8], amount: i32) -> (i32, HashMap<String,Vec<i32>>) {
        let mut unspent_outputs = HashMap::<String,Vec<i32>>::new();
        let mut unspent_txs = self.find_unspent_transaction(pub_key_hash);
        let mut acc = 0;

        for tx in unspent_txs {
            let tx_id = hex::encode(tx.id);
            let mut unspent_array = Vec::<i32>::new();
            for (idx, out) in tx.vout.iter().enumerate() {
                if out.is_locked_with_key(pub_key_hash) && acc < amount {
                    acc += out.value;
                    unspent_array.push(idx as i32);

                    if acc >= amount {
                        break;
                    }
                }
            }
            unspent_outputs.insert(tx_id, unspent_array);
            if acc >= amount {
                break;
            }
        }

        (acc, unspent_outputs)
    }

    pub fn find_unspent_transaction(&self, pub_key_hash: &[u8]) -> Vec<Transaction> {
        let mut unspent_txs = Vec::new();
        let mut spent_txs: HashMap<String, Vec<i32>> = HashMap::new();

        let mut iter = self.iter();
        while let Some(bc) = iter.next() {
            for tx in bc.transaction {
                let tx_id = hex::encode(tx.id.clone());
                for (out_idx, out) in tx.vout.iter().enumerate() {
                    let mut spent = false;
                    if let Some(spent_outs) = spent_txs.get(&tx_id) {
                        for spent_out in spent_outs {
                            if *spent_out == out_idx as i32 {
                                spent = true;
                                break;
                            }
                        }
                    }

                    if !spent && out.is_locked_with_key(pub_key_hash) {
                        unspent_txs.push(tx.clone());
                        break;
                    }
                }
                if !tx.is_coinbase() {
                    let mut vout_arr = Vec::<i32>::new();
                    for tx_in in tx.vin {
                        if tx_in.uses_key(pub_key_hash) {
                            let id = hex::encode(tx_in.tx_id);
                            if let Some(arr) = spent_txs.get_mut(&id) {
                                arr.push(tx_in.vout);
                            } else {
                                spent_txs.insert(id, vec![tx_in.vout]);
                            }
                        }
                    }
                }
            }
        }

        unspent_txs
    }

    pub fn iter(&self) -> BlockChainIter {
        let mut cur_hash = [0u8; 32];
        cur_hash.copy_from_slice(&self.db.get("last").unwrap().unwrap().to_vec());

        BlockChainIter {
            cur_hash,
            db: self.db.clone()
        }
    }

    pub fn get_block(&self, hash: &[u8]) -> Block {
        serde_json::from_slice(&self.db.get(hash).unwrap().unwrap()).unwrap()
    }

    pub fn print(&self) {
        let mut iter = self.iter();
        while let Some(bc) = iter.next() {
            println!("Prev Hash: {:?}", bc.pre_block_hash());
            println!("Curr Hash: {:?}", bc.cur_block_hash());
            println!("Data: {:?}\n", bc.transaction);
        }
    }
}


impl Transaction {

    pub fn new_coinbase_tx(to: &str, data: String) -> Self {
        let data = if data.is_empty() {
            format!("Reward to '{}'.", to)
        } else { data };

        let tx_in = TXInput{
            tx_id: [0u8;32],
            vout: -1,
            signature: vec![],
            pub_key: data.as_bytes().to_vec()
        };

        let tx_out = TXOutput::new(10, to);

        let mut tx = Transaction {
            id: vec![0],
            vin: vec![tx_in],
            vout: vec![tx_out]
        };
        tx.set_id();

        tx
    }

    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].tx_id == [0u8;32] && self.vin[0].vout == -1
    }

    pub fn new_utxo_transaction(from: &str, to: &str, amount: i32, bc: &BlockChain) -> Option<Self>
    {
        let mut inputs = Vec::<TXInput>::new();
        let mut outputs = Vec::<TXOutput>::new();

        let wallets = Wallets::new();
        let wallet = wallets.get_wallet(from).unwrap();
        let pub_key_hash = wallet.hash_pub_key();

        let (acc, valid_outputs) = bc.find_spendable_outputs(
            &pub_key_hash, amount);
        if acc < amount {
            return None;
        }

        for (key, value) in valid_outputs {
            let mut tx_id = [0u8; 32];
            tx_id.copy_from_slice(hex::decode(key).unwrap().as_slice());
            for out in value {
                let input = TXInput {
                    tx_id,
                    vout: out,
                    signature: vec![],
                    pub_key: wallet.public_key()
                };
                inputs.push(input);
            }
        }

        outputs.push(TXOutput::new(amount, to));

        if acc > amount {
            outputs.push(TXOutput::new(acc - amount, from));
        }

         let mut tx = Transaction{
            id: vec![],
            vin: inputs,
            vout: outputs,
        };
        tx.set_id();

        Some(tx)
    }

    pub fn set_id(&mut self) {
        let enc = serde_json::to_string(self).unwrap();
        self.id = sha256(&enc.as_bytes().to_vec()).to_vec();
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