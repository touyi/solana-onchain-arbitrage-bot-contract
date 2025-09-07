use anchor_lang::prelude::*;

pub struct BaseModel<'a, 'info> {
    pub user: &'a AccountInfo<'info>,
    /// CHECK: NONE
    pub user_token_base: &'a AccountInfo<'info>,
    /// CHECK: NONE
    pub token_base_mint: &'a AccountInfo<'info>,
    /// CHECK: NONE
    pub token_program: &'a AccountInfo<'info>,
    /// CHECK: NONE
    pub sys_program: &'a AccountInfo<'info>,

    pub recipient: &'a AccountInfo<'info>,

    /// CHECK: NONE
    pub associated_token_program: &'a AccountInfo<'info>,
}