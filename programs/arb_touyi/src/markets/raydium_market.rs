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
use crate::utils::utils::*;
use crate::common::accounts_iter::*;

use super::mock_reverse_market::MockReverseMarketPool;

pub struct RaydiumAMMMarketPool<'a, 'info> {
    pub pool_program: &'a AccountInfo<'info>,
    pub token_program: &'a AccountInfo<'info>,
    pub amm_info: &'a AccountInfo<'info>,
    pub amm_authorithy: &'a AccountInfo<'info>,
    pub pool_coin_token_account: &'a AccountInfo<'info>,
    pub pool_pc_token_account: &'a AccountInfo<'info>,
    pub user_token_account_coin: &'a AccountInfo<'info>,
    pub user_token_account_pc: &'a AccountInfo<'info>,
    pub user: &'a AccountInfo<'info>,

    pub fee_denominator: u64,
    pub fee_numerator: u64,
    pub amount_x: u64,
    pub amount_y: u64,
    pub x_mint: &'a Pubkey,
    pub y_mint: &'a Pubkey,
}

impl<'a, 'info> RaydiumAMMMarketPool<'a, 'info> {
    pub fn init(&mut self) {
        let amm_data = self.amm_info.data.borrow();

        let coin_amount = unpack_token_account_ammount(self.pool_coin_token_account).unwrap();
        let pc_amount = unpack_token_account_ammount(self.pool_pc_token_account).unwrap();

        let mut need_take_pnl_coin_buffer = [0u8; 8];
        need_take_pnl_coin_buffer.copy_from_slice(&amm_data[192..192 + 8]);
        let need_take_pnl_coin = u64::from_le_bytes(need_take_pnl_coin_buffer);

        let mut need_take_pnl_pc_buffer = [0u8; 8];
        need_take_pnl_pc_buffer.copy_from_slice(&amm_data[192 + 8..192 + 8 + 8]);
        let need_take_pnl_pc = u64::from_le_bytes(need_take_pnl_pc_buffer);

        self.amount_x = coin_amount - need_take_pnl_coin;
        self.amount_y = pc_amount - need_take_pnl_pc;
        
        let mut fee_numerator_buffer = [0u8; 8];
        fee_numerator_buffer.copy_from_slice(&amm_data[128 + 48..128 + 48 + 8]);
        self.fee_numerator = u64::from_le_bytes(fee_numerator_buffer);
    
        let mut fee_denominator_buffer = [0u8; 8];
        fee_denominator_buffer.copy_from_slice(&amm_data[128 + 48 + 8..128 + 48 + 8 + 8]);
        self.fee_denominator = u64::from_le_bytes(fee_denominator_buffer);
    
    }
}

impl<'a, 'info> BaseMarketPool<'a, 'info> for RaydiumAMMMarketPool<'a, 'info> {

    fn get_real_time_user_token_amount_x(&self) -> u64 {
        unpack_token_account_ammount(self.user_token_account_coin).unwrap()
    }

    fn get_real_time_user_token_amount_y(&self) -> u64 {
        unpack_token_account_ammount(self.user_token_account_pc).unwrap()
    }

    fn valid(&self) -> bool {
        self.amount_x > 0 && self.amount_y > 0
    }

    #[cfg(feature = "debug-out")]
    fn out_x(&self, in_y: u64) -> u64 {
        let in_y = in_y as u128;
        let fee_n = self.fee_numerator() as u128;
        let fee_d = self.fee_denominator() as u128;
        let fee = in_y.mul(fee_n).add(fee_d.sub(1)).div(fee_d);

        let remain_y = in_y.sub(fee);

        let x = self.amount_x() as u128;
        let y = self.amount_y() as u128;
        let denominator = y.add(remain_y);
        x.mul(remain_y).div(denominator) as u64
    }

    #[cfg(feature = "debug-out")]
    fn out_y(&self, in_x: u64) -> u64 {
        let in_x = in_x as u128;
        let fee_n = self.fee_numerator() as u128;
        let fee_d = self.fee_denominator() as u128;
        let fee = in_x.mul(fee_n).add(fee_d.sub(1)).div(fee_d);
        let remain_x = in_x.sub(fee);

        let x = self.amount_x() as u128;
        let y = self.amount_y() as u128;
        let denominator = x.add(remain_x);
        y.mul(remain_x).div(denominator) as u64
    }

    fn x_mint(&self) -> &Pubkey {
        &self.x_mint
    }
    fn y_mint(&self) -> &Pubkey {
        &self.y_mint
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
        let mut swap_data = vec![9u8];
        let min_ammount_out:u64 = 0;
        swap_data.extend(borsh::to_vec(&amount_in).unwrap());
        swap_data.extend(borsh::to_vec(&min_ammount_out).unwrap());
        let account_meta = vec![
            AccountMeta::new(self.token_program.key(), false),
            AccountMeta::new(self.amm_info.key(), false),
            AccountMeta::new_readonly(self.amm_authorithy.key(), false),
            AccountMeta::new(self.amm_info.key(), false),
            AccountMeta::new(self.pool_coin_token_account.key(), false),
            AccountMeta::new(self.pool_pc_token_account.key(), false),
            AccountMeta::new(self.amm_info.key(), false),
            AccountMeta::new(self.amm_info.key(), false),
            AccountMeta::new(self.amm_info.key(), false),
            AccountMeta::new(self.amm_info.key(), false),
            AccountMeta::new(self.amm_info.key(), false),
            AccountMeta::new(self.amm_info.key(), false),
            AccountMeta::new(self.amm_info.key(), false),
            AccountMeta::new(self.amm_info.key(), false),
            AccountMeta::new(match y2x {
                true => {
                    self.user_token_account_pc.key()
                    
                },
                false => {
                    self.user_token_account_coin.key()
                }
            } , false),
            AccountMeta::new(match y2x {
                true => {
                    self.user_token_account_coin.key()
                },
                false => {
                    self.user_token_account_pc.key()
                }
            }, false),
            AccountMeta::new(self.user.key(), true),
        ];
        let accounts_info = vec![
            self.token_program.to_account_info(),
            self.amm_info.to_account_info(),
            self.amm_authorithy.to_account_info(),
            self.amm_info.to_account_info(),
            self.pool_coin_token_account.to_account_info(),
            self.pool_pc_token_account.to_account_info(),
            self.amm_info.to_account_info(),
            self.amm_info.to_account_info(),
            self.amm_info.to_account_info(),
            self.amm_info.to_account_info(),
            self.amm_info.to_account_info(),
            self.amm_info.to_account_info(),
            self.amm_info.to_account_info(),
            self.amm_info.to_account_info(),
            match y2x {
                true => {
                    self.user_token_account_pc.to_account_info()
                },
                false => {
                    self.user_token_account_coin.to_account_info()
                }
            },
            match y2x {
                true => {
                    self.user_token_account_coin.to_account_info()
                },
                false => {
                    self.user_token_account_pc.to_account_info()
                }
            },
            self.user.to_account_info(),
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

impl<'a, 'info> CreateMarket<'a, 'info> for RaydiumAMMMarketPool<'a, 'info> {
    fn create_market(
            base: &'a crate::model::base_model::BaseModel<'a, 'info>,
            min_x_user_account: &'a Option<UncheckedAccount<'info>>,
            mint_x_mint_account: &'a Option<UncheckedAccount<'info>>,
            accounts_iter: Rc<RefCell<AccountsIter<'a, 'info>>>,
            reverse: bool,
        ) -> Box<dyn BaseMarketPool<'a, 'info> + 'a> {
        let account_vec = accounts_iter.borrow_mut().take(5);
        // TODO(touyi): check the order of the accounts
        let mut raydium = Box::new(RaydiumAMMMarketPool {
                pool_program: account_vec[0].as_ref().unwrap(),
                amm_info: account_vec[1].as_ref().unwrap(),
                amm_authorithy: account_vec[2].as_ref().unwrap(),
                pool_coin_token_account: account_vec[3].as_ref().unwrap(),
                pool_pc_token_account: account_vec[4].as_ref().unwrap(),

                user_token_account_coin: match reverse {
                    true => base.user_token_base.as_ref(),
                    false => min_x_user_account.as_ref().unwrap(),
                },
                user_token_account_pc: match reverse {
                    true => min_x_user_account.as_ref().unwrap(),
                    false => base.user_token_base.as_ref(),
                },
                user: base.user.as_ref(),

                x_mint: match reverse {
                    true => base.token_base_mint.key,
                    false => mint_x_mint_account.as_ref().unwrap().key,
                },
                y_mint: match reverse {
                    true => mint_x_mint_account.as_ref().unwrap().key,
                    false => base.token_base_mint.key,
                },
                token_program: base.token_program.as_ref(),

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

