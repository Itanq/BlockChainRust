
use structopt::StructOpt;
use serde::export::Option::Some;

use crate::block::Block;
use crate::block_chain::BlockChain;
use crate::transaction::*;
use crate::utils::Utils;
use crate::wallet::Wallets;

#[derive(Debug, StructOpt)]
#[structopt(name = "bc_cli", about = "An command line interface for BlockChainRust!!!")]
pub struct Opt {
    #[structopt(short,long, help = "print all block information in the main chain of the blockchain!")]
    Print: bool,

    #[structopt(long, help = "Generates a new key-pair and saves it into the wallet file!")]
    CreateWallet: bool,

    #[structopt(long, help = "Lists all addresses from the wallet file!")]
    ListAddress: bool,

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
    if !Utils::validate_address(address) {
        println!("ERROR: Address is not valid!");
        return;
    }
    if let Some(bc) = BlockChain::create_blockchain(address) {
        println!("Block: {:?}", hex::encode(bc.tip));
        println!("Create BlockChain DONE!!!");
    }
}

fn create_wallet() {
    let mut wallets = Wallets::new();
    let address = wallets.create_wallet();
    wallets.save_to_file();

    println!("Your new wallet address: {}", address);
}

fn get_balance(address: &str) {
    if !Utils::validate_address(address) {
        println!("ERROR: Address is not valid!");
        return;
    }

    if let Some(bc) = BlockChain::new_block_chain(address) {
        let pub_key_hash = Utils::get_pub_key_hash(address);
        let utxo = bc.find_utxo(&pub_key_hash);
        let balance = utxo.iter().fold(0, |acc, x| {
            acc + x.value
        });
        println!("Balance of '{}': {}", address, balance);
    }
}

fn list_addresses() {
    let wallets = Wallets::new();
    let addresses = wallets.get_address();
    addresses.iter().for_each(|x| {
        println!("Address: {}", x);
    });
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
    if opt.Print {
        print_blockchain();
    } else if opt.CreateWallet {
        create_wallet();
    } else if opt.ListAddress {
        list_addresses();
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