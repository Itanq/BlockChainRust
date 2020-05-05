
use structopt::StructOpt;
use BlockChainRust::{ Opt, run };

use openssl::nid::Nid;
use openssl::bn::BigNumContext;
use openssl::ecdsa::EcdsaSig;
use openssl::ec::*;
use openssl::pkey::Private;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, BufReader};
use std::path::Path;

#[test]
fn test_serde() {
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Wallet {
        private_key: Vec<u8>,
        public_key: Vec<u8>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Wallets {
        wallets: HashMap<String, Wallet>
    }

    impl Wallets {
        fn serialize(&self) {
            let res = serde_json::to_value(self).unwrap();
            println!("res={}", res);

            let path = Path::new("test_serde.json");
            if path.exists() {
                std::fs::remove_file(path);
            }

            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .open("test_serde.json")
                .unwrap();

            let buf_writer = BufWriter::new(file);
            serde_json::to_writer(buf_writer, &res);
        }

        fn load_from_file(&self) -> Self {
            let file = OpenOptions::new().read(true).open("test_serde.json").unwrap();
            let buf_reader = BufReader::new(file);
            serde_json::from_reader(buf_reader).unwrap()
        }
    }

    let mut map = HashMap::new();
    map.insert("one".to_string(), Wallet{private_key:vec![1,2,3], public_key: vec![4,5,6]});
    map.insert("two".to_string(), Wallet{private_key:vec![11,22,33], public_key:vec![44,55,66]});
    map.insert("six".to_string(), Wallet{private_key:vec![111,222,133], public_key:vec![144,155,155]});

    let test = Wallets{
        wallets:map
    };

    test.serialize();
    let res = test.load_from_file();
    println!("load_from_file: {:?}", res);
}

#[test]
fn test_ecdsa() {
    let curve = EcGroup::from_curve_name(Nid::SECP256K1).unwrap();
    let key = EcKey::generate(&*curve).unwrap();

    let pub_key = key.public_key();
    let pri_key = hex::encode(key.private_key().to_vec());

    let mut ctx = BigNumContext::new().unwrap();

    let pub_bytes = pub_key.to_bytes(&*curve, PointConversionForm::COMPRESSED, &mut ctx).unwrap();

    let pubkey = hex::encode(pub_bytes.clone());

    println!("private_key: {:?}", pri_key);
    println!("public_key: {:?}", pubkey);

    let data = "hello rust!";

    let res = EcdsaSig::sign(
        data.as_bytes(), &*key).unwrap();

    println!("Original Data: {}", data);
    println!("signatureData: {}", hex::encode(res.to_der().unwrap()));

    let pkey = EcKey::from_public_key(
        &*curve,
        &EcPoint::from_bytes(&*curve, &pub_bytes, &mut *ctx).unwrap()).unwrap();
     println!("verify: {}",
              res.verify(data.as_bytes(), &*pkey).unwrap());

    // get bytes from somewhere, i.e. this will not produce a valid key
    let public_key: Vec<u8> = vec![];

}

fn main() {
    let opt = Opt::from_args();
    run(opt);
}
