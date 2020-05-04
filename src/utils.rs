
pub struct Utils;

pub const version: u8 = 0x00;
pub const address_checksum_len: usize = 4;

impl Utils {
    pub fn check_sum(data: &[u8]) -> Vec<u8> {
        let hash1 = openssl::sha::sha256(data);
        let hash2 = openssl::sha::sha256(&hash1);
        hash2[..address_checksum_len].to_vec()
    }

    pub fn get_pub_key_hash(address: &str) -> Vec<u8> {
        let pub_key_hash = openssl::base64::decode_block(address).unwrap();
        pub_key_hash[1..pub_key_hash.len() - address_checksum_len].to_vec()
    }

    pub fn hash_pub_key(data: &[u8]) -> Vec<u8> {
        let hash_sha256 = openssl::sha::sha256(data);
        let hash_ripemd160 = openssl::hash::hash(
            openssl::hash::MessageDigest::ripemd160(), &hash_sha256).unwrap();
        hash_ripemd160.to_vec()
    }

    pub fn validate_address(address: &str) -> bool {
        let address_payload = openssl::base64::decode_block(address).unwrap();
        if address_payload.len() < address_checksum_len {
            return false;
        }

        let checksum = &address_payload[address_payload.len() - address_checksum_len..];

        let mut data = vec![ address_payload[0] ];
        let pub_key_hash = address_payload[1..address_payload.len() - address_checksum_len].to_vec();
        data.extend_from_slice(&pub_key_hash);

        let target_checksum = Utils::check_sum(&data);

        checksum == target_checksum.as_slice()
    }
}

