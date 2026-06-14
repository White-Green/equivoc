mod frontend_ir;
mod frontend_to_mir;

use clap::Parser;
use equivoc_engine::basic_block_interpreter::execute;
use equivoc_engine::mir_optimizer::optimize_source_mir;
use equivoc_engine::mir2lir_translator::convert_equivoc_mir_to_equivoc_lir;
use frontend_ir::EquivocFrontendIr;
use frontend_to_mir::convert_frontend_ir_to_mir;
use std::io;
use std::io::Read;

#[derive(Debug, clap::Parser)]
struct CliOption {
    #[clap(long)]
    read_frontend_ir_from_stdin: bool,
}

fn main() {
    let option = CliOption::parse();
    assert!(option.read_frontend_ir_from_stdin);
    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin).unwrap();
    let frontend_ir = serde_json::from_str::<EquivocFrontendIr>(&stdin).unwrap();
    dbg!(&frontend_ir);
    let mut mir = convert_frontend_ir_to_mir(frontend_ir);
    optimize_source_mir(&mut mir);
    dbg!(&mir);
    let lir = convert_equivoc_mir_to_equivoc_lir(&mir);
    dbg!(&lir);
    execute(&lir);
    println!("Executed!");
}
