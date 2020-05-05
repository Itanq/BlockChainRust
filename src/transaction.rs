
use serde::{ Serialize, Deserialize };

use crate::utils::*;
use crate::block::Block;
use crate::block_chain::BlockChain;
use crate::wallet::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    pub(crate) tx_id: [u8;32],
    pub(crate) vout: i32,
    pub(crate) signature: Vec<u8>,
    pub(crate) pub_key: Vec<u8>,
}

impl TXInput {

    pub fn uses_key(&self, pub_key_hash: &[u8]) -> bool {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub(crate) id: Vec<u8>,
    pub(crate) vin: Vec<TXInput>,
    pub(crate) vout: Vec<TXOutput>,
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
        self.id = openssl::sha::sha256(&enc.as_bytes().to_vec()).to_vec();
    }

}
