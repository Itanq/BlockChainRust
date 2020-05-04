
use structopt::StructOpt;
use BlockChainRust::{ Opt, run };

use openssl::nid::Nid;
use openssl::bn::BigNumContext;
use openssl::ecdsa::EcdsaSig;
use openssl::ec::*;
use openssl::pkey::Private;

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
