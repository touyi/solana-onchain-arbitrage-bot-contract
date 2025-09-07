use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;

const RECIPIENT_PUBKEY: Pubkey = pubkey!("B2kcKQCZUWvK59w9V9n7oDiFwqrh5FowymgpsKZV5NHu");

#[derive(Accounts)]
pub struct CommonAccountsInfo64<'info> {
    /// CHECK: NONE
    pub user: Signer<'info>,
    #[account(mut)]
    /// CHECK: NONE
    pub user_token_base: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: NONE
    pub token_base_mint: UncheckedAccount<'info>,
    
    #[account(mut)]
    /// CHECK: NONE
    pub token_program: UncheckedAccount<'info>,

    #[account(address = anchor_lang::solana_program::system_program::ID)]
    /// CHECK: NONE
    pub sys_program: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: NONE
    pub token_pair_0_user_token_account_x: Option<UncheckedAccount<'info>>,

    #[account(mut)]
    /// CHECK: NONE
    pub token_pair_0_mint_x: Option<UncheckedAccount<'info>>,

    #[account(mut, address = RECIPIENT_PUBKEY)]
    /// CHECK: NONE
    pub recipient: AccountInfo<'info>,

    #[account(address = spl_associated_token_account::ID)]
    /// CHECK: NONE
    pub associated_token_program: UncheckedAccount<'info>,
    


    #[account(mut)]
    /// CHECK: NONE
    pub account_0: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_1: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_2: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_3: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_4: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_5: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_6: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_7: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_8: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_9: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_10: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_11: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_12: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_13: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_14: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_15: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_16: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_17: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_18: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_19: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_20: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_21: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_22: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_23: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_24: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_25: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_26: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_27: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_28: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_29: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_30: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_31: Option<UncheckedAccount<'info>>,    
    #[account(mut)]
    /// CHECK: NONE
    pub account_32: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_33: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_34: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_35: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_36: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_37: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_38: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_39: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_40: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_41: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_42: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_43: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_44: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_45: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_46: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_47: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_48: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_49: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_50: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_51: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_52: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_53: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_54: Option<UncheckedAccount<'info>>,
}



#[derive(Accounts)]
pub struct CommonAccountsInfo32<'info> {
    /// CHECK: NONE
    pub user: Signer<'info>,
    #[account(mut)]
    /// CHECK: NONE
    pub user_token_base: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: NONE
    pub token_base_mint: UncheckedAccount<'info>,
    
    #[account(mut)]
    /// CHECK: NONE
    pub token_program: UncheckedAccount<'info>,

    #[account(address = anchor_lang::solana_program::system_program::ID)]
    /// CHECK: NONE
    pub sys_program: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: NONE
    pub token_pair_0_user_token_account_x: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub token_pair_0_mint_x: Option<UncheckedAccount<'info>>,

    #[account(mut, address = RECIPIENT_PUBKEY)]
    /// CHECK: NONE
    pub recipient: AccountInfo<'info>,

    #[account(address = spl_associated_token_account::ID)]
    /// CHECK: NONE
    pub associated_token_program: UncheckedAccount<'info>,


    #[account(mut)]
    /// CHECK: NONE
    pub account_0: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_1: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_2: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_3: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_4: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_5: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_6: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_7: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_8: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_9: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_10: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_11: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_12: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_13: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_14: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_15: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_16: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_17: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_18: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_19: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_20: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_21: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_22: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_23: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_24: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_25: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_26: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_27: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK: NONE
    pub account_28: Option<UncheckedAccount<'info>>,
}
