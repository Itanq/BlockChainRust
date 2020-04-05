
use BlockChainRust::BlockChain;

fn main() {
    let mut block = BlockChain::genesis_block();

    block.add_block("Send 1 BTC to Ivan".to_string());
    block.add_block("Send 2 BTC to Ivan".to_string());

    let blocks = block.blocks();
    for bc in blocks {
        println!("Prev. hash: {:?}", bc.pre_block_hash());
        println!("Data: {:?}", bc.data());
        println!("Hash: {:?}", bc.cur_block_hash());
        println!("\n");
    }
}
