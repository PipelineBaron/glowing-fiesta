use crate::account_state;
use crate::account_store::AccountStore;
use crate::transaction::{
    ChargebackTransaction, DepositTransaction, DisputeTransaction, ResolveTransaction, Transaction,
    WithdrawalTransaction,
};
use crate::transaction_store::TransactionStore;
use log::error;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("{0}")]
    AccountStateError(#[from] account_state::Error),
    #[error("Account ({client}) Dispute transaction {tx} not found")]
    DisputeTransactionNotFound { client: u16, tx: u32 },
    #[error("Account ({client}) is attempting to dispute transaction {tx} owned by client {owner}")]
    DisputeUnOwnedTransaction { client: u16, tx: u32, owner: u16 },
}

#[derive(Debug, Default)]
pub struct Ledger {
    accounts: AccountStore,
    transactions: TransactionStore,
}

impl Ledger {
    pub fn new(accounts: AccountStore, transactions: TransactionStore) -> Self {
        Ledger {
            accounts,
            transactions,
        }
    }

    pub fn process(&mut self, transaction: &Transaction) -> Result<(), Error> {
        match transaction {
            Transaction::Deposit(deposit) => self.process_deposit(&deposit),
            Transaction::Withdrawal(withdrawal) => self.process_withdrawal(&withdrawal),
            Transaction::Dispute(dispute) => self.process_dispute(&dispute),
            Transaction::Resolve(resolve) => self.process_resolve(&resolve),
            Transaction::Chargeback(chargeback) => self.process_chargeback(&chargeback),
        }
    }

    fn process_deposit(&mut self, deposit: &DepositTransaction) -> Result<(), Error> {
        let account = self.accounts.get_or_create(deposit.client);
        account.deposit(deposit.amount)?;
        self.transactions.store(deposit);
        Ok(())
    }

    fn process_withdrawal(&mut self, withdrawal: &WithdrawalTransaction) -> Result<(), Error> {
        let account = self.accounts.get_or_create(withdrawal.client);
        account.withdraw(withdrawal.amount)?;
        self.transactions.store(withdrawal);
        Ok(())
    }

    fn process_dispute(&mut self, dispute: &DisputeTransaction) -> Result<(), Error> {
        let account = self.accounts.get_or_create(dispute.client);
        if let Some(disputed) = self.transactions.get(dispute.tx) {
            if dispute.client != disputed.client() {
                Err(Error::DisputeUnOwnedTransaction {
                    client: dispute.client,
                    tx: dispute.tx,
                    owner: disputed.client(),
                })
            } else {
                account.dispute(disputed)?;
                Ok(())
            }
        } else {
            Err(Error::DisputeTransactionNotFound {
                client: dispute.client,
                tx: dispute.tx,
            })
        }
    }

    fn process_resolve(&mut self, resolve: &ResolveTransaction) -> Result<(), Error> {
        let account = self.accounts.get_or_create(resolve.client);
        account.resolve(resolve.tx)?;
        Ok(())
    }

    fn process_chargeback(&mut self, chargeback: &ChargebackTransaction) -> Result<(), Error> {
        let account = self.accounts.get_or_create(chargeback.client);
        account.chargeback(chargeback.tx)?;
        Ok(())
    }

    pub fn write_accounts<W: std::io::Write>(&self, writer: W) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        let mut csv_writer = csv::WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);
        for account in self.accounts.iter() {
            csv_writer.serialize(account)?;
        }
        csv_writer.flush()?;
        Ok(())
    }
}
