use std::vec;
use std::cell::RefCell;
use std::rc::Rc;
use std::ops::{Add, Mul, Sub, Div};
// use base64::{engine::general_purpose, Engine};
use crate::base_market::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke,
    instruction::Instruction,
};
use hex::FromHex;
use crate::utils::utils::*;
use crate::common::accounts_iter::*;

use super::mock_reverse_market::MockReverseMarketPool;
pub struct RaydiumCPMMMarketPool<'a, 'info> {
    pub pool_program: &'a AccountInfo<'info>,
    pub user: &'a AccountInfo<'info>,
    pub authorithy: &'a AccountInfo<'info>,
    pub amm_config: &'a AccountInfo<'info>,
    pub pool_state: &'a AccountInfo<'info>,
    pub user_token_account_x: &'a AccountInfo<'info>,
    pub user_token_account_y: &'a AccountInfo<'info>,
    pub pool_token_account_x: &'a AccountInfo<'info>,
    pub pool_token_account_y: &'a AccountInfo<'info>,
    pub token_x_program: &'a AccountInfo<'info>,
    pub token_y_program: &'a AccountInfo<'info>,
    pub token_x_mint: &'a AccountInfo<'info>,
    pub token_y_mint: &'a AccountInfo<'info>,
    pub observation: &'a AccountInfo<'info>,


    pub amount_x: u64,
    pub amount_y: u64,
    pub fee_denominator: u64,
    pub fee_numerator: u64,
}

impl<'a, 'info> RaydiumCPMMMarketPool<'a, 'info> {
    pub fn init(&mut self) {
        let amm_config_data = self.amm_config.data.borrow();
        let pool_data = self.pool_state.data.borrow();

        let mut fee_buffer = [0u8; 8];
        fee_buffer.copy_from_slice(&amm_config_data[8 + 4..8 + 4 + 8]);
        self.fee_numerator = u64::from_le_bytes(fee_buffer);
        self.fee_denominator = 1000000u64;

        let vaule_x_amount = unpack_token_account_ammount(self.pool_token_account_x).unwrap();
        let vaule_y_amount = unpack_token_account_ammount(self.pool_token_account_y).unwrap();

        let mut protocol_fee_x_buffer = [0u8; 8];
        let mut protocol_fee_y_buffer = [0u8; 8];
        protocol_fee_x_buffer.copy_from_slice(&pool_data[8 + 10 * 32 + 5 + 8..8 + 10 * 32 + 5 + 8 + 8]);
        protocol_fee_y_buffer.copy_from_slice(&pool_data[8 + 10 * 32 + 5 + 8 + 8..8 + 10 * 32 + 5 + 8 + 8 + 8]);
        let protocol_fee_x = u64::from_le_bytes(protocol_fee_x_buffer);
        let protocol_fee_y = u64::from_le_bytes(protocol_fee_y_buffer);

        let mut fund_fee_x_buffer = [0u8; 8];
        let mut fund_fee_y_buffer = [0u8; 8];
        fund_fee_x_buffer.copy_from_slice(&pool_data[8 + 10 * 32 + 5 + 8 + 8 + 8..8 + 10 * 32 + 5 + 8 + 8 + 8 + 8]);
        fund_fee_y_buffer.copy_from_slice(&pool_data[8 + 10 * 32 + 5 + 8 + 8 + 8 + 8..8 + 10 * 32 + 5 + 8 + 8 + 8 + 8 + 8]);
        let fund_fee_x = u64::from_le_bytes(fund_fee_x_buffer);
        let fund_fee_y = u64::from_le_bytes(fund_fee_y_buffer);

        self.amount_x = vaule_x_amount - protocol_fee_x - fund_fee_x;
        self.amount_y = vaule_y_amount - protocol_fee_y - fund_fee_y;
        

    }
}

impl<'a, 'info> BaseMarketPool<'a, 'info> for RaydiumCPMMMarketPool<'a, 'info> {

    fn get_real_time_user_token_amount_x(&self) -> u64 {
        unpack_token_account_ammount(self.user_token_account_x).unwrap()
    }

    fn get_real_time_user_token_amount_y(&self) -> u64 {
        unpack_token_account_ammount(self.user_token_account_y).unwrap()
    }

    fn valid(&self) -> bool {
        self.amount_x > 0 && self.amount_y > 0
    }

    #[cfg(feature = "debug-out")]
    fn out_x(&self, in_y: u64) -> u64 {
        let fee = (in_y as u128).mul(self.fee_numerator() as u128).add(self.fee_denominator() as u128).sub(1).div(self.fee_denominator() as u128);
        let remain_y = in_y as u128 - fee;
        let invariant = (self.amount_x as u128).mul(self.amount_y as u128);

        let new_swap_source_amount = (self.amount_y as u128).add(remain_y);
        let new_swap_destination_amount = invariant.div_ceil(new_swap_source_amount);

        let destination_amount_swapped = (self.amount_x as u128).sub(new_swap_destination_amount);
        
        destination_amount_swapped as u64
    }

    #[cfg(feature = "debug-out")]
    fn out_y(&self, in_x: u64) -> u64 {
        let fee = (in_x as u128).mul(self.fee_numerator() as u128).add(self.fee_denominator() as u128).sub(1).div(self.fee_denominator() as u128);
        let remain_x = in_x as u128 - fee;
        let invariant = (self.amount_x as u128).mul(self.amount_y as u128);

        let new_swap_source_amount = (self.amount_x as u128).add(remain_x);
        let new_swap_destination_amount = invariant.div_ceil(new_swap_source_amount);

        let destination_amount_swapped = (self.amount_y as u128).sub(new_swap_destination_amount);
        destination_amount_swapped as u64
    }

    fn x_mint(&self) -> &Pubkey {
        &self.token_x_mint.key
    }
    fn y_mint(&self) -> &Pubkey {
        &self.token_y_mint.key
    }
    fn fee_denominator(&self) -> u64 {
        self.fee_denominator
    }
    fn fee_numerator(&self) -> u64 {
        self.fee_numerator
    }
    fn amount_x(&self) -> u64 {
        self.amount_x
    }
    fn amount_y(&self) -> u64 {
        self.amount_y
    }
    // only for
    fn price(&self) -> f64 {
        1.0
    }
    fn swap(&self, y2x: bool, amount_in: u64) -> Result<()> {
        // "8fbe 5ada c41e 33de b140b8f0 0500 0000 0000 0000 0000 0000"
        let mut swap_data = Vec::from_hex("8fbe5adac41e33de").unwrap();
        swap_data.append(borsh::to_vec(&amount_in).unwrap().as_mut());
        swap_data.append(borsh::to_vec(&(0u64)).unwrap().as_mut());

        
        let account_meta = vec![
            AccountMeta::new(self.user.key(), true),
            AccountMeta::new_readonly(self.authorithy.key(), false),
            AccountMeta::new_readonly(self.amm_config.key(), false),
            AccountMeta::new(self.pool_state.key(), false),
            match y2x {
                true => AccountMeta::new(self.user_token_account_y.key(), false),
                false => AccountMeta::new(self.user_token_account_x.key(), false),
            },
            match y2x {
                true => AccountMeta::new(self.user_token_account_x.key(), false),
                false => AccountMeta::new(self.user_token_account_y.key(), false),
            },
            match y2x {
                true => AccountMeta::new(self.pool_token_account_y.key(), false),
                false => AccountMeta::new(self.pool_token_account_x.key(), false),
            },
            match y2x {
                true => AccountMeta::new(self.pool_token_account_x.key(), false),
                false => AccountMeta::new(self.pool_token_account_y.key(), false),
            },
            match y2x {
                true => AccountMeta::new_readonly(self.token_y_program.key(), false),
                false => AccountMeta::new_readonly(self.token_x_program.key(), false),
            },
            match y2x {
                true => AccountMeta::new_readonly(self.token_x_program.key(), false),
                false => AccountMeta::new_readonly(self.token_y_program.key(), false),
            },
            match y2x {
                true => AccountMeta::new_readonly(self.token_y_mint.key(), false),
                false => AccountMeta::new_readonly(self.token_x_mint.key(), false),
            },
            match y2x {
                true => AccountMeta::new_readonly(self.token_x_mint.key(), false),
                false => AccountMeta::new_readonly(self.token_y_mint.key(), false),
            },
            AccountMeta::new(self.observation.key(), false),
        ];
        let accounts_info = vec![
            self.user.to_account_info(),
            self.authorithy.to_account_info(),
            self.amm_config.to_account_info(),
            self.pool_state.to_account_info(),
            match y2x {
                true => self.user_token_account_y.to_account_info(),
                false => self.user_token_account_x.to_account_info(),
            },
            match y2x {
                true => self.user_token_account_x.to_account_info(),
                false => self.user_token_account_y.to_account_info(),
            },
            match y2x {
                true => self.pool_token_account_y.to_account_info(),
                false => self.pool_token_account_x.to_account_info(),
            },
            match y2x {
                true => self.pool_token_account_x.to_account_info(),
                false => self.pool_token_account_y.to_account_info(),
            },
            match y2x {
                true => self.token_y_program.to_account_info(),
                false => self.token_x_program.to_account_info(),
            },
            match y2x {
                true => self.token_x_program.to_account_info(),
                false => self.token_y_program.to_account_info(),
            },
            match y2x {
                true => self.token_y_mint.to_account_info(),
                false => self.token_x_mint.to_account_info(),
            },
            match y2x {
                true => self.token_x_mint.to_account_info(),
                false => self.token_y_mint.to_account_info(),
            },
            self.observation.to_account_info(),
        ];
        let swap_ix = Instruction {
            program_id: self.pool_program.key(),
            accounts: account_meta,
            data: swap_data,
        };
        invoke(&swap_ix, &accounts_info)?;
        Ok(())
    }
    fn market_type(&self) -> MarketType {
        MarketType::NormalAMM
    }
}

impl<'a, 'info> CreateMarket<'a, 'info> for RaydiumCPMMMarketPool<'a, 'info> {
    fn create_market(
            base: &'a crate::model::base_model::BaseModel<'a, 'info>,
            min_x_user_account: &'a Option<UncheckedAccount<'info>>,
            mint_x_mint_account: &'a Option<UncheckedAccount<'info>>,
            accounts_iter: Rc<RefCell<AccountsIter<'a, 'info>>>,
            reverse: bool,
        ) -> Box<dyn BaseMarketPool<'a, 'info> + 'a> {
        let account_vec = accounts_iter.borrow_mut().take(9);
        let mut raydium = Box::new(RaydiumCPMMMarketPool {
                pool_program: account_vec[0].as_ref().unwrap(),
                user: base.user.as_ref(),
                authorithy: account_vec[1].as_ref().unwrap(),
                amm_config: account_vec[2].as_ref().unwrap(),
                pool_state: account_vec[3].as_ref().unwrap(),
                user_token_account_x: match reverse {
                    true => base.user_token_base.as_ref(),
                    false => min_x_user_account.as_ref().unwrap(),
                },
                user_token_account_y: match reverse {
                    true => min_x_user_account.as_ref().unwrap(),
                    false => base.user_token_base.as_ref(),
                },
                pool_token_account_x: account_vec[4].as_ref().unwrap(),
                pool_token_account_y: account_vec[5].as_ref().unwrap(),
                token_x_program: account_vec[6].as_ref().unwrap(),
                token_y_program: account_vec[7].as_ref().unwrap(),
                token_x_mint: match reverse {
                    true => base.token_base_mint,
                    false => mint_x_mint_account.as_ref().unwrap(),
                },
                token_y_mint: match reverse {
                    true => mint_x_mint_account.as_ref().unwrap(),
                    false => base.token_base_mint,
                },
                observation: account_vec[8].as_ref().unwrap(),

                amount_x: 0,
                amount_y: 0,
                fee_numerator: 0,
                fee_denominator: 0,
        });
        raydium.init();
        if reverse {
            return Box::new(
                MockReverseMarketPool {
                    market: raydium,
                }
            )
        }
        raydium
    }
}

