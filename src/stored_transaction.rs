use crate::transaction::{DepositTransaction, WithdrawalTransaction};
use rust_decimal::Decimal;

#[derive(Debug)]
pub enum StoredTransaction {
    Deposit(StoredDepositTransaction),
    Withdrawal(StoredWithdrawalTransaction),
}

impl StoredTransaction {
    pub fn tx(&self) -> u32 {
        match self {
            StoredTransaction::Deposit(deposit) => deposit.tx,
            StoredTransaction::Withdrawal(withdrawal) => withdrawal.tx,
        }
    }

    pub fn client(&self) -> u16 {
        match self {
            StoredTransaction::Deposit(deposit) => deposit.client,
            StoredTransaction::Withdrawal(withdrawal) => withdrawal.client,
        }
    }
}

impl From<&DepositTransaction> for StoredTransaction {
    fn from(deposit: &DepositTransaction) -> Self {
        StoredTransaction::Deposit(deposit.into())
    }
}

impl From<&WithdrawalTransaction> for StoredTransaction {
    fn from(withdrawal: &WithdrawalTransaction) -> Self {
        StoredTransaction::Withdrawal(withdrawal.into())
    }
}

#[derive(Debug)]
pub struct StoredDepositTransaction {
    pub tx: u32,
    pub client: u16,
    pub amount: Decimal,
}

impl From<&DepositTransaction> for StoredDepositTransaction {
    fn from(deposit: &DepositTransaction) -> Self {
        StoredDepositTransaction {
            tx: deposit.tx,
            client: deposit.client,
            amount: deposit.amount,
        }
    }
}

#[derive(Debug)]
pub struct StoredWithdrawalTransaction {
    pub tx: u32,
    pub client: u16,
    pub amount: Decimal,
}

impl From<&WithdrawalTransaction> for StoredWithdrawalTransaction {
    fn from(withdrawal: &WithdrawalTransaction) -> Self {
        StoredWithdrawalTransaction {
            tx: withdrawal.tx,
            client: withdrawal.client,
            amount: withdrawal.amount,
        }
    }
}
