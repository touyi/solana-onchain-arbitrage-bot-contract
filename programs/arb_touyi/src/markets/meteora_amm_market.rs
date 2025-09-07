use std::vec;
use std::cell::{Ref, RefCell};
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
const LOCKED_PROFIT_DEGRADATION_DENOMINATOR : u128 = 1000000000000u128;
pub struct MeteoraAMMMarketPool<'a, 'info> {
    pub pool_program: &'a AccountInfo<'info>,
    pub pool_state: &'a AccountInfo<'info>,
    pub user_token_account_x: &'a AccountInfo<'info>,
    pub user_token_account_y: &'a AccountInfo<'info>,
    pub x_vault: &'a AccountInfo<'info>,
    pub y_vault: &'a AccountInfo<'info>,
    pub x_token_vault: &'a AccountInfo<'info>,
    pub y_token_vault: &'a AccountInfo<'info>,
    pub x_vault_lp_mint: &'a AccountInfo<'info>,
    pub y_vault_lp_mint: &'a AccountInfo<'info>,
    pub x_vault_lp: &'a AccountInfo<'info>,
    pub y_vault_lp: &'a AccountInfo<'info>,
    pub protocol_fee_account_x: &'a AccountInfo<'info>,
    pub protocol_fee_account_y: &'a AccountInfo<'info>,
    pub user: &'a AccountInfo<'info>,
    pub vault_program: &'a AccountInfo<'info>,
    pub token_program: &'a AccountInfo<'info>,

    pub fee_denominator: u64,
    pub fee_numerator: u64,
    pub amount_x: u64,
    pub amount_y: u64,
    pub x_mint: &'a Pubkey,
    pub y_mint: &'a Pubkey,

    pub vault_x_withdrawable_amount: u64,
    pub vault_y_withdrawable_amount: u64,
    pub vault_x_lp_supply: u64,
    pub vault_y_lp_supply: u64,
    pub pool_x_lp_amount: u64,
    pub pool_y_lp_amount: u64,
    pub protocol_fee_numerator: u64,
    pub protocol_fee_denominator: u64,

}

impl<'a, 'info> MeteoraAMMMarketPool<'a, 'info> {
    pub fn init(&mut self) {
        let pool_state_data = self.pool_state.data.borrow();

        let mut fee_numerator_buffer = [0u8; 8];
        fee_numerator_buffer.copy_from_slice(&pool_state_data[8 + 9 * 32 + 2 + 8 + 24..8 + 9 * 32 + 2 + 8 + 24 + 8]);
        self.fee_numerator = u64::from_le_bytes(fee_numerator_buffer);

        let mut fee_denominator_buffer = [0u8; 8];
        fee_denominator_buffer.copy_from_slice(&pool_state_data[8 + 9 * 32 + 2 + 8 + 24 + 8..8 + 9 * 32 + 2 + 8 + 24 + 8 + 8]);
        self.fee_denominator = u64::from_le_bytes(fee_denominator_buffer);

        #[cfg(feature = "debug-out")]
        {
            let mut protocol_fee_numerator_buffer = [0u8; 8];
            protocol_fee_numerator_buffer.copy_from_slice(&pool_state_data[8 + 9 * 32 + 2 + 8 + 24 + 8 + 8..8 + 9 * 32 + 2 + 8 + 24 + 8 + 8 + 8]);
            self.protocol_fee_numerator = u64::from_le_bytes(protocol_fee_numerator_buffer);

            let mut protocol_fee_denominator_buffer = [0u8; 8];
            protocol_fee_denominator_buffer.copy_from_slice(&pool_state_data[8 + 9 * 32 + 2 + 8 + 24 + 8 + 8 + 8..8 + 9 * 32 + 2 + 8 + 24 + 8 + 8 + 8 + 8]);
            self.protocol_fee_denominator = u64::from_le_bytes(protocol_fee_denominator_buffer);

            // msg!("protocol_fee_numerator: {}, protocol_fee_denominator: {}", self.protocol_fee_numerator, self.protocol_fee_denominator);
        }
        


        let current_time = Clock::get().unwrap().unix_timestamp as u64;

        // msg!("current_time: {}, fee_numerator: {}, fee_denominator: {}", current_time, self.fee_numerator, self.fee_denominator);

        {
            self.vault_x_withdrawable_amount = self.cal_vault_withdrawable_amount(current_time, self.x_vault.data.borrow());
            self.pool_x_lp_amount = unpack_token_account_ammount(self.x_vault_lp).unwrap();
            self.vault_x_lp_supply = unpack_token_supply_ammount(self.x_vault_lp_mint).unwrap();
            self.amount_x = self.get_amount_by_share(self.pool_x_lp_amount as u128, 
                                                    self.vault_x_withdrawable_amount as u128, 
                                                    self.vault_x_lp_supply as u128) as u64;
            // msg!("vault_x_withdrawable_amount: {}, pool_vault_lp: {}, vault_lp_supply: {}, amount_x: {}", self.vault_x_withdrawable_amount, self.pool_x_lp_amount, self.vault_x_lp_supply, self.amount_x);
        }

        {
            self.vault_y_withdrawable_amount = self.cal_vault_withdrawable_amount(current_time, self.y_vault.data.borrow());
            self.pool_y_lp_amount = unpack_token_account_ammount(self.y_vault_lp).unwrap();
            self.vault_y_lp_supply = unpack_token_supply_ammount(self.y_vault_lp_mint).unwrap();
            self.amount_y = self.get_amount_by_share(self.pool_y_lp_amount as u128, 
                                                    self.vault_y_withdrawable_amount as u128, 
                                                    self.vault_y_lp_supply as u128) as u64;
            // msg!("vault_y_withdrawable_amount: {}, pool_vault_lp: {}, vault_lp_supply: {}, amount_y: {}", self.vault_y_withdrawable_amount, self.pool_y_lp_amount, self.vault_y_lp_supply, self.amount_y);
        }
    
    }

    fn get_amount_by_share(&self, pool_vault_lp: u128, vault_withdrawable_amount: u128, vault_lp_supply: u128) -> u128 {
        match vault_lp_supply == 0 {
            true => 0,
            false => (pool_vault_lp).mul(vault_withdrawable_amount).div(vault_lp_supply),
        }
    }

    fn cal_vault_withdrawable_amount(&self, current_time : u64, state_data: Ref<&mut[u8]>) -> u64 {
        let vault_withdrawable_amount: u128;
        let (last_report, 
            locked_profit_degradation, 
            last_updated_locked_profit,
            vault_total_amount) = self.get_last_state(state_data);

        let duration = current_time - last_report;
        let locked_fund_ratio = (duration as u128).mul(locked_profit_degradation as u128);

        if locked_fund_ratio > LOCKED_PROFIT_DEGRADATION_DENOMINATOR {
            vault_withdrawable_amount = vault_total_amount as u128;
        } else {
            let locked_profit = (last_updated_locked_profit as u128)
                .mul(LOCKED_PROFIT_DEGRADATION_DENOMINATOR.sub(locked_fund_ratio))
                .div(LOCKED_PROFIT_DEGRADATION_DENOMINATOR);
            vault_withdrawable_amount = (vault_total_amount as u128).sub(locked_profit as u128);
        }
        return vault_withdrawable_amount as u64;
    }

    fn get_last_state(&self, state_data: Ref<&mut[u8]>) -> (u64, u64, u64, u64) {

        let mut last_updated_locked_profit_buffer = [0u8; 8];
        last_updated_locked_profit_buffer.copy_from_slice(&state_data[11 + 8 + 37 * 32..11 + 8 + 37 * 32 + 8]);
        let last_updated_locked_profit = u64::from_le_bytes(last_updated_locked_profit_buffer);

        let mut last_report_buffer = [0u8; 8];
        last_report_buffer.copy_from_slice(&state_data[11 + 8 + 37 * 32 + 8..11 + 8 + 37 * 32 + 8 + 8]);
        let last_report = u64::from_le_bytes(last_report_buffer);

        let mut locked_profit_degradation_buffer = [0u8; 8];
        locked_profit_degradation_buffer.copy_from_slice(&state_data[11 + 8 + 37 * 32 + 8 + 8..11 + 8 + 37 * 32 + 8 + 8 + 8]);
        let locked_profit_degradation = u64::from_le_bytes(locked_profit_degradation_buffer);

        let mut vault_total_amount_buffer = [0u8; 8];
        vault_total_amount_buffer.copy_from_slice(&state_data[11..11 + 8]);
        let vault_total_amount = u64::from_le_bytes(vault_total_amount_buffer);
        // msg!("last_report: {}, locked_profit_degradation: {}, last_updated_locked_profit: {}, vault_total_amount: {}", last_report, locked_profit_degradation, last_updated_locked_profit, vault_total_amount);
        return (last_report, locked_profit_degradation, last_updated_locked_profit, vault_total_amount);
    }

    #[cfg(feature = "debug-out")]
    fn cal_out_amount(&self, 
                    source_amount: u128,
                    swap_source_amount: u128,
                    swap_dst_amount: u128,
                    source_vault_withdrawable_amount: u128,
                    swap_source_pool_vault_lp: u128,
                    swap_source_lp_supply: u128) -> u128 {
        let trade_fee = source_amount.mul(self.fee_numerator() as u128).div(self.fee_denominator() as u128);
        let protocol_fee = trade_fee.mul(self.protocol_fee_numerator as u128).div(self.protocol_fee_denominator as u128);
        let trade_fee_after_protocol_fee = trade_fee.sub(protocol_fee);

        let before_swap_source_amount = swap_source_amount;
        let source_amount_less_protocol_fee = source_amount.sub(protocol_fee);

        // getUnmintAmount 1.mul(3).div(2);
        let source_vault_lp = source_amount_less_protocol_fee.mul(swap_source_lp_supply).div(source_vault_withdrawable_amount);
        let source_vault_total_amount = source_vault_withdrawable_amount.add(source_amount_less_protocol_fee);

        let after_swap_source_amount = self.get_amount_by_share(
            source_vault_lp.add(swap_source_pool_vault_lp),
            source_vault_total_amount,
            swap_source_lp_supply.add(source_vault_lp));

            // msg!("trade_fee_after_protocol_fee: {} before_swap_source_amount: {} after_swap_source_amount:{}, source_vault_lp:{} swap_source_pool_vault_lp:{}", 
            // trade_fee_after_protocol_fee, 
            // before_swap_source_amount, 
            // after_swap_source_amount,
            // source_vault_lp,
            // swap_source_pool_vault_lp);

        let actual_source_amount = after_swap_source_amount.sub(before_swap_source_amount);
        let source_amount_with_fee = actual_source_amount.sub(trade_fee_after_protocol_fee);

        let invariant = swap_source_amount.mul(swap_dst_amount);
        let new_swap_destination_amount = invariant.div_ceil(swap_source_amount.add(source_amount_with_fee));
        swap_dst_amount - new_swap_destination_amount
    }
}

impl<'a, 'info> BaseMarketPool<'a, 'info> for MeteoraAMMMarketPool<'a, 'info> {

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
        self.cal_out_amount(in_y as u128,
                            self.amount_y as u128,
                            self.amount_x as u128,
                            self.vault_y_withdrawable_amount as u128,
                            self.pool_y_lp_amount as u128,
                            self.vault_y_lp_supply as u128) as u64
    }

    #[cfg(feature = "debug-out")]
    fn out_y(&self, in_x: u64) -> u64 {
        self.cal_out_amount(in_x as u128,
                            self.amount_x as u128,
                            self.amount_y as u128,
                            self.vault_x_withdrawable_amount as u128,
                            self.pool_x_lp_amount as u128,
                            self.vault_x_lp_supply as u128) as u64
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
        let mut swap_data = Vec::from_hex("f8c69e91e17587c8").unwrap();
        let min_ammount_out:u64 = 0;
        swap_data.extend(borsh::to_vec(&amount_in).unwrap());
        swap_data.extend(borsh::to_vec(&min_ammount_out).unwrap());
        let account_meta = vec![
            AccountMeta::new(self.pool_state.key(), false),
            AccountMeta::new(match y2x {
                true => {
                    self.user_token_account_y.key()
                },
                false => {
                    self.user_token_account_x.key()
                }
            }, false),
            AccountMeta::new(match y2x {
                true => {
                    self.user_token_account_x.key()
                },
                false => {
                    self.user_token_account_y.key()
                }
            }, false),
            AccountMeta::new(self.x_vault.key(), false),
            AccountMeta::new(self.y_vault.key(), false),
            AccountMeta::new(self.x_token_vault.key(), false),
            AccountMeta::new(self.y_token_vault.key(), false),
            AccountMeta::new(self.x_vault_lp_mint.key(), false),
            AccountMeta::new(self.y_vault_lp_mint.key(), false),
            AccountMeta::new(self.x_vault_lp.key(), false),
            AccountMeta::new(self.y_vault_lp.key(), false),
            AccountMeta::new(match y2x {
                true => {
                    self.protocol_fee_account_y.key()
                },
                false => {
                    self.protocol_fee_account_x.key()
                }
            }, false),
            AccountMeta::new(self.user.key(), true),
            AccountMeta::new_readonly(self.vault_program.key(), false),
            AccountMeta::new_readonly(self.token_program.key(), false),
        ];
        let accounts_info = vec![
            self.pool_state.to_account_info(),
            match y2x {
                true => {
                    self.user_token_account_y.to_account_info()
                },
                false => {
                    self.user_token_account_x.to_account_info()
                }
            },
            match y2x {
                true => {
                    self.user_token_account_x.to_account_info()
                },
                false => {
                    self.user_token_account_y.to_account_info()
                }
            },
            self.x_vault.to_account_info(),
            self.y_vault.to_account_info(),
            self.x_token_vault.to_account_info(),
            self.y_token_vault.to_account_info(),
            self.x_vault_lp_mint.to_account_info(),
            self.y_vault_lp_mint.to_account_info(),
            self.x_vault_lp.to_account_info(),
            self.y_vault_lp.to_account_info(),
            match y2x {
                true => {
                    self.protocol_fee_account_y.to_account_info()
                },
                false => {
                    self.protocol_fee_account_x.to_account_info()
                }
            },
            self.user.to_account_info(),
            self.vault_program.to_account_info(),
            self.token_program.to_account_info(),
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

impl<'a, 'info> CreateMarket<'a, 'info> for MeteoraAMMMarketPool<'a, 'info> {
    fn create_market(
            base: &'a crate::model::base_model::BaseModel<'a, 'info>,
            min_x_user_account: &'a Option<UncheckedAccount<'info>>,
            mint_x_mint_account: &'a Option<UncheckedAccount<'info>>,
            accounts_iter: Rc<RefCell<AccountsIter<'a, 'info>>>,
            reverse: bool,
        ) -> Box<dyn BaseMarketPool<'a, 'info> + 'a> {
        let account_vec = accounts_iter.borrow_mut().take(13);
        
        let mut meteora = Box::new(MeteoraAMMMarketPool {
            pool_program: account_vec[0].as_ref().unwrap(),
            pool_state: account_vec[1].as_ref().unwrap(),
            x_vault: account_vec[2].as_ref().unwrap(),
            y_vault: account_vec[3].as_ref().unwrap(),
            x_token_vault: account_vec[4].as_ref().unwrap(),
            y_token_vault: account_vec[5].as_ref().unwrap(),
            x_vault_lp_mint: account_vec[6].as_ref().unwrap(),
            y_vault_lp_mint: account_vec[7].as_ref().unwrap(),
            x_vault_lp: account_vec[8].as_ref().unwrap(),
            y_vault_lp: account_vec[9].as_ref().unwrap(),
            protocol_fee_account_x: account_vec[10].as_ref().unwrap(),
            protocol_fee_account_y: account_vec[11].as_ref().unwrap(),
            vault_program: account_vec[12].as_ref().unwrap(),
            user_token_account_x: match reverse {
                true => base.user_token_base.as_ref(),
                false => min_x_user_account.as_ref().unwrap(),
                },
            user_token_account_y: match reverse {
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
            vault_x_withdrawable_amount: 0,
            vault_y_withdrawable_amount: 0,
            vault_x_lp_supply: 0,
            vault_y_lp_supply: 0,
            pool_x_lp_amount: 0,
            pool_y_lp_amount: 0,
            protocol_fee_numerator: 0,
            protocol_fee_denominator: 0,
        });
        meteora.init();
        if reverse {
            return Box::new(
                MockReverseMarketPool {
                    market: meteora,
                }
            )
        }
        meteora
    }
}

