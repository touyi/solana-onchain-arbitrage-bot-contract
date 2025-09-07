use anchor_lang::prelude::*;
#[error_code]
pub enum MyErrorCode {
    #[msg("mint error")]
    InvalidTokenAccount,
    #[msg("parse base amount error")]
    InvalidBaseAccount,
    #[msg("Not Profit")]
    NoProfit,
    #[msg("Fake Profit")]
    FakeProfit,
    #[msg("no support market")]
    NoSupportMarket,
}