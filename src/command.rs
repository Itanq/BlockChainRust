use structopt::StructOpt;
use crate::BlockChain;
use serde::export::Option::Some;
use crate::block::Transaction;

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
    #[structopt( help = "Create a blockchain and send genesis block reward to ADDRESS")]
    CreateBlockChain {
        #[structopt(short,long, help = "create-blockchain --address ADDRESS")]
        address: String,
    },

    #[structopt( help = "Get balance of ADDRESS!")]
    GetBalance {
        #[structopt(short,long, help = "get-balance --address ADDRESS")]
        address: String
    },

    #[structopt( help = "Send AMOUNT of coins from FROM address to TO address")]
    Send {
        #[structopt(long, help = "send --from FROM --to TO --amount AMOUNT")]
        from: String,

        #[structopt(long, help = "The dest address of the send transaction")]
        to: String,

        #[structopt(long, help = "The amount of the send transaction")]
        amount: i32
    }
}

fn create_blockchain(address: &str) {
    if let Some(bc) = BlockChain::create_blockchain(address) {
        println!("Block: {:?}", hex::encode(bc.tip));
        println!("Create BlockChain DONE!!!");
    }
}


fn get_balance(address: &str) {
    if let Some(bc) = BlockChain::new_block_chain(address) {
        let utxo = bc.find_utxo(address);
        let balance = utxo.iter().fold(0, |acc, x| {
            acc + x.value
        });
        println!("Balance of {}: {}", address, balance);
    }
}

fn print_blockchain() {
    if let Some(bc) = BlockChain::new_block_chain("") {
        bc.print();
    }
}

fn send(from: &str, to: &str, amount: i32) {
    if let Some(mut bc) = BlockChain::new_block_chain(&from) {
        if let Some(tx) = Transaction::new_utxo_transaction(from, to, amount, &bc) {
            bc.mine_block(vec![tx])
        } else {
            println!("The balance of {} is not enough!", from);
        }
    }
}

pub fn run(opt: Opt) {
    if opt.print {
        print_blockchain();
    } else if let Some(cmd) = opt.cmd {
        match cmd {
            SubCommand::CreateBlockChain { address } => {
                create_blockchain(&address);
            },
            SubCommand::GetBalance{ address } => {
                get_balance(&address);
            },
            SubCommand::Send { from, to, amount} => {
                send(&from, &to, amount);
            }
        }
    }
}