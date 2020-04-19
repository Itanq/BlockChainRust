
use structopt::StructOpt;
use BlockChainRust::{ Opt, run };

fn main() {
    let opt = Opt::from_args();
    run(opt);
}
