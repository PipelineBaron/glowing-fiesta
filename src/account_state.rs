use crate::stored_transaction::StoredTransaction;
use rust_decimal::Decimal;
use serde::Serialize;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Account ({client}) has insufficient funds")]
    InsufficientFunds { client: u16 },
    #[error("Account ({client}) is locked")]
    AccountLocked { client: u16 },
    #[error("Account ({client}) already has a dispute for transaction {tx}")]
    TransactionAlreadyDisputed { client: u16, tx: u32 },
    #[error(
        "Account ({client}) has a dispute for withdrawal transaction {tx}, which is not allowed"
    )]
    DisputeOnWithdrawal { client: u16, tx: u32 },
    #[error("Account ({client}) does not have a dispute for transaction {tx}")]
    DisputeNotFound { client: u16, tx: u32 },
}

#[derive(Debug, PartialEq, Serialize)]
pub struct AccountState {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
    #[serde(skip)]
    disputes: HashMap<u32, Decimal>,
}

/*
#[derive(Debug, PartialEq)]
pub enum Dispute {
    Deposit(Decimal),
    Withdrawal(Decimal),
}
*/

impl AccountState {
    pub fn new(client: u16) -> Self {
        AccountState {
            client,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: false,
            disputes: HashMap::new(),
        }
    }

    pub fn deposit(&mut self, amount: Decimal) -> Result<(), Error> {
        if self.locked {
            return Err(Error::AccountLocked {
                client: self.client,
            });
        }
        self.available += amount;
        self.total += amount;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: Decimal) -> Result<(), Error> {
        if self.locked {
            return Err(Error::AccountLocked {
                client: self.client,
            });
        }
        if amount > self.total {
            return Err(Error::InsufficientFunds {
                client: self.client,
            });
        }
        self.available -= amount;
        self.total -= amount;
        Ok(())
    }

    pub fn dispute(&mut self, stored_transaction: &StoredTransaction) -> Result<(), Error> {
        if self.locked {
            return Err(Error::AccountLocked {
                client: self.client,
            });
        }
        if self.disputes.contains_key(&stored_transaction.tx()) {
            let client = self.client;
            let tx = stored_transaction.tx();
            return Err(Error::TransactionAlreadyDisputed { client, tx });
        }
        match stored_transaction {
            StoredTransaction::Deposit(deposit) => {
                self.available -= deposit.amount;
                self.held += deposit.amount;
                self.disputes.insert(deposit.tx, deposit.amount);
            }
            StoredTransaction::Withdrawal(withdrawal) => {
                // WTF!?!? Can some-one even dispute a deposit into their own account?
                // last I checked this is not a thing. Maybe I'm wrong, seems sketch
                // though so going to consider it an error for now.
                return Err(Error::DisputeOnWithdrawal {
                    client: self.client,
                    tx: withdrawal.tx,
                });
            }
        }
        Ok(())
    }

    pub fn resolve(&mut self, tx: u32) -> Result<(), Error> {
        if self.locked {
            return Err(Error::AccountLocked {
                client: self.client,
            });
        }
        if let Some(amount) = self.disputes.remove(&tx) {
            self.available += amount;
            self.held -= amount;
            Ok(())
        } else {
            Err(Error::DisputeNotFound {
                client: self.client,
                tx,
            })
        }
    }

    pub fn chargeback(&mut self, tx: u32) -> Result<(), Error> {
        if self.locked {
            return Err(Error::AccountLocked {
                client: self.client,
            });
        }
        if let Some(amount) = self.disputes.remove(&tx) {
            self.held -= amount;
            self.total -= amount;
            self.locked = true;
            Ok(())
        } else {
            Err(Error::DisputeNotFound {
                client: self.client,
                tx,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stored_transaction::{StoredDepositTransaction, StoredWithdrawalTransaction};

    #[test]
    fn test_deposit_on_fresh_account() {
        // given ...
        let mut account = AccountState::new(1);

        // when ...
        let result = account.deposit(Decimal::new(100, 2));

        // then ...
        assert_eq!(result, Ok(()));
        assert_eq!(account.available, Decimal::new(100, 2));
        assert_eq!(account.total, Decimal::new(100, 2));
        assert_eq!(account.held, Decimal::ZERO);
    }

    #[test]
    fn test_deposit_on_locked_account() {
        // given ...
        let mut account = AccountState::new(1);
        account.locked = true;

        // when ...
        let result = account.deposit(Decimal::new(100, 2));

        // then ...
        assert_eq!(result, Err(Error::AccountLocked { client: 1 }));
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.total, Decimal::ZERO);
        assert_eq!(account.held, Decimal::ZERO);
    }

    #[test]
    fn test_withdraw_on_sufficiently_funded_account() {
        // given ...
        let mut account = AccountState::new(1);
        account.deposit(Decimal::new(200, 2)).unwrap();

        // when ...
        let result = account.withdraw(Decimal::new(100, 2));

        // then ...
        assert_eq!(result, Ok(()));
        assert_eq!(account.available, Decimal::new(100, 2));
        assert_eq!(account.total, Decimal::new(100, 2));
        assert_eq!(account.held, Decimal::ZERO);
    }

    #[test]
    fn test_withdraw_on_empty_account() {
        // given ...
        let mut account = AccountState::new(1);

        // when ...
        let result = account.withdraw(Decimal::new(100, 2));

        // then ...
        assert_eq!(result, Err(Error::InsufficientFunds { client: 1 }));
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.total, Decimal::ZERO);
        assert_eq!(account.held, Decimal::ZERO);
    }

    #[test]
    fn test_over_withdraw_on_funded_account() {
        // given ...
        let mut account = AccountState::new(1);
        account.deposit(Decimal::new(100, 2)).unwrap();

        // when ...
        let result = account.withdraw(Decimal::new(200, 2));

        // then ...
        assert_eq!(result, Err(Error::InsufficientFunds { client: 1 }));
        assert_eq!(account.available, Decimal::new(100, 2));
        assert_eq!(account.total, Decimal::new(100, 2));
        assert_eq!(account.held, Decimal::ZERO);
    }

    #[test]
    fn test_withdrawal_on_locked_account() {
        // given ...
        let mut account = AccountState::new(1);
        account.deposit(Decimal::new(100, 2)).unwrap();
        account.locked = true;

        // when ...
        let result = account.withdraw(Decimal::new(50, 2));

        // then ...
        assert_eq!(result, Err(Error::AccountLocked { client: 1 }));
        assert_eq!(account.available, Decimal::new(100, 2));
        assert_eq!(account.total, Decimal::new(100, 2));
        assert_eq!(account.held, Decimal::ZERO);
    }

    #[test]
    fn test_dispute_on_deposit() {
        // given ...
        let mut account = AccountState::new(1);
        let amount = Decimal::new(100, 2);
        let deposit = StoredTransaction::Deposit(StoredDepositTransaction {
            tx: 1,
            client: 1,
            amount,
        });
        account.deposit(amount).unwrap();

        // when ...
        let result = account.dispute(&deposit);

        // then ...
        assert_eq!(result, Ok(()));
        assert_eq!(account.available, Decimal::new(0, 2));
        assert_eq!(account.held, Decimal::new(100, 2));
        assert_eq!(account.total, Decimal::new(100, 2));
        assert_eq!(account.disputes.get(&1), Some(&Decimal::new(100, 2)));
    }

    #[test]
    fn test_dispute_on_withdrawal() {
        // given ...
        let mut account = AccountState::new(1);
        let amount = Decimal::new(100, 2);
        let withdrawal = StoredTransaction::Withdrawal(StoredWithdrawalTransaction {
            tx: 1,
            client: 1,
            amount,
        });
        account.deposit(amount).unwrap();
        account.withdraw(amount).unwrap();

        // when ...
        let result = account.dispute(&withdrawal);

        // then ...
        assert_eq!(result, Err(Error::DisputeOnWithdrawal { client: 1, tx: 1 }));
        assert_eq!(account.available, Decimal::new(0, 2));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::ZERO);
        assert!(account.disputes.is_empty());
    }

    #[test]
    fn test_dispute_on_locked_account() {
        // given ...
        let mut account = AccountState::new(1);
        let amount = Decimal::new(100, 2);
        let deposit = StoredTransaction::Deposit(StoredDepositTransaction {
            tx: 1,
            client: 1,
            amount,
        });
        account.deposit(amount).unwrap();
        account.locked = true;

        // when ...
        let result = account.dispute(&deposit);

        // then ...
        assert_eq!(result, Err(Error::AccountLocked { client: 1 }));
        assert_eq!(account.available, Decimal::new(100, 2));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::new(100, 2));
        assert!(account.disputes.is_empty());
    }

    #[test]
    fn test_resolve_dispute() {
        // given ...
        let mut account = AccountState::new(1);
        let amount = Decimal::new(100, 2);
        let deposit = StoredTransaction::Deposit(StoredDepositTransaction {
            tx: 1,
            client: 1,
            amount,
        });
        account.deposit(amount).unwrap();
        account.dispute(&deposit).unwrap();

        // when ...
        let result = account.resolve(1);

        // then ...
        assert_eq!(result, Ok(()));
        assert_eq!(account.available, Decimal::new(100, 2));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::new(100, 2));
        assert!(!account.disputes.contains_key(&1));
    }

    #[test]
    fn test_resolve_on_locked_account() {
        // given ...
        let mut account = AccountState::new(1);
        let amount = Decimal::new(100, 2);
        let deposit = StoredTransaction::Deposit(StoredDepositTransaction {
            tx: 1,
            client: 1,
            amount,
        });
        account.deposit(amount).unwrap();
        account.dispute(&deposit).unwrap();
        account.locked = true;

        // when ...
        let result = account.resolve(1);

        // then ...
        assert_eq!(result, Err(Error::AccountLocked { client: 1 }));
        assert_eq!(account.available, Decimal::new(0, 2));
        assert_eq!(account.held, Decimal::new(100, 2));
        assert_eq!(account.total, Decimal::new(100, 2));
        assert!(account.disputes.contains_key(&1));
    }

    #[test]
    fn test_resolve_on_missing_dispute() {
        // given ...
        let mut account = AccountState::new(1);
        let amount = Decimal::new(100, 2);
        account.deposit(amount).unwrap();

        // when ...
        let result = account.resolve(1);

        // then ...
        assert_eq!(result, Err(Error::DisputeNotFound { client: 1, tx: 1 }));
        assert_eq!(account.available, Decimal::new(100, 2));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::new(100, 2));
    }

    #[test]
    fn test_chargeback() {
        // given ...
        let mut account = AccountState::new(1);
        let amount = Decimal::new(100, 2);
        let deposit = StoredTransaction::Deposit(StoredDepositTransaction {
            tx: 1,
            client: 1,
            amount,
        });
        account.deposit(amount).unwrap();
        account.dispute(&deposit).unwrap();

        // when ...
        let result = account.chargeback(1);

        // then ...
        assert_eq!(result, Ok(()));
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::ZERO);
        assert!(account.locked);
        assert!(!account.disputes.contains_key(&1));
    }

    #[test]
    fn test_chargeback_on_locked_account() {
        // given ...
        let mut account = AccountState::new(1);
        let amount = Decimal::new(100, 2);
        let deposit = StoredTransaction::Deposit(StoredDepositTransaction {
            tx: 1,
            client: 1,
            amount,
        });
        account.deposit(amount).unwrap();
        account.dispute(&deposit).unwrap();
        account.locked = true;

        // when ...
        let result = account.chargeback(1);

        // then ...
        assert_eq!(result, Err(Error::AccountLocked { client: 1 }));
        assert_eq!(account.available, Decimal::new(0, 2));
        assert_eq!(account.held, Decimal::new(100, 2));
        assert_eq!(account.total, Decimal::new(100, 2));
        assert!(account.disputes.contains_key(&1));
    }

    #[test]
    fn test_chargeback_on_missing_dispute() {
        // given ...
        let mut account = AccountState::new(1);
        let amount = Decimal::new(100, 2);
        account.deposit(amount).unwrap();

        // when ...
        let result = account.chargeback(1);

        // then ...
        assert_eq!(result, Err(Error::DisputeNotFound { client: 1, tx: 1 }));
        assert_eq!(account.available, Decimal::new(100, 2));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::new(100, 2));
    }
}
