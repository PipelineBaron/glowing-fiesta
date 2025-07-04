use crate::ledger::Ledger;
use crate::transaction_reader::TransactionReader;
use log::error;
use std::io;

pub struct LedgerSystem<R, W> {
    ledger: Ledger,
    reader: R,
    writer: W,
}

impl<R, W> LedgerSystem<R, W>
where
    R: io::Read,
    W: io::Write,
{
    pub fn new(ledger: Ledger, reader: R, writer: W) -> Self {
        LedgerSystem {
            ledger,
            reader,
            writer,
        }
    }

    pub fn run(mut self) {
        let mut transactions = TransactionReader::new(self.reader);

        for transaction in transactions.iter() {
            let _ = self
                .ledger
                .process(&transaction)
                .inspect_err(|e| error!("{e}"));
        }

        let _ = self
            .ledger
            .write_accounts(self.writer)
            .inspect_err(|e| error!("{e}"));
    }
}
