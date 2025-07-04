use crate::stored_transaction::StoredTransaction;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct TransactionStore {
    transactions: HashMap<u32, StoredTransaction>,
}

impl TransactionStore {
    pub fn store<T>(&mut self, transaction: T)
    where
        T: Into<StoredTransaction>,
    {
        let stored: StoredTransaction = transaction.into();
        self.transactions.insert(stored.tx(), stored);
    }

    pub fn get(&self, tx: u32) -> Option<&StoredTransaction> {
        self.transactions.get(&tx)
    }
}
