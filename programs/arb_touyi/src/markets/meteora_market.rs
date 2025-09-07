use std::cell::RefCell;
use std::ops::{Add, Mul, Sub};
use std::rc::Rc;
use std::{ops::Div, vec};
// use base64::{engine::general_purpose, Engine};
use crate::base_market::*;
use crate::utils::utils::unpack_token_account_ammount;
use anchor_lang::prelude::*;
use crate::model::errors::*;
use crate::model::base_model::*;
use crate::markets::mock_reverse_market::*;
use hex::FromHex;
use crate::common::accounts_iter::*;
use anchor_lang::solana_program::{
    program::invoke,
    instruction::Instruction,
    log::sol_log_compute_units,
};
// const MAX_FEE_RATE: u64 = 100000000;
const FEE_PRECISION: u64 = 1000000000;
const SCALE_OFFSET: u32 = 64;
pub struct MeteorCMMMarketPool<'a, 'info> {
    pub lb_pair: &'a AccountInfo<'info>,
    pub bin_array_bitmap_extension: &'a Option<UncheckedAccount<'info>>,
    pub reserve_x: &'a AccountInfo<'info>,
    pub reserve_y: &'a AccountInfo<'info>,
    pub user_token_account_x: &'a AccountInfo<'info>,
    pub user_token_account_y: &'a AccountInfo<'info>,
    pub token_mint_x: &'a AccountInfo<'info>,
    pub token_mint_y: &'a AccountInfo<'info>,
    pub oracle: &'a AccountInfo<'info>,
    pub token_program: &'a AccountInfo<'info>,
    pub event_authorithy: &'a AccountInfo<'info>,
    pub dlmm_program: &'a AccountInfo<'info>,
    pub bin_arrays_vec: Vec<&'a AccountInfo<'info>>,
    pub user: &'a AccountInfo<'info>,

    pub decimal_diff: i8,
    pub price: f64,
    pub fee: u64,
    pub amount_x: u64,
    pub amount_y: u64,

    pub bin_price: u128, // TODO(fill this)


}
const MAX_BIN_ARRAY_SIZE: i32 = 70;
impl<'a, 'info> MeteorCMMMarketPool<'a, 'info> {

    fn get_bin_array_bound(&self, bin_array_index: i64) -> (i32, i32) {
        let lower_bin_id = bin_array_index as i32 * MAX_BIN_ARRAY_SIZE;
        let upper_bin_id = lower_bin_id + MAX_BIN_ARRAY_SIZE - 1;
        return (lower_bin_id, upper_bin_id);
    }
    fn cal_bin_index(&self, activate_id: i32, lower_bin_id: i32, upper_bin_id: i32) -> i32 {
        if activate_id > 0 {
            return activate_id - lower_bin_id;
        } else {
            let delta = upper_bin_id - activate_id;
            return MAX_BIN_ARRAY_SIZE - delta - 1;
        }
    }

    pub fn init(&mut self) {
        let lb_data = self.lb_pair.data.borrow();

        let mut activeid_buf = [0u8; 4];
        activeid_buf[0..4].copy_from_slice(&lb_data[8 + 32 + 32 + 1 + 2 + 1..8 + 32 + 32 + 1 + 2 + 1 + 4]);
        let activeid = i32::from_le_bytes(activeid_buf);

        let mut bin_step_buf = [0u8; 2];
        bin_step_buf[0..2].copy_from_slice(&lb_data[8 + 32 + 32 + 1 + 2 + 1 + 4..8 + 32 + 32 + 1 + 2 + 1 + 4 + 2]);
        let bin_step = u16::from_le_bytes(bin_step_buf);

        let mut base_factor_buf = [0u8; 2];
        base_factor_buf[0..2].copy_from_slice(&lb_data[8..8 + 2]);
        let base_factor = u16::from_le_bytes(base_factor_buf);

        let mut variable_fee_control_buf = [0u8; 4];
        variable_fee_control_buf[0..4].copy_from_slice(&lb_data[8 + 8..8 + 8 + 4]);
        let variable_fee_control = u32::from_le_bytes(variable_fee_control_buf);
        
        let mut volatility_accumulator_buf = [0u8; 4];
        volatility_accumulator_buf[0..4].copy_from_slice(&lb_data[8 + 32..8 + 32 + 4]);
        let volatility_accumulator = u32::from_le_bytes(volatility_accumulator_buf);
        // FEE
        self.fee = self.get_total_fee(bin_step, variable_fee_control, volatility_accumulator, base_factor) as u64;
        // NOTE: not calculate decimal_diff
        self.price = (10000 as f64).div((10000 + bin_step) as f64).powi(activeid);
        // amount
        for bins_array in self.bin_arrays_vec.iter() {
            // FIXME(touyi): find not empy bins
            if bins_array.data_is_empty() {
                continue;
            }
            let bins_array_data = bins_array.data.borrow();
            let mut index_buf = [0u8; 8];
            index_buf[0..8].copy_from_slice(&bins_array_data[8..8 + 8]);
            let index = i64::from_le_bytes(index_buf);

            let bound = self.get_bin_array_bound(index);
            if activeid < bound.0 ||  bound.1 < activeid {
                continue;
            }
            let bin_index = self.cal_bin_index(activeid, bound.0, bound.1);
            let bin_start_bytes = 8 + 16 + 32;
            let per_bin_size = 144;
            let bin_offset = (per_bin_size * bin_index + bin_start_bytes) as usize;

            let mut amount_x_buf = [0u8; 8];
            amount_x_buf[0..8].copy_from_slice(&bins_array_data[bin_offset..bin_offset + 8]);
            self.amount_x = u64::from_le_bytes(amount_x_buf);

            let mut amount_y_buf = [0u8; 8];
            amount_y_buf[0..8].copy_from_slice(&bins_array_data[bin_offset + 8..bin_offset + 8 + 8]);
            self.amount_y = u64::from_le_bytes(amount_y_buf);

            let mut bin_price_buf = [0u8; 16];
            bin_price_buf[0..16].copy_from_slice(&bins_array_data[bin_offset + 8 + 8..bin_offset + 8 + 8 + 16]);
            self.bin_price = u128::from_le_bytes(bin_price_buf);
        }
    }

    fn get_base_fee(&self, bin_step: u16, base_factor: u16) -> u128 {
        (base_factor as u128) * (bin_step as u128) * (10 as u128)
        // BN::from(base_factor).mul(&BN::from(bin_step)).mul(&BN::from(10))
    }
    
    fn get_variable_fee(&self, bin_step: u16, variable_fee_control: u32, volatility_accumulator: u32) -> u128 {
        if variable_fee_control > 0 {
            let squart = (volatility_accumulator as u128 * bin_step as u128).pow(2);
            // let squart = BN::from(volatility_accumulator).mul(&BN::from(bin_step)).pow(&BN::from(2));
            let v_fee = (variable_fee_control as u128) * squart;
            // let v_fee = BN::from(variable_fee_control).mul(&square);
            return (v_fee + 99999999999u128) / 100000000000u128;
        }
        return 0;
    }

    fn get_total_fee(&self, bin_step: u16, variable_fee_control: u32, volatility_accumulator: u32, base_factor: u16) -> u128 {
        self.get_base_fee(bin_step, base_factor) + self.get_variable_fee(bin_step, variable_fee_control, volatility_accumulator)
    }
}

impl<'a, 'info> BaseMarketPool<'a, 'info> for MeteorCMMMarketPool<'a, 'info> {

    fn get_real_time_user_token_amount_x(&self) -> u64 {
        unpack_token_account_ammount(self.user_token_account_x).unwrap()
    }

    fn get_real_time_user_token_amount_y(&self) -> u64 {
        unpack_token_account_ammount(self.user_token_account_y).unwrap()
    }

    fn valid(&self) -> bool {
        self.amount_x > 0 || self.amount_y > 0
    }

    #[cfg(feature = "debug-out")]
    fn out_x(&self, in_y: u64) -> u64 {
        let in_y = in_y as u128;
        let fee_n = self.fee_numerator() as u128;
        let fee_d = self.fee_denominator() as u128;
        let fee = in_y.mul(fee_n).add(fee_d.sub(1)).div(fee_d);
        let remain_y = in_y.sub(fee);
        let scale = 1u128 << SCALE_OFFSET;
        remain_y.mul(scale).div(self.bin_price) as u64
    }

    #[cfg(feature = "debug-out")]
    fn out_y(&self, in_x: u64) -> u64 {
        let in_x = in_x as u128;
        let fee_n = self.fee_numerator() as u128;
        let fee_d = self.fee_denominator() as u128;
        let fee = in_x.mul(fee_n).add(fee_d.sub(1)).div(fee_d);
        let remain_x = in_x.sub(fee);
        let denominator = 1u128 << SCALE_OFFSET;
        remain_x.mul(self.bin_price).div(denominator) as u64
    }

    fn x_mint(&self) -> &Pubkey {
        &self.token_mint_x.key
    }

    fn y_mint(&self) -> &Pubkey {
        &self.token_mint_y.key
    }

    fn swap(&self, y2x: bool, amount_in: u64) -> Result<()> {
        let swap_data = match Vec::from_hex("f8c69e91e17587c8") {
            Ok(mut v) => {
                let min_ammount_out:u64 = 0;
                v.append(borsh::to_vec(&amount_in).unwrap().as_mut());
                v.append(borsh::to_vec(&min_ammount_out).unwrap().as_mut());
                v
            },
            Err(e) => {
                msg!("Error decoding hex: {:?}", e);
                return Err(error!(MyErrorCode::InvalidTokenAccount));
            }
        };
        let mut accounts_meta = vec![
            AccountMeta::new(self.lb_pair.key(), false),
            AccountMeta::new_readonly(match self.bin_array_bitmap_extension {
                Some(ref account) => account.key(),
                None => self.dlmm_program.key(),
            }, false),
            AccountMeta::new(self.reserve_x.key(), false),
            AccountMeta::new(self.reserve_y.key(), false),
            AccountMeta::new(match y2x {
                true => self.user_token_account_y.key(),
                false => self.user_token_account_x.key(),
            }, false),
            AccountMeta::new(match y2x {
                true => self.user_token_account_x.key(),
                false => self.user_token_account_y.key(),
            }, false),
            AccountMeta::new_readonly(self.token_mint_x.key(), false),
            AccountMeta::new_readonly(self.token_mint_y.key(), false),
            AccountMeta::new(self.oracle.key(), false),
            AccountMeta::new(self.dlmm_program.key(), false),
            AccountMeta::new(self.user.key(), true),
            AccountMeta::new_readonly(self.token_program.key(), false),
            AccountMeta::new_readonly(self.token_program.key(), false),
            AccountMeta::new_readonly(self.event_authorithy.key(), false),
            AccountMeta::new_readonly(self.dlmm_program.key(), false),
          ];
        let mut accounts_info = vec![
            self.lb_pair.to_account_info(),
            match self.bin_array_bitmap_extension  {
                Some(ref account) => account.to_account_info(),
                None => self.dlmm_program.to_account_info(),
            },
            self.reserve_x.to_account_info(),
            self.reserve_y.to_account_info(),
            match y2x {
                true => self.user_token_account_y.to_account_info(),
                false => self.user_token_account_x.to_account_info(),
            },
            match y2x {
                true => self.user_token_account_x.to_account_info(),
                false => self.user_token_account_y.to_account_info(),
            },
            self.token_mint_x.to_account_info(),
            self.token_mint_y.to_account_info(),
            self.oracle.to_account_info(),
            self.dlmm_program.to_account_info(),
            self.user.to_account_info(),
            self.token_program.to_account_info(),
            self.token_program.to_account_info(),
            self.event_authorithy.to_account_info(),
            self.dlmm_program.to_account_info(),
        ];
        for account in self.bin_arrays_vec.iter() {
            if account.data_is_empty() {
                continue;
            }
            accounts_meta.push(AccountMeta::new(account.key(), false));
            accounts_info.push(account.to_account_info());
        }
        let swap_ix = Instruction {
          program_id: self.dlmm_program.key.clone(),
          accounts: accounts_meta,
          data: swap_data,
        };
        invoke(&swap_ix, &accounts_info)?;
        Ok(())
    }
    fn fee_denominator(&self) -> u64 {
        FEE_PRECISION
    }
    fn fee_numerator(&self) -> u64 {
        self.fee
    }
    fn amount_x(&self) -> u64 {
        self.amount_x
    }
    fn amount_y(&self) -> u64 {
        self.amount_y
    }
    fn market_type(&self) -> MarketType {
        MarketType::NormalCMM
    }
    fn price(&self) -> f64 {
        // current_price = current_price * 10f64.powf(decimal_diff as f64) / PRECISION as f64;
        /*
        export function getPriceOfBinByBinId(binId: number, binStep: number): Decimal {
        const binStepNum = new Decimal(binStep).div(new Decimal(10000));
        return new Decimal(1).add(new Decimal(binStepNum)).pow(new Decimal(binId));
        }
        */
        self.price
    }
}


impl<'a, 'info> CreateMarket<'a, 'info> for MeteorCMMMarketPool<'a, 'info>
{
    fn create_market(
        base: &'a BaseModel<'a, 'info>,
        min_x_user_accounts: &'a Option<UncheckedAccount<'info>>,
        mint_x_mint_accounts: &'a Option<UncheckedAccount<'info>>,
        accounts_iter: Rc<RefCell<AccountsIter<'a, 'info>>>,
        reverse: bool,
    ) -> Box<dyn BaseMarketPool<'a, 'info> + 'a> {
        let accounts = accounts_iter.borrow_mut().take(10);
        let mut meteora = Box::new(MeteorCMMMarketPool {
            lb_pair: accounts[0].as_ref().unwrap(),
            bin_array_bitmap_extension: accounts[1],
            reserve_x: accounts[2].as_ref().unwrap(),
            reserve_y: accounts[3].as_ref().unwrap(),
            oracle: accounts[4].as_ref().unwrap(),
            event_authorithy: accounts[5].as_ref().unwrap(),
            dlmm_program: accounts[6].as_ref().unwrap(),
            bin_arrays_vec: vec![accounts[7].as_ref().unwrap(), accounts[8].as_ref().unwrap(), accounts[9].as_ref().unwrap()],

            token_program: base.token_program.as_ref(),
            user: base.user.as_ref(),
            token_mint_x: match reverse {
                true => base.token_base_mint.as_ref(),
                false => mint_x_mint_accounts.as_ref().unwrap(),
            },
            token_mint_y: match reverse {
                true => mint_x_mint_accounts.as_ref().unwrap(),
                false => base.token_base_mint.as_ref(),
            },
            user_token_account_x: match reverse {
                true => base.user_token_base.as_ref(),
                false => min_x_user_accounts.as_ref().unwrap(),
            },
            user_token_account_y: match reverse {
                true => min_x_user_accounts.as_ref().unwrap(),
                false => base.user_token_base.as_ref(),
            },

            decimal_diff: 0,
            price: 0.0,
            fee: 0,
            amount_x: 0,
            amount_y: 0,
            bin_price: 0,
        });
        meteora.init();

        if reverse {
            return Box::new(MockReverseMarketPool {
                market: meteora,
            });
        }
        return meteora;

    }
}
