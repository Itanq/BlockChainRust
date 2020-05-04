
use std::collections::HashMap;
use std::fs::*;
use std::io::{Write, BufWriter, BufReader};

use openssl::bn::BigNumContext;
use openssl::ec::*;
use openssl::pkey::{Private, Public};
use openssl::nid::Nid;
use openssl::sha::Sha256;
use serde::{
    Deserialize, Serialize,
};

use crate::utils::*;

const wallet_file: &str = "wallet.dat";

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    private_key: Vec<u8>,
    public_key: Vec<u8>,
}

impl Wallet {
    pub fn new() -> Self {
        let curve = EcGroup::from_curve_name( Nid::SECP256K1).unwrap();

        let key = EcKey::generate(&*curve).unwrap();
        let private_key = key.private_key_to_der().unwrap();

        let pub_key = EcKey::from_public_key(&*curve, key.public_key()).unwrap();
        let pub_key = pub_key.public_key();

        let mut ctx = BigNumContext::new().unwrap();
        let public_key = pub_key.to_bytes(&*curve, PointConversionForm::COMPRESSED, &mut ctx).unwrap();

        Wallet {
            private_key,
            public_key,
        }
    }

    pub fn get_address(&self) -> String {
        let mut pub_key_hash = Utils::hash_pub_key(&self.public_key);
        // Add version prefix
        pub_key_hash.insert(0, version);

        let checksum = Utils::check_sum(&pub_key_hash);

        let full_payload = checksum.iter().fold(pub_key_hash, |mut acc, x| {
            acc.push(*x);
            acc
        });
        openssl::base64::encode_block(&full_payload.as_slice())
    }

    pub fn hash_pub_key(&self) -> Vec<u8> {
        Utils::hash_pub_key(&self.public_key)
    }

    pub fn public_key(&self) -> Vec<u8> {
        self.public_key.clone()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Wallets {
    wallets: HashMap<String, Wallet>
}

impl Wallets {
    pub fn new() -> Self {
        let wallets = if let Some(wallets) = Wallets::load_from_file() {
            wallets
        } else {
            let mut wallets = Wallets {
                wallets: HashMap::new()
            };
            wallets.create_wallet();
            wallets
        };
        wallets
    }

    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        if let Some(w) = self.wallets.get_mut(&address) {
            *w = wallet
        }
        address
    }

    pub fn get_address(&self) -> Vec<String> {
        let mut address = Vec::new();
        self.wallets.keys().for_each(|x| {
            address.push(x.clone())
        });
        address
    }

    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(&address.to_string())
    }

    fn load_from_file() -> Option<Self> {
        let file = File::open(wallet_file).unwrap();
        let buf_reader = BufReader::new(file);
        if let Ok(res) = serde_json::from_reader(buf_reader) {
            return Some(res);
        }
        None
    }

    fn save_to_file(&self) {
        let file = OpenOptions::new().append(true).open(wallet_file).unwrap();
        let buf_writer = BufWriter::new(file);
        serde_json::to_writer(buf_writer, self).unwrap()
    }

}