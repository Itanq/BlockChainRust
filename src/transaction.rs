
use serde::{ Serialize, Deserialize };
use openssl::bn::BigNumContext;
use openssl::ecdsa::EcdsaSig;
use openssl::ec::*;
use openssl::pkey::Private;

use crate::utils::*;
use crate::block::Block;
use crate::block_chain::BlockChain;
use crate::wallet::*;
use std::collections::HashMap;
use openssl::nid::Nid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    pub(crate) tx_id: [u8;32],
    pub(crate) vout: i32,
    pub(crate) signature: Vec<u8>,
    pub(crate) pub_key: Vec<u8>,
}

impl TXInput {

    pub fn used_by_key(&self, pub_key_hash: &[u8]) -> bool {
        let lock_hash = Utils::hash_pub_key(&self.pub_key);
        lock_hash == pub_key_hash
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub(crate) value: i32,
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

impl ToString for TXOutput {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutputVec {
    pub(crate) outputs: Vec<TXOutput>
}

impl ToString for TXOutputVec {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub(crate) id: Vec<u8>,
    pub(crate) vin: Vec<TXInput>,
    pub(crate) vout: Vec<TXOutput>,
}

impl Transaction {

    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].tx_id == [0u8;32] && self.vin[0].vout == -1
    }

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

        bc.sign_transaction(&wallet.private_key, &mut tx);

        Some(tx)
    }

    pub fn hash(&self) -> Vec<u8> {
        let enc = serde_json::to_string(self).unwrap();
        openssl::sha::sha256(&enc.as_bytes().to_vec()).to_vec()
    }

    pub fn set_id(&mut self) {
        let enc = serde_json::to_string(self).unwrap();
        self.id = openssl::sha::sha256(&enc.as_bytes().to_vec()).to_vec();
    }

    pub fn set_hash(data: Transaction) -> Vec<u8>{
        let enc = serde_json::to_string(&data).unwrap();
        openssl::sha::sha256(&enc.as_bytes().to_vec()).to_vec()
    }

    pub fn sign(&mut self, priv_key: &[u8], prev_txs: HashMap<String, Transaction>) {
        if self.is_coinbase() {
            return;
        }

        let tx_sign = self.trimmed_copy();

        println!("tx_sign: before ::: {:?}", self);

        for (idx, vin) in tx_sign.vin.iter().enumerate() {
            let prev_tx = prev_txs.get(&hex::encode(vin.tx_id)).unwrap();
            let mut tx = tx_sign.clone();
            tx.vin[idx].signature = vec![];
            tx.vin[idx].pub_key = prev_tx.vout.get(vin.vout as usize).unwrap().pub_key_hash.clone();
            tx.set_id();

            let key = EcKey::private_key_from_der(priv_key).unwrap();
            let sig = EcdsaSig::sign(&tx.id, &*key).unwrap();

            self.vin[idx].signature = sig.to_der().unwrap();
        }
        println!("tx_sign: before ::: {:?}", self);
    }

    pub fn trimmed_copy(&self) -> Self {
        let mut inputs = Vec::<TXInput>::new();

        self.vin.iter().for_each(|x| {
            inputs.push(TXInput{ tx_id: x.tx_id, vout: x.vout, signature: vec![], pub_key: vec![]})
        });


        Transaction{
            id: self.id.clone(),
            vin: inputs,
            vout: self.vout.clone(),
        }
    }

    pub fn verify(&self, pub_key: &[u8], prev_txs: HashMap<String, Transaction>) -> bool {
        let tx_copy = self.trimmed_copy();
        let curve = EcGroup::from_curve_name(Nid::SECP256K1).unwrap();
        let mut ctx = BigNumContext::new().unwrap();
        let pkey = EcKey::from_public_key(
            &*curve,
            &EcPoint::from_bytes(&*curve, pub_key, &mut *ctx).unwrap()
        ).unwrap();

        for (idx, vin) in tx_copy.vin.iter().enumerate() {
            let prev_tx = prev_txs.get(&hex::encode(vin.tx_id)).unwrap();

            let mut tx = tx_copy.clone();
            tx.vin[idx].signature = vec![];
            tx.vin[idx].pub_key = prev_tx.vout.get(vin.vout as usize).unwrap().clone().pub_key_hash;
            tx.set_id();

            let ecdsa = EcdsaSig::from_der(&vin.signature).unwrap();
            if !ecdsa.verify(&tx.id, &*pkey).unwrap() {
                return false
            }
        }
        true
    }

}
