use clap::Parser;
use equivoc_engine::mir::EquivocMir;
use std::io;
use std::io::Read;
use equivoc_engine::basic_block_interpreter::execute;
use equivoc_engine::mir2lir_translator::convert_equivoc_mir_to_equivoc_lir;

#[derive(Debug, clap::Parser)]
struct CliOption {
    #[clap(long)]
    read_mir_from_stdin: bool,
}

fn main() {
    let option = CliOption::parse();
    assert!(option.read_mir_from_stdin);
    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin).unwrap();
    let mir = serde_json::from_str::<EquivocMir>(&stdin).unwrap();
    dbg!(&mir);
    let lir = convert_equivoc_mir_to_equivoc_lir(&mir);
    dbg!(&lir);
    execute(&lir);
    println!("Executed!");
}
