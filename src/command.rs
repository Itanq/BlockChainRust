use structopt::StructOpt;
use crate::BlockChain;
use serde::export::Option::Some;

#[derive(Debug, StructOpt)]
#[structopt(name = "bc_cli", about = "An command line interface for BlockChainRust!!!")]
pub struct Opt {
    #[structopt(short,long, help = "print all block information in the main chain of the blockchain!")]
    print: bool,
    #[structopt(subcommand)]
    cmd: Option<SubCommand>,
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    #[structopt( help = "Add a new block to chain!")]
    AddBlock {
        #[structopt(short,long, help = "the data of block need to be add!")]
        data: String,
    },

    #[structopt( help = "Get a specific block from chain!")]
    Get {
        #[structopt(short,long, help = "the hash value of the node that need to obtain!")]
        node: String
    }
}

pub fn run(opt: Opt) {
    let mut block = BlockChain::new_block_chain();
    if opt.print {
        block.print();
    } else if let Some(cmd) = opt.cmd {
        match cmd {
            SubCommand::AddBlock { data } => {
                println!("Mining the block: {}", data);
                block.add_block(data);
                println!("\nSuccess!");
            },
            SubCommand::Get{ node } => {
                println!("Get the block of {}", node);
                let block = block.get_block(&hex::decode(node.as_bytes()).unwrap());
                block.print();
                println!("\nSuccess!");
            }
        }
    }
}