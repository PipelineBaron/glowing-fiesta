use crate::account_state::AccountState;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct AccountStore {
    accounts: HashMap<u16, AccountState>,
}

impl AccountStore {
    pub fn get_or_create(&mut self, client_id: u16) -> &mut AccountState {
        self.accounts
            .entry(client_id)
            .or_insert_with(|| AccountState::new(client_id))
    }

    pub fn iter(&self) -> impl Iterator<Item = &AccountState> {
        self.accounts.values()
    }
}
