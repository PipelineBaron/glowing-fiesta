use crate::transaction_type::TransactionType;
use rust_decimal::{Decimal, RoundingStrategy};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CsvTransaction {
    pub r#type: TransactionType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<Decimal>,
}

impl TryFrom<CsvTransaction> for Transaction {
    type Error = String;

    fn try_from(csv: CsvTransaction) -> Result<Self, Self::Error> {
        match csv.r#type {
            TransactionType::Deposit => {
                if let Some(amount) = csv.amount {
                    Ok(Transaction::Deposit(DepositTransaction {
                        client: csv.client,
                        tx: csv.tx,
                        amount: amount.round_dp_with_strategy(4, RoundingStrategy::ToZero),
                    }))
                } else {
                    Err("Deposit transaction must have an amount".to_string())
                }
            }
            TransactionType::Withdrawal => {
                if let Some(amount) = csv.amount {
                    Ok(Transaction::Withdrawal(WithdrawalTransaction {
                        client: csv.client,
                        tx: csv.tx,
                        amount: amount.round_dp_with_strategy(4, RoundingStrategy::ToZero),
                    }))
                } else {
                    Err("Withdrawal transaction must have an amount".to_string())
                }
            }
            TransactionType::Dispute => Ok(Transaction::Dispute(DisputeTransaction {
                client: csv.client,
                tx: csv.tx,
            })),
            TransactionType::Resolve => Ok(Transaction::Resolve(ResolveTransaction {
                client: csv.client,
                tx: csv.tx,
            })),
            TransactionType::Chargeback => Ok(Transaction::Chargeback(ChargebackTransaction {
                client: csv.client,
                tx: csv.tx,
            })),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Transaction {
    Deposit(DepositTransaction),
    Withdrawal(WithdrawalTransaction),
    Dispute(DisputeTransaction),
    Resolve(ResolveTransaction),
    Chargeback(ChargebackTransaction),
}

#[derive(Debug, PartialEq, Clone)]
pub struct DepositTransaction {
    pub client: u16,
    pub tx: u32,
    pub amount: Decimal,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WithdrawalTransaction {
    pub client: u16,
    pub tx: u32,
    pub amount: Decimal,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DisputeTransaction {
    pub client: u16,
    pub tx: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ResolveTransaction {
    pub client: u16,
    pub tx: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChargebackTransaction {
    pub client: u16,
    pub tx: u32,
}
