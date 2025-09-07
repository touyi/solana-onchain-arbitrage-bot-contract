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
use crate::model::errors::*;
use crate::utils::utils::*;
use crate::common::accounts_iter::*;
use hex::FromHex;
use super::mock_reverse_market::MockReverseMarketPool;

pub struct PumpSwapMarketPool<'a, 'info> {
    pub token_program: &'a AccountInfo<'info>,
    pub pool_program: &'a AccountInfo<'info>,
    pub sys_program: &'a AccountInfo<'info>,
    pub associated_token_program: &'a AccountInfo<'info>,

    pub coin_creator_vault_ata: &'a AccountInfo<'info>,
    pub coin_creator_auth: &'a AccountInfo<'info>,
    
    pub pool_info: &'a AccountInfo<'info>,
    pub global_config: &'a AccountInfo<'info>,
    pub mint_x_mint: &'a AccountInfo<'info>,
    pub mint_y_mint: &'a AccountInfo<'info>,
    pub mint_x_user_token_account: &'a AccountInfo<'info>,
    pub mint_y_user_token_account: &'a AccountInfo<'info>,
    pub mint_x_pool_token_account: &'a AccountInfo<'info>,
    pub mint_y_pool_token_account: &'a AccountInfo<'info>,
    pub protocol_fee_reciver: &'a AccountInfo<'info>,
    pub protocol_fee_reciver_token_account: &'a AccountInfo<'info>,
    pub authorithy: &'a AccountInfo<'info>,
    pub user : &'a AccountInfo<'info>,

    pub fee_denominator: u64,
    pub base_fee: u64,
    pub protocol_fee: u64,
    pub creator_fee: u64,
    pub amount_x: u64,
    pub amount_y: u64,
}

impl<'a, 'info> PumpSwapMarketPool<'a, 'info> {
    pub fn init(&mut self) {
        self.fee_denominator = 10000u64;
        let config_data = self.global_config.data.borrow();

        let mut base_fee_buffer = [0u8; 8];
        base_fee_buffer.copy_from_slice(&config_data[8 + 32..8 + 32 + 8]);
        self.base_fee = u64::from_le_bytes(base_fee_buffer);

        let mut protocol_fee_buffer = [0u8; 8];
        protocol_fee_buffer.copy_from_slice(&config_data[8 + 32 + 8..8 + 32 + 8 + 8]);
        self.protocol_fee = u64::from_le_bytes(protocol_fee_buffer);


        let mut creator_fee_buffer = [0u8; 8];
        creator_fee_buffer.copy_from_slice(&config_data[8 + 32 + 8 + 8 + 1 + 8 * 32..8 + 32 + 8 + 8 + 1 + 8 * 32 + 8]);
        self.creator_fee = u64::from_le_bytes(creator_fee_buffer);


        self.amount_x = unpack_token_account_ammount(self.mint_x_pool_token_account).unwrap();
        self.amount_y = unpack_token_account_ammount(self.mint_y_pool_token_account).unwrap();

        // msg!("pump: amx:{} amy:{} fee:{}", self.amount_x, self.amount_y, self.fee_numerator());

    }

    fn inner_out_x(&self, in_y: u64) -> u64 {
        let fee_denominator = self.fee_denominator() as u128;
        let fee_numerator = self.fee_numerator() as u128;
        
        let in_y = (in_y as u128).mul(fee_denominator).div(fee_numerator + fee_denominator);

        let x = self.amount_x() as u128;
        let y = self.amount_y() as u128;
        let denominator = y.add(in_y);
        let all_x = x.mul(in_y).div(denominator);
        all_x as u64
    }
}

impl<'a, 'info> BaseMarketPool<'a, 'info> for PumpSwapMarketPool<'a, 'info> {

    fn get_real_time_user_token_amount_x(&self) -> u64 {
        unpack_token_account_ammount(self.mint_x_user_token_account).unwrap()
    }

    fn get_real_time_user_token_amount_y(&self) -> u64 {
        unpack_token_account_ammount(self.mint_y_user_token_account).unwrap()
    }

    fn valid(&self) -> bool {
        self.amount_x > 0 && self.amount_y > 0
    }
    #[cfg(feature = "debug-out")]
    fn out_x(&self, in_y: u64) -> u64 {
        self.inner_out_x(in_y)
    }

    #[cfg(feature = "debug-out")]
    fn out_y(&self, in_x: u64) -> u64 {
        let in_x = in_x as u128;
        let x = self.amount_x() as u128;
        let y = self.amount_y() as u128;
        let denominator = x.add(in_x);
        let all_y = y.mul(in_x).div(denominator);
        let fee_denominator = self.fee_denominator() as u128;

        let base_fee_amount = all_y.mul(self.base_fee as u128).add(fee_denominator.sub(1)).div(fee_denominator);
        let proto_fee_amount = all_y.mul(self.protocol_fee as u128).add(fee_denominator.sub(1)).div(fee_denominator);
        let creator_fee_amount = all_y.mul(self.creator_fee as u128).add(fee_denominator.sub(1)).div(fee_denominator);

        all_y.sub(base_fee_amount).sub(proto_fee_amount).sub(creator_fee_amount) as u64
    }

    fn x_mint(&self) -> &Pubkey {
        self.mint_x_mint.key
    }
    fn y_mint(&self) -> &Pubkey {
        self.mint_y_mint.key
    }
    fn fee_denominator(&self) -> u64 {
        self.fee_denominator
    }
    fn fee_numerator(&self) -> u64 {
        self.base_fee + self.protocol_fee + self.creator_fee
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
        let swap_data = match y2x {
            true => {
                // buy y -> x
                match Vec::from_hex("66063d1201daebea") {
                    Ok(mut v) => {
                        let amount_out = self.inner_out_x(amount_in);
                        v.append(borsh::to_vec(&amount_out).unwrap().as_mut());
                        v.append(borsh::to_vec(&(amount_in.mul(2))).unwrap().as_mut());
                        v
                    },
                    Err(e) => {
                        msg!("Error decoding hex: {:?}", e);
                        return Err(error!(MyErrorCode::InvalidTokenAccount));
                    }
                }
            },
            false => {
                // sell x -> y
                match Vec::from_hex("33e685a4017f83ad") {
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
                }
            }
        };

        let account_meta = vec![
            AccountMeta::new_readonly(self.pool_info.key(), false),
            AccountMeta::new(self.user.key(), true),
            AccountMeta::new_readonly(self.global_config.key(), false),
            AccountMeta::new_readonly(self.mint_x_mint.key(), false),
            AccountMeta::new_readonly(self.mint_y_mint.key(), false),
            AccountMeta::new(self.mint_x_user_token_account.key(), false),
            AccountMeta::new(self.mint_y_user_token_account.key(), false),
            AccountMeta::new(self.mint_x_pool_token_account.key(), false),
            AccountMeta::new(self.mint_y_pool_token_account.key(), false),
            AccountMeta::new_readonly(self.protocol_fee_reciver.key(), false),
            AccountMeta::new(self.protocol_fee_reciver_token_account.key(), false),
            AccountMeta::new_readonly(self.token_program.key(), false),
            AccountMeta::new_readonly(self.token_program.key(), false),
            AccountMeta::new_readonly(self.sys_program.key(), false),
            AccountMeta::new_readonly(self.associated_token_program.key(), false),
            AccountMeta::new_readonly(self.authorithy.key(), false),
            AccountMeta::new_readonly(self.pool_program.key(), false),
            AccountMeta::new(self.coin_creator_vault_ata.key(), false),
            AccountMeta::new_readonly(self.coin_creator_auth.key(), false),
        ];
        let accounts_info = vec![
            self.pool_info.to_account_info(),
            self.user.to_account_info(),
            self.global_config.to_account_info(),
            self.mint_x_mint.to_account_info(),
            self.mint_y_mint.to_account_info(),
            self.mint_x_user_token_account.to_account_info(),
            self.mint_y_user_token_account.to_account_info(),
            self.mint_x_pool_token_account.to_account_info(),
            self.mint_y_pool_token_account.to_account_info(),
            self.protocol_fee_reciver.to_account_info(),
            self.protocol_fee_reciver_token_account.to_account_info(),
            self.token_program.to_account_info(),
            self.token_program.to_account_info(),
            self.sys_program.to_account_info(),
            self.associated_token_program.to_account_info(),
            self.authorithy.to_account_info(),
            self.pool_program.to_account_info(),
            self.coin_creator_vault_ata.to_account_info(),
            self.coin_creator_auth.to_account_info(),
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

impl<'a, 'info> CreateMarket<'a, 'info> for PumpSwapMarketPool<'a, 'info> {
    fn create_market(
            base: &'a crate::model::base_model::BaseModel<'a, 'info>,
            min_x_user_account: &'a Option<UncheckedAccount<'info>>,
            mint_x_mint_account: &'a Option<UncheckedAccount<'info>>,
            accounts_iter: Rc<RefCell<AccountsIter<'a, 'info>>>,
            reverse: bool,
        ) -> Box<dyn BaseMarketPool<'a, 'info> + 'a> {
        let account_vec = accounts_iter.borrow_mut().take(11);
        let mut pumpswap = Box::new(PumpSwapMarketPool {
                pool_program: account_vec[0].as_ref().unwrap(),
                pool_info: account_vec[1].as_ref().unwrap(),
                global_config: account_vec[2].as_ref().unwrap(),
                mint_x_pool_token_account: account_vec[3].as_ref().unwrap(),
                mint_y_pool_token_account: account_vec[4].as_ref().unwrap(),
                protocol_fee_reciver: account_vec[5].as_ref().unwrap(),
                protocol_fee_reciver_token_account: account_vec[6].as_ref().unwrap(),
                associated_token_program: account_vec[7].as_ref().unwrap(),
                authorithy: account_vec[8].as_ref().unwrap(),
                coin_creator_vault_ata: account_vec[9].as_ref().unwrap(),
                coin_creator_auth: account_vec[10].as_ref().unwrap(),

                user: base.user,
                token_program: base.token_program,
                sys_program: base.sys_program,

                mint_x_mint: match reverse {
                    true => base.token_base_mint,
                    false => mint_x_mint_account.as_ref().unwrap(),
                },
                mint_y_mint: match reverse {
                    true => mint_x_mint_account.as_ref().unwrap(),
                    false => base.token_base_mint,
                },
                mint_x_user_token_account: match reverse {
                    true => base.user_token_base.as_ref(),
                    false => min_x_user_account.as_ref().unwrap(),
                },
                mint_y_user_token_account: match reverse {
                    true => min_x_user_account.as_ref().unwrap(),
                    false => base.user_token_base.as_ref(),
                },

                amount_x: 0,
                amount_y: 0,
                base_fee: 0,
                protocol_fee: 0,
                fee_denominator: 0,
                creator_fee: 0,
        });
        pumpswap.init();
        if reverse {
            return Box::new(
                MockReverseMarketPool {
                    market: pumpswap,
                }
            )
        }
        pumpswap
    }
}

