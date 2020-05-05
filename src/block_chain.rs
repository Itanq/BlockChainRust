
use std::collections::HashMap;

use crate::block::Block;
use crate::transaction::*;

const blockchain_db: &str = "block_chain.db";
const genesis_coinbase_data: &str = "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

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
        //TODO: Verify the transaction before mine block!

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
                        if tx_in.used_by_key(pub_key_hash) {
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

    pub fn find_transaction(&self, id: &[u8]) -> Option<Transaction> {
        let mut iter = self.iter();
        while let Some(bc) = iter.next() {
            for tx in bc.transaction {
                if tx.id == id {
                    return Some(tx);
                }
            }
        }
        None
    }

    pub fn sign_transaction(&self, priv_key: &[u8], tx: &mut Transaction) {
        let mut prev_txs = HashMap::<String, Transaction>::new();
        for vin in &tx.vin {
            if let Some(tx) = self.find_transaction(&vin.tx_id) {
                prev_txs.insert(hex::encode(tx.id.clone()), tx.clone());
            }
        }
        tx.sign(priv_key, prev_txs);
    }

    pub fn verify_transaction(&self, pub_key: &[u8], tx: &Transaction) -> bool {
        let mut prev_txs = HashMap::<String,Transaction>::new();
        for vin in &tx.vin {
            if let Some(tx) = self.find_transaction(&vin.tx_id) {
                prev_txs.insert(hex::encode(tx.id.clone()), tx.clone());
            }
        }
        return tx.verify(pub_key, prev_txs);
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