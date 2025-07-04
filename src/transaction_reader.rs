use crate::transaction::{CsvTransaction, Transaction};
use log::error;
use std::io;

pub struct TransactionReader<R> {
    csv_reader: csv::Reader<R>,
}

impl<R> TransactionReader<R>
where
    R: io::Read,
{
    pub fn new(reader: R) -> Self {
        let csv_reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(reader);
        TransactionReader { csv_reader }
    }

    pub fn iter(&mut self) -> impl Iterator<Item = Transaction> {
        self.csv_reader
            .deserialize::<CsvTransaction>()
            .filter_map(|row| row.inspect_err(|e| error!("{e}")).ok())
            .filter_map(|row| {
                Transaction::try_from(row)
                    .inspect_err(|e| error!("{e}"))
                    .ok()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::*;
    use rust_decimal::Decimal;
    use std::io::Cursor;

    #[test]
    fn test_transaction_reader() {
        // given ...
        let data = "type, client, tx, amount\n\
        deposit, 1, 1, 1.0\n\
        withdrawal, 2, 2, 2.0\n\
        dispute, 3, 3,\n\
        resolve, 4, 4,\n\
        chargeback, 5, 5,\n";
        let cursor = Cursor::new(data);

        // when ...
        let mut reader = TransactionReader::new(cursor);
        let transactions: Vec<Transaction> = reader.iter().collect();

        // then ...
        assert_eq!(
            transactions,
            vec![
                Transaction::Deposit(DepositTransaction {
                    client: 1,
                    tx: 1,
                    amount: Decimal::new(10, 1),
                }),
                Transaction::Withdrawal(WithdrawalTransaction {
                    client: 2,
                    tx: 2,
                    amount: Decimal::new(20, 1),
                }),
                Transaction::Dispute(DisputeTransaction { client: 3, tx: 3 }),
                Transaction::Resolve(ResolveTransaction { client: 4, tx: 4 }),
                Transaction::Chargeback(ChargebackTransaction { client: 5, tx: 5 }),
            ]
        )
    }
}
