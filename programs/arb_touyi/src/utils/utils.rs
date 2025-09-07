use anchor_lang::prelude::*;
use anchor_spl::token::{spl_token,close_account,CloseAccount};
use anchor_spl::associated_token::{self, AssociatedToken, Create};
use anchor_lang::solana_program::program_pack::Pack;
use crate::{model::errors::*, model::errors::MyErrorCode};

pub fn create_associated_token_account<'a>(
    payer: AccountInfo<'a>,
    token_mint: AccountInfo<'a>,
    token_program: AccountInfo<'a>,
    associated_token: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
    associated_token_program: AccountInfo<'a>
) -> Result<()> {
    
    let cpi_accounts = Create {
        payer: payer.to_account_info(),
        associated_token: associated_token.to_account_info(),
        authority: payer.to_account_info(),
        mint: token_mint.to_account_info(),
        system_program: system_program.to_account_info(),
        token_program: token_program.to_account_info(),
    };

    let cpi_program = associated_token_program.to_account_info();

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    associated_token::create_idempotent(cpi_ctx)
}

pub fn close_token_account<'a>(
    payer: AccountInfo<'a>,
    token_account: AccountInfo<'a>,
    token_program: AccountInfo<'a>
) -> Result<()> {
    let cpi_accounts = CloseAccount {
        account: token_account.to_account_info(),
        destination: payer.to_account_info(),
        authority: payer.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(token_program.to_account_info(), cpi_accounts);
    close_account(cpi_ctx)
}


pub fn is_bit_set(num: u8, bit_index: u8) -> bool {
    (num & (1 << bit_index)) != 0
}

pub fn unpack_token_account_ammount(account: &AccountInfo) -> Result<u64> {
    if account.data_len() >= spl_token::state::Account::LEN {
        let mut amount_buffer = [0u8; 8];
        let account_data = account.data.borrow();
        amount_buffer.copy_from_slice(&account_data[64..64 + 8]);
        let amount = u64::from_le_bytes(amount_buffer);
        return Ok(amount);
    }
    msg!("{}", account.key);
    return Err(error!(MyErrorCode::InvalidBaseAccount));
    
    
}

pub fn unpack_token_supply_ammount(account: &AccountInfo) -> Result<u64> {
    if account.data_len() >= spl_token::state::Mint::LEN {
        let mut amount_buffer = [0u8; 8];
        let account_data = account.data.borrow();
        amount_buffer.copy_from_slice(&account_data[36..36 + 8]);
        let amount = u64::from_le_bytes(amount_buffer);
        return Ok(amount);
    }
    msg!("supply {}", account.key);
    return Err(error!(MyErrorCode::InvalidBaseAccount));
    
    
}