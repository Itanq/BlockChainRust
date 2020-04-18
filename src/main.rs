
use BlockChainRust::BlockChain;

fn main() {
    let mut block = BlockChain::new_block_chain();

    //block.add_block("Send 1 BTC to Ivan".to_string());
    //block.add_block("Send 2 BTC to Ivan".to_string());

    block.print();
}
