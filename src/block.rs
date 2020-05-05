use openssl::sha;
use serde::{ Serialize, Deserialize };

use openssl::sha::sha256;
use openssl::version::version;
use std::collections::HashMap;
use std::path::Display;
use sled::open;

use crate::transaction::*;
use crate::wallet::Wallets;
use crate::utils::*;
use crate::consensus::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub(crate) time_stamp: u64,
    pub(crate) transaction: Vec<Transaction>,
    pub(crate) pre_block_hash: [u8; 32],
    pub(crate) cur_block_hash: [u8; 32],
    pub(crate) target_bits: u8,
    pub(crate) nonce: u32,
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

