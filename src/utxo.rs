
use crate::block_chain::BlockChain;
use std::collections::HashMap;
use serde::export::Option::Some;
use crate::transaction::TXOutputVec;

const chainstate_db: &str = "chain_state.db";

pub struct UTXOSet {
    block_chain: BlockChain,
    db: sled::Db,
}

impl UTXOSet {

    pub fn new() -> Option<Self> {
        if std::path::Path::new(chainstate_db).exists() {
            println!("ChainState already exists.");
            return None;
        }

        let db = sled::open(chainstate_db).unwrap();
        let block_chain = BlockChain::new_block_chain()?;

        Some(UTXOSet {
            block_chain,
            db
        })
    }

    pub fn reindex(&self) {
        let utxo = self.block_chain.find_all_utxo();

        for (txid, outs) in utxo {
            self.db.insert(txid, &outs.to_string()[..]);
        }
    }

    pub fn find_spendable_outputs(&self, pub_key_hash: &[u8], amount: i32) -> (i32, HashMap::<String,Vec<i32>>) {

        let mut acc = 0;
        let mut unspent_outputs: HashMap::<String,Vec<i32>> = HashMap::new();

        let mut iter = self.db.iter();
        while let Ok((k, v)) = iter.next().unwrap() {
            let txid = hex::encode(k);
            let out: TXOutputVec = serde_json::from_slice(&v).unwrap();
            let mut unspent_vec = Vec::<i32>::new();
            for (out_idx, out) in out.outputs.iter().enumerate() {
                if out.is_locked_with_key(pub_key_hash) && acc < amount {
                    acc += out.value;
                    unspent_vec.push(out_idx as i32);
                    if acc >= amount {
                        break;
                    }
                }
            }
            unspent_outputs.insert(txid, unspent_vec);
            if acc >= amount {
                break;
            }
        }
        (acc, unspent_outputs)
    }
}

