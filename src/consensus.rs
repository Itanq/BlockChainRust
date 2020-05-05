use bigint::uint;

use crate::block::Block;

pub trait ProofOfWork {
    fn proof_of_work(&mut self) -> [u8;32];
}

impl ProofOfWork for Block {

    fn proof_of_work(&mut self) -> [u8; 32] {
        let one = uint::U256::one();
        let target = one << ( 256 - self.target_bits as usize );

        while self.nonce < std::u32::MAX {
            let value = serde_json::to_string(&self).unwrap_or("".to_string());
            let hash = openssl::sha::sha256(value.as_bytes());
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
