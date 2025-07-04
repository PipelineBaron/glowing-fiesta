use glowing_fiesta::ledger::Ledger;
use glowing_fiesta::ledger_system::LedgerSystem;
use std::fs::File;
use std::{env, io};

fn main() {
    env_logger::init();
    let input_path = env::args().nth(1).expect("The input file path is required");
    let input_file = File::open(&input_path).expect("Failed to open input file");

    LedgerSystem::new(Ledger::default(), input_file, io::stdout()).run();
}
