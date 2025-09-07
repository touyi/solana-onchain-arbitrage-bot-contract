use crate::model::base_model::*;
use crate::common::accounts_iter::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarketType {
    NormalCMM,
    NormalAMM,
    NormalCLMM,
}

pub trait BaseMarketPool<'a, 'info> {
    fn fee_denominator(&self) -> u64;
    fn fee_numerator(&self) -> u64;
    fn amount_x(&self) -> u64;
    fn amount_y(&self) -> u64;
    // only for
    fn price(&self) -> f64 {
        1.0
    }
    fn x_mint(&self) -> &Pubkey;

    fn y_mint(&self) -> &Pubkey;

    #[cfg(feature = "debug-out")]
    fn out_x(&self, in_y: u64) -> u64;
    
    #[cfg(feature = "debug-out")]
    fn out_y(&self, in_x: u64) -> u64;

    fn limit_in_x(&self) -> u64 {
        u64::MAX
    }
    fn limit_in_y(&self) -> u64 {
        u64::MAX
    }

    fn liquidity(&self) -> u128 {
        u128::MAX
    }
    fn sqrt_price(&self) -> f64 {
        1.0
    }

    fn valid(&self) -> bool;

    fn get_real_time_user_token_amount_x(&self) -> u64;
    fn get_real_time_user_token_amount_y(&self) -> u64;

    fn swap(&self, y2x: bool, amount_in: u64) -> Result<()>;
    fn market_type(&self) -> MarketType;
}

pub trait CreateMarket<'a, 'info> {
    fn create_market(
        base: &'a BaseModel<'a, 'info>,
        min_x_user_accounts: &'a Option<UncheckedAccount<'info>>,
        mint_x_mint_accounts: &'a Option<UncheckedAccount<'info>>,
        accounts_itesr: Rc<RefCell<AccountsIter<'a, 'info>>>,
        reverse: bool,
    ) -> Box<dyn BaseMarketPool<'a, 'info> + 'a>;
}

