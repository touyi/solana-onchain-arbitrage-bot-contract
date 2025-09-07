use anchor_lang::prelude::*;

use crate::accounts;

pub struct AccountsIter<'a, 'info> {
    accounts: &'a Vec<&'a Option<UncheckedAccount<'info>>>,
    index: usize,
}

impl<'a, 'info> AccountsIter<'a, 'info> {
    pub fn new(accounts: &'a Vec<&'a Option<UncheckedAccount<'info>>>) -> Self {
        Self { 
            accounts,
            index: 0,
        }
    }

    pub fn print_index(&self) {
        msg!("now index:{}", self.index)
    }

    pub fn take(&mut self, n: usize) -> &'a [&'a Option<UncheckedAccount<'info>>] {
        self.index += n;
        return &self.accounts[self.index - n..self.index]
    }
}