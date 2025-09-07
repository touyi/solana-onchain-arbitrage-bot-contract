pub mod base_market;
pub mod markets;
pub mod model;
pub mod utils;
pub mod processor;
pub mod common;

use crate::model::errors::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::log::sol_log_compute_units;
use base_market::*;
use model::input_model::*;
use model::base_model::*;
use markets::raydium_clmm_market::*;
declare_id!("DxeQQ7PQ94j26ism5ivTqNHAkteFNmgRpqYx7XQFqs9Z");

#[program]
pub mod arb_touyi {
    use markets::raydium_market;

    use crate::utils::utils::unpack_token_account_ammount;

    use super::*;
    pub fn arb_process_64_account(
        ctx: Context<CommonAccountsInfo64>,
        max_in: u64,
        min_profit: u64,
        market_type: Vec<u8>,
        market_flag: Vec<u8>
    ) -> Result<()> {
        msg!(
            "max_in {}, min_profit {}， market_type {:?}, market_flag {:?}",
            max_in,
            min_profit,
            market_type,
            market_flag
        );
        let base_model = BaseModel {
            user: ctx.accounts.user.as_ref(),
            token_program: ctx.accounts.token_program.as_ref(),
            token_base_mint: ctx.accounts.token_base_mint.as_ref(),
            user_token_base: ctx.accounts.user_token_base.as_ref(),
            sys_program: ctx.accounts.sys_program.as_ref(),
            recipient: ctx.accounts.recipient.as_ref(),
            associated_token_program: ctx.accounts.associated_token_program.as_ref(),
        };
        let mint_x_user_accounts = vec![&ctx.accounts.token_pair_0_user_token_account_x];
        let mint_x_mint_accounts = vec![&ctx.accounts.token_pair_0_mint_x];
        let account_list = vec![
            &ctx.accounts.account_0,
            &ctx.accounts.account_1,
            &ctx.accounts.account_2,
            &ctx.accounts.account_3,
            &ctx.accounts.account_4,
            &ctx.accounts.account_5,
            &ctx.accounts.account_6,
            &ctx.accounts.account_7,
            &ctx.accounts.account_8,
            &ctx.accounts.account_9,
            &ctx.accounts.account_10,
            &ctx.accounts.account_11,
            &ctx.accounts.account_12,
            &ctx.accounts.account_13,
            &ctx.accounts.account_14,
            &ctx.accounts.account_15,
            &ctx.accounts.account_16,
            &ctx.accounts.account_17,
            &ctx.accounts.account_18,
            &ctx.accounts.account_19,
            &ctx.accounts.account_20,
            &ctx.accounts.account_21,
            &ctx.accounts.account_22,
            &ctx.accounts.account_23,
            &ctx.accounts.account_24,
            &ctx.accounts.account_25,
            &ctx.accounts.account_26,
            &ctx.accounts.account_27,
            &ctx.accounts.account_28,
            &ctx.accounts.account_29,
            &ctx.accounts.account_30,
            &ctx.accounts.account_31,
            &ctx.accounts.account_32,
            &ctx.accounts.account_33,
            &ctx.accounts.account_34,
            &ctx.accounts.account_35,
            &ctx.accounts.account_36,
            &ctx.accounts.account_37,
            &ctx.accounts.account_38,
            &ctx.accounts.account_39,
            &ctx.accounts.account_40,
            &ctx.accounts.account_41,
            &ctx.accounts.account_42,
            &ctx.accounts.account_43,
            &ctx.accounts.account_44,
            &ctx.accounts.account_45,
            &ctx.accounts.account_46,
            &ctx.accounts.account_47,
            &ctx.accounts.account_48,
            &ctx.accounts.account_49,
            &ctx.accounts.account_50,
            &ctx.accounts.account_51,
            &ctx.accounts.account_52,
            &ctx.accounts.account_53,
            &ctx.accounts.account_54
        ];

        processor::main_process(
            &base_model,
            &mint_x_user_accounts,
            &mint_x_mint_accounts,
            &account_list,
            &market_type,
            &market_flag,
            max_in,
            min_profit
        )?;

        Ok(())
    }
    pub fn arb_process_32_account(
        ctx: Context<CommonAccountsInfo32>,
        max_in: u64,
        min_profit: u64,
        market_type: Vec<u8>,
        market_flag: Vec<u8>
    ) -> Result<()> {
        msg!(
            "max_in {}, min_profit {}， market_type {:?}, market_flag {:?}",
            max_in,
            min_profit,
            market_type,
            market_flag
        );
        let base_model = BaseModel {
            user: ctx.accounts.user.as_ref(),
            token_program: ctx.accounts.token_program.as_ref(),
            token_base_mint: ctx.accounts.token_base_mint.as_ref(),
            user_token_base: ctx.accounts.user_token_base.as_ref(),
            sys_program: ctx.accounts.sys_program.as_ref(),
            recipient: ctx.accounts.recipient.as_ref(),
            associated_token_program: ctx.accounts.associated_token_program.as_ref(),
        };
        let mint_x_user_accounts = vec![&ctx.accounts.token_pair_0_user_token_account_x];
        let mint_x_mint_accounts = vec![&ctx.accounts.token_pair_0_mint_x];
        let account_list = vec![
            &ctx.accounts.account_0,
            &ctx.accounts.account_1,
            &ctx.accounts.account_2,
            &ctx.accounts.account_3,
            &ctx.accounts.account_4,
            &ctx.accounts.account_5,
            &ctx.accounts.account_6,
            &ctx.accounts.account_7,
            &ctx.accounts.account_8,
            &ctx.accounts.account_9,
            &ctx.accounts.account_10,
            &ctx.accounts.account_11,
            &ctx.accounts.account_12,
            &ctx.accounts.account_13,
            &ctx.accounts.account_14,
            &ctx.accounts.account_15,
            &ctx.accounts.account_16,
            &ctx.accounts.account_17,
            &ctx.accounts.account_18,
            &ctx.accounts.account_19,
            &ctx.accounts.account_20,
            &ctx.accounts.account_21,
            &ctx.accounts.account_22,
            &ctx.accounts.account_23,
            &ctx.accounts.account_24,
            &ctx.accounts.account_25,
            &ctx.accounts.account_26,
            &ctx.accounts.account_27,
            &ctx.accounts.account_28,
        ];

        processor::main_process(
            &base_model,
            &mint_x_user_accounts,
            &mint_x_mint_accounts,
            &account_list,
            &market_type,
            &market_flag,
            max_in,
            min_profit
        )?;

        Ok(())
    }

    
    pub fn test_raydium_clmm(
        ctx: Context<CommonAccountsInfo32>,
        amount_in: u64,
        market_type: u8,
        reverse: bool,
        y2x: bool,
        use_limit: bool,
        always_fail: bool
    ) -> Result<()> {
        let base_model = BaseModel {
            user: ctx.accounts.user.as_ref(),
            token_program: ctx.accounts.token_program.as_ref(),
            token_base_mint: ctx.accounts.token_base_mint.as_ref(),
            user_token_base: ctx.accounts.user_token_base.as_ref(),
            sys_program: ctx.accounts.sys_program.as_ref(),
            recipient: ctx.accounts.recipient.as_ref(),
            associated_token_program: ctx.accounts.associated_token_program.as_ref(),
        };
        let account_list = vec![
            &ctx.accounts.account_0,
            &ctx.accounts.account_1,
            &ctx.accounts.account_2,
            &ctx.accounts.account_3,
            &ctx.accounts.account_4,
            &ctx.accounts.account_5,
            &ctx.accounts.account_6,
            &ctx.accounts.account_7,
            &ctx.accounts.account_8,
            &ctx.accounts.account_9,
            &ctx.accounts.account_10,
            &ctx.accounts.account_11,
            &ctx.accounts.account_12,
            &ctx.accounts.account_13,
            &ctx.accounts.account_14,
            &ctx.accounts.account_15,
            &ctx.accounts.account_16
        ];
        let mint_x_user_accounts = vec![&ctx.accounts.token_pair_0_user_token_account_x];
        let mint_x_mint_accounts = vec![&ctx.accounts.token_pair_0_mint_x];
        let market_types: Vec<u8> = vec![market_type];
        let market_flags: Vec<u8> = vec![match reverse {
            true => 1u8 << 7,
            false => 0,
        }];

        let market_list = processor
            ::create_market_list(
                &base_model,
                &mint_x_user_accounts,
                &mint_x_mint_accounts,
                &account_list,
                &market_types,
                &market_flags
            )
            .unwrap();
        let mut market = market_list.get(0).unwrap();
        let before_amount_markt_x = market.get_real_time_user_token_amount_x();
        let before_amount_markt_y = market.get_real_time_user_token_amount_y();
        let before_amount_x = unpack_token_account_ammount(
            ctx.accounts.token_pair_0_user_token_account_x.as_ref().unwrap()
        ).unwrap();
        let before_amount_y = unpack_token_account_ammount(
            ctx.accounts.user_token_base.as_ref()
        ).unwrap();

        #[cfg(feature = "debug-out")]
        let cal_out_x = market.out_x(amount_in);
        #[cfg(feature = "debug-out")]
        let cal_out_y = market.out_y(amount_in);
        #[cfg(not(feature = "debug-out"))]
        let cal_out_x = 0u64;
        #[cfg(not(feature = "debug-out"))]
        let cal_out_y = 0u64;
        
        msg!("CU_LOG {}:{}", file!(), line!());
        sol_log_compute_units();
        if use_limit {
            market.swap(y2x, match y2x {
                true => market.limit_in_y(),
                false => market.limit_in_x(),
            })?;
        } else {
            market.swap(y2x, amount_in)?;
        }
        msg!("CU_LOG {}:{}", file!(), line!());
        sol_log_compute_units();
        let after_amount_markt_x = market.get_real_time_user_token_amount_x();
        let after_amount_markt_y = market.get_real_time_user_token_amount_y();

        let after_amount_x = unpack_token_account_ammount(
            ctx.accounts.token_pair_0_user_token_account_x.as_ref().unwrap()
        ).unwrap();
        let after_amount_y = unpack_token_account_ammount(
            ctx.accounts.user_token_base.as_ref()
        ).unwrap();

        msg!(
            "fd:{} fe:{} ax:{} ay:{} lx:{} ly:{} mt:{} cx:{} cy:{} bax:{} bay:{} aax:{} aay:{} bamx:{} bamy:{} aamx:{} aamy:{}",
            market.fee_denominator(),
            market.fee_numerator(),
            market.amount_x(),
            market.amount_y(),
            market.limit_in_x(),
            market.limit_in_y(),
            market.market_type() as u8,
            cal_out_x,
            cal_out_y,
            before_amount_x,
            before_amount_y,
            after_amount_x,
            after_amount_y,
            before_amount_markt_x,
            before_amount_markt_y,
            after_amount_markt_x,
            after_amount_markt_y
        );
        if always_fail {
            return Err(error!(MyErrorCode::FakeProfit));
        } else {
            Ok(())
        }
    }
}
