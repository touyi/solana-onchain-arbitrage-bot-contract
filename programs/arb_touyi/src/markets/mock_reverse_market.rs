use anchor_lang::prelude::*;
use crate::base_market::*;

pub struct MockReverseMarketPool<'a, 'info> {
    pub market: Box<dyn BaseMarketPool<'a, 'info> + 'a>,
}

impl<'a, 'info> BaseMarketPool<'a, 'info> for MockReverseMarketPool<'a, 'info> {
    fn get_real_time_user_token_amount_x(&self) -> u64 {
        self.market.get_real_time_user_token_amount_y()
    }

    fn get_real_time_user_token_amount_y(&self) -> u64 {
        self.market.get_real_time_user_token_amount_x()
    }

    fn liquidity(&self) -> u128 {
        self.market.liquidity()
    }

    fn sqrt_price(&self) -> f64 {
        1.0 / self.market.sqrt_price()
    }

    fn valid(&self) -> bool {
        self.market.valid()
    }
    
    fn amount_x(&self) -> u64 {
        self.market.amount_y()
    }
    fn amount_y(&self) -> u64 {
        self.market.amount_x()
    }
    fn fee_numerator(&self) -> u64 {
        self.market.fee_numerator()
    }
    fn fee_denominator(&self) -> u64 {
        self.market.fee_denominator()
    }
    fn price(&self) -> f64 {
        1.0 / self.market.price()
    }
    fn x_mint(&self) -> &Pubkey {
        self.market.y_mint()
    }
    fn y_mint(&self) -> &Pubkey {
        self.market.x_mint()
    }

    fn limit_in_x(&self) -> u64 {
        self.market.limit_in_y()
    }

    fn limit_in_y(&self) -> u64 {
        self.market.limit_in_x()
    }

    #[cfg(feature = "debug-out")]
    fn out_x(&self, in_y: u64) -> u64 {
        self.market.out_y(in_y)
    }

    #[cfg(feature = "debug-out")]
    fn out_y(&self, in_x: u64) -> u64 {
        self.market.out_x(in_x)
    }

    fn swap(&self, y2x: bool, amount_in: u64) -> Result<()> {
        self.market.swap(!y2x, amount_in)
    }
    fn market_type(&self) -> MarketType {
        self.market.market_type()
    }
}

