use anchor_lang::prelude::*;
use crate::base_market::*;
use crate::model::base_model::*;
use crate::model::errors::*;
use crate::common::accounts_iter::*;
use crate::utils::utils::*;
use anchor_lang::solana_program::log::sol_log_compute_units;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::cmp::Ordering;
use crate::markets::*;
use anchor_spl::token_interface::{self, TransferChecked};

pub fn process_arb<'a, 'info>(arb_markets: &Vec<Box<dyn BaseMarketPool<'a, 'info> + 'a>>, 
                            max_in: u64, 
                            min_profit: u64,
                        ) -> Result<()> {
    for i in 0..arb_markets.len() {
        if !arb_markets[i].valid() {
            continue;
        }
        for j in 0..arb_markets.len() {
            if!arb_markets[j].valid() {
                continue;
            }
            if i == j || arb_markets[i].x_mint() != arb_markets[j].x_mint() {
                continue;
            }
            match try_arb(arb_markets[i].deref(), 
                        arb_markets[j].deref(), 
                        max_in, 
                        min_profit) {
                Ok(need_coutinue) => {
                    if !need_coutinue {
                        return Ok(());
                    }
                }
                Err(e) => {
                    msg!("try_arb Fail:{} next", e);
                    return Err(e.into());
                }
            };
        }
    }
    return Err(error!(MyErrorCode::NoProfit));
}

/// return true means need continue arb, false means stop arb, already get profit
pub fn try_arb<'a, 'info>(
    market1: &dyn BaseMarketPool<'a, 'info>,
    market2: &dyn BaseMarketPool<'a, 'info>,
    max_in: u64,
    min_profit: u64,
) -> Result<bool> {
    let (in_y, dx, out_y) = market_arb_calc(market1, market2);
    msg!("iy:{} dx:{} oy:{}", in_y, dx, out_y);
    if in_y > 1.0 && dx > 1.0 && out_y > 1.0 && out_y > in_y + min_profit as f64 {
        let in_y = if in_y > max_in as f64 {
            max_in as f64
        } else {
            in_y
        };
        
        market1.swap(true, in_y as u64)?;
        let in_x = market1.get_real_time_user_token_amount_x();
        market2.swap(false, in_x)?;
        msg!("real in_y:{} in_x:{}", in_y, in_x);
        return Ok(false);
    }
    return Ok(true);
}

pub fn market_arb_calc(
    market1: &dyn BaseMarketPool,
    market2: &dyn BaseMarketPool,
) -> (f64, f64, f64) {
    if market1.market_type() == MarketType::NormalAMM
        && market2.market_type() == MarketType::NormalAMM
    {
        // AMM -> AMM
        let a1_f = 1.0 - (market1.fee_numerator() as f64 / market1.fee_denominator() as f64);
        let a2_f = 1.0 - (market2.fee_numerator() as f64 / market2.fee_denominator() as f64);
        let a1_x = market1.amount_x() as f64;
        let a1_y = market1.amount_y() as f64;
        let a2_x = market2.amount_x() as f64;
        let a2_y = market2.amount_y() as f64;

        let sqrt_0 = ((a1_f * a1_x * a1_y * a2_f) / (a2_x * a2_y)).sqrt();

        let numerator_0 = a1_f * a1_x * a2_f;
        let numerator_1 = a1_x * a1_y * a2_f / (a2_y);
        let numerator_2 = a1_x * a2_f * sqrt_0;
        let numerator_3 = a2_x * sqrt_0;

        let denominator_0 = a1_f * a2_f;
        let denominator_1 = a1_x * a1_y * a2_f * a2_f / (a2_x * a2_y);

        let a1_dx_pos = (numerator_0 + numerator_1 - numerator_2 - numerator_3) / (denominator_0 - denominator_1);
        let a1_dx_neg = (numerator_0 + numerator_1 + numerator_2 + numerator_3) / (denominator_0 - denominator_1);

        let a1_dy_pos = ((a1_y * a1_x) / (a1_x - a1_dx_pos) - a1_y) / a1_f;
        let a2_dy_pos = a2_y - (a2_y * a2_x) / (a2_x + a2_f * a1_dx_pos);

        if a1_dy_pos > 0.0 && a2_dy_pos > 0.0 && a1_dx_pos > 0.0 && a2_dy_pos > a1_dy_pos {
            return (a1_dy_pos, a1_dx_pos, a2_dy_pos);
        }

        let a1_dy_neg = ((a1_y * a1_x) / (a1_x - a1_dx_neg) - a1_y) / a1_f;
        let a2_dy_neg = a2_y - (a2_y * a2_x) / (a2_x + a2_f * a1_dx_neg);
        return (a1_dy_neg, a1_dx_neg, a2_dy_neg);

    } else if market1.market_type() == MarketType::NormalCMM
        && market2.market_type() == MarketType::NormalCMM
    {
        let c1_p = market1.price();
        let c2_p = market2.price();
        let c1_f = 1.0 - (market1.fee_numerator() as f64 / market1.fee_denominator() as f64);
        let c2_f = 1.0 - (market2.fee_numerator() as f64 / market2.fee_denominator() as f64);
        let cd1_x = market1.amount_x() as f64;
        let profit = c1_p * c1_f * c2_f / c2_p;
        if profit > 1.0 && cd1_x > 1.0 {
            let mut cd1_y = market1.amount_y() as f64;

            let mut cd2_y = market2.amount_y() as f64;
            // 使用 partial_cmp 处理 f64 比较
            cd1_y = if cd1_y
                .partial_cmp(&(cd2_y / profit))
                .unwrap_or(Ordering::Less)
                == Ordering::Less
            {
                cd1_y
            } else {
                cd2_y / profit
            };
            cd1_y = if cd1_y
                .partial_cmp(&(cd1_x / c1_f / c1_p))
                .unwrap_or(Ordering::Less)
                == Ordering::Less
            {
                cd1_y
            } else {
                cd1_x / c1_f / c1_p
            };
            cd2_y = cd1_y * profit;
            return (cd1_y, cd1_y * c1_f * c1_p, cd2_y);
        } else {
            return (0.0, 0.0, 0.0);
        }
    } else if market1.market_type() == MarketType::NormalAMM
        && market2.market_type() == MarketType::NormalCMM
    {
        // AMM -> CMM
        let a_x = market1.amount_x() as f64;
        let a_y = market1.amount_y() as f64;
        let c_f = 1.0 - (market2.fee_numerator() as f64 / market2.fee_denominator() as f64);
        let a_f = 1.0 - (market1.fee_numerator() as f64 / market1.fee_denominator() as f64);
        let p = market2.price();

        let numerator_0 = a_x;
        let numerator_1 = ((p * a_y * a_x) / (a_f * c_f)).sqrt();

        let mut a_dx_pos = numerator_0 - numerator_1;
        let mut a_dy_pos = ((a_y * a_x) / (a_x - a_dx_pos) - a_y) / a_f;
        let mut c_dy_pos = a_dx_pos * c_f / p;

        if c_dy_pos > market2.amount_y() as f64 {
            c_dy_pos = market2.amount_y() as f64;
            a_dx_pos = c_dy_pos * p / c_f;
            a_dy_pos = ((a_y * a_x) / (a_x - a_dx_pos) - a_y) / a_f;
        }

        if a_dx_pos > 0.0 && a_dy_pos > 0.0 && c_dy_pos > 0.0 && c_dy_pos > a_dy_pos {
            return (a_dy_pos, a_dx_pos, c_dy_pos);
        }

        let mut a_dx_neg = numerator_0 + numerator_1;
        let mut a_dy_neg = ((a_y * a_x) / (a_x - a_dx_neg) - a_y) / a_f;
        let mut c_dy_neg = a_dx_neg * c_f / p;

        if c_dy_neg > market2.amount_y() as f64 {
            c_dy_neg = market2.amount_y() as f64;
            a_dx_neg = c_dy_neg * p / c_f;
            a_dy_neg = ((a_y * a_x) / (a_x - a_dx_neg) - a_y) / a_f;
        }
        return (a_dy_neg, a_dx_neg, c_dy_neg);
    } else if market1.market_type() == MarketType::NormalCMM
        && market2.market_type() == MarketType::NormalAMM
    {
        // CMM -> AMM
        // let c_x = market1.amount_x();
        // let c_y = market1.amount_y();
        let a_x = market2.amount_x() as f64;
        let a_y = market2.amount_y() as f64;
        let c_f = 1.0 - (market1.fee_numerator() as f64 / market1.fee_denominator() as f64);
        let a_f = 1.0 - (market2.fee_numerator() as f64 / market2.fee_denominator() as f64);
        let p = market1.price();

        let numerator_0 = -a_x;
        let numerator_1 = (a_y * a_f * a_x * p * c_f).sqrt();

        let mut c_dx_pos = numerator_0 + numerator_1;

        c_dx_pos = if c_dx_pos
            .partial_cmp(&(market1.amount_x() as f64))
            .unwrap_or(Ordering::Greater)
            == Ordering::Greater
        {
            market1.amount_x() as f64
        } else {
            c_dx_pos
        };
        let c_dy_pos = c_dx_pos / (p * c_f);
        let a_dy_pos = a_y - (a_y * a_x) / (a_x + c_dx_pos * a_f);

        if c_dx_pos > 0.0 && c_dy_pos > 0.0 && a_dy_pos > 0.0 && a_dy_pos > c_dy_pos  {
            return (c_dy_pos, c_dx_pos, a_dy_pos);
        }

        let mut c_dx_neg = numerator_0 - numerator_1;
        c_dx_neg = if c_dx_neg
            .partial_cmp(&(market1.amount_x() as f64))
            .unwrap_or(Ordering::Greater)
            == Ordering::Greater
        {
            market1.amount_x() as f64
        } else {
            c_dx_neg
        };

        let c_dy_neg = c_dx_neg / (p * c_f);
        let a_dy_neg = a_y - (a_y * a_x) / (a_x + c_dx_neg * a_f);
        return (c_dy_neg, c_dx_neg, a_dy_neg);

    } else if market1.market_type() == MarketType::NormalCLMM
        && market2.market_type() == MarketType::NormalAMM
    {
        let c_f = 1.0 - (market1.fee_numerator() as f64 / market1.fee_denominator() as f64);
        let a_f = 1.0 - (market2.fee_numerator() as f64 / market2.fee_denominator() as f64);
        let a_x = market2.amount_x() as f64;
        let a_y = market2.amount_y() as f64;
        let sp = market1.sqrt_price();
        let l = market1.liquidity() as f64;
        let numerator_0 = -a_f*a_y*c_f;
        let numerator_1 = a_f*l*sp;
        let sqrt_0 = (a_f*a_y*c_f*a_x).sqrt();
        let numerator_2 = a_f*l*sqrt_0 / a_x;
        let numerator_3 = sp*sqrt_0;
        let denominator_0 = a_f * a_f *l*sp / (a_x);
        let denominator_1 = a_f*a_y*c_f*sp / (l);
        let mut dx_pos = (numerator_0 - numerator_1 + numerator_2 + numerator_3) / (denominator_0 - denominator_1);

        let mut cdy_pos = (l / c_f) * (sp * sp * dx_pos) / (l - sp * dx_pos);
        if cdy_pos > market1.limit_in_y() as f64 {
            cdy_pos = market1.limit_in_y() as f64;
            dx_pos = c_f * cdy_pos * l / (sp * (c_f * cdy_pos + l * sp));
        }
        let ady_pos = (a_y * dx_pos * a_f) / (a_x + dx_pos * a_f);
        if dx_pos > 0.0 && cdy_pos > 0.0 && ady_pos > 0.0 && ady_pos > cdy_pos {
            return (cdy_pos, dx_pos, ady_pos);
        }

        let mut dx_neg = (numerator_0 - numerator_1 - numerator_2 - numerator_3) / (denominator_0 - denominator_1);
        let mut cdy_neg = (l / c_f) * (sp * sp * dx_neg) / (l - sp * dx_neg);
        if cdy_neg > market1.limit_in_y() as f64 {
            cdy_neg = market1.limit_in_y() as f64;
            dx_neg = c_f * cdy_neg * l / (sp * (c_f * cdy_neg + l * sp));
        }
        let ady_neg = (a_y * dx_neg * a_f) / (a_x + dx_neg * a_f);
        return (cdy_neg, dx_neg, ady_neg)
    } else if market1.market_type() == MarketType::NormalCLMM
        && market2.market_type() == MarketType::NormalCMM
    {
        let sp = market1.sqrt_price();
        let l = market1.liquidity() as f64;
        let cl_f = 1.0 - (market1.fee_numerator() as f64 / market1.fee_denominator() as f64);
        let c_f = 1.0 - (market2.fee_numerator() as f64 / market2.fee_denominator() as f64);
        let c_p = market2.price();

        let numerator_0 = c_f*cl_f*l / sp;
        let numerator_1 = l*(c_f*c_p*cl_f).sqrt();

        let denominator_0 = c_f*cl_f;

        let mut dx_pos = (numerator_0 - numerator_1) / denominator_0;
        let mut cl_dy_pos= (l/(cl_f*(-dx_pos*sp + l)))*dx_pos*sp*sp;

        if cl_dy_pos > market1.limit_in_y() as f64 {
            cl_dy_pos = market1.limit_in_y() as f64;
            dx_pos = (cl_dy_pos*cl_f*l/sp)/(cl_dy_pos*cl_f + l*sp);
        }
        let mut c_dy_pos = c_f*dx_pos/c_p;
        if c_dy_pos > market2.amount_y() as f64 {
            c_dy_pos = market2.amount_y() as f64;
            dx_pos = c_dy_pos*c_p/c_f;
            cl_dy_pos = (l/(cl_f*(-dx_pos*sp + l)))*dx_pos*sp*sp;
            
        }
        if dx_pos > 0.0 && cl_dy_pos > 0.0 && c_dy_pos > 0.0 && c_dy_pos > cl_dy_pos {
            return (cl_dy_pos, dx_pos, c_dy_pos);
        }

        let mut dx_neg = (numerator_0 + numerator_1) / denominator_0;
        let mut cl_dy_neg = (l/(cl_f*(-dx_neg*sp + l)))*dx_neg*sp*sp;
        
        if cl_dy_neg > market1.limit_in_y() as f64 {
            cl_dy_neg = market1.limit_in_y() as f64;
            dx_neg = (cl_dy_neg*cl_f*l/sp)/(cl_dy_neg*cl_f + l*sp);
        }
        let mut c_dy_neg = c_f*dx_neg/c_p;
        if c_dy_neg > market2.amount_y() as f64 {
            c_dy_neg = market2.amount_y() as f64;
            dx_neg = c_dy_neg*c_p/c_f;
            cl_dy_neg = (l/(cl_f*(-dx_neg*sp + l)))*dx_neg*sp*sp;
            
        }
        return (cl_dy_neg, dx_neg, c_dy_neg)
    } else if market1.market_type() == MarketType::NormalCLMM
        && market2.market_type() == MarketType::NormalCLMM
    {
        let sp = market1.sqrt_price();
        let l = market1.liquidity() as f64;
        let cl_f = 1.0 - (market1.fee_numerator() as f64 / market1.fee_denominator() as f64);

        let sp2 = market2.sqrt_price();
        let l2 = market2.liquidity() as f64;
        let cl2_f = 1.0 - (market2.fee_numerator() as f64 / market2.fee_denominator() as f64);

        let sqrt_0 = f64::sqrt(cl2_f*cl_f);
        
        let numerator_0 = -cl2_f*cl_f*l2 / (sp);
        let numerator_1 = cl2_f*l / (sp2);
        let numerator_2 = cl2_f*l*sqrt_0  / (sp);
        let numerator_3 = l2*sqrt_0 / (sp2);

        let denominator_0 = cl2_f*cl2_f*l / (l2);
        let denominator_1 = cl2_f*cl_f*l2 / (l);

        let mut cl_dx_pos = (numerator_0 - numerator_1 + numerator_2 + numerator_3) / (denominator_0 - denominator_1);
        // let cl_dy_pos = cl_dx_pos*l*sp*sp/(cl_f*(-cl_dx_pos*sp + l));
        // let cl2_dy_pos = cl2_f*cl_dx_pos*l2*sp2*sp2/(cl2_f*cl_dx_pos*sp2 + l2);

        if cl_dx_pos > market2.limit_in_x() as f64 {
            cl_dx_pos = market2.limit_in_x() as f64
        }
        let mut cl_dy_pos = cl_dx_pos*l*sp*sp/(cl_f*(-cl_dx_pos*sp + l));
        if cl_dy_pos > market1.limit_in_y() as f64{
            cl_dy_pos = market1.limit_in_y() as f64;
            cl_dx_pos = cl_dy_pos*cl_f*l/(sp*(cl_dy_pos*cl_f + l*sp))
        }
        let cl2_dy_pos = cl2_f*cl_dx_pos*l2*sp2*sp2/(cl2_f*cl_dx_pos*sp2 + l2);
        if cl_dx_pos > 0.0 && cl_dy_pos > 0.0 && cl2_dy_pos > 0.0 && cl2_dy_pos > cl_dy_pos {
            return (cl_dy_pos, cl_dx_pos, cl2_dy_pos);
        } 

        let mut cl_dx_neg = (numerator_0 - numerator_1 - numerator_2 - numerator_3) / (denominator_0 - denominator_1);
        if cl_dx_neg > market2.limit_in_x() as f64 {
            cl_dx_neg = market2.limit_in_x() as f64
        }

        let mut cl_dy_neg = cl_dx_neg*l*sp*sp/(cl_f*(-cl_dx_neg*sp + l));
        if cl_dy_neg > market1.limit_in_y() as f64{
            cl_dy_neg = market1.limit_in_y() as f64;
            cl_dx_neg = cl_dy_neg*cl_f*l/(sp*(cl_dy_neg*cl_f + l*sp))
        }
        let cl2_dy_neg = cl2_f*cl_dx_neg*l2*sp2*sp2/(cl2_f*cl_dx_neg*sp2 + l2);
        return (cl_dy_neg, cl_dx_neg, cl2_dy_neg);

    } else if market1.market_type() == MarketType::NormalAMM
        && market2.market_type() == MarketType::NormalCLMM
    {
        let a_y = market1.amount_y() as f64;
        let a_x = market1.amount_x() as f64;
        let a_f = 1.0 - (market1.fee_numerator() as f64 / market1.fee_denominator() as f64);
        let sp2 = market2.sqrt_price();
        let l2 = market2.liquidity() as f64;
        let cl2_f = 1.0 - (market2.fee_numerator() as f64 / market2.fee_denominator() as f64);

        let numerator_0 = a_f*cl2_f*l2*sp2;
        let numerator_1 = a_y*cl2_f;
        let sqrt_0 = f64::sqrt(a_f*a_x*a_y*cl2_f);
        let numerator_2 = cl2_f*sp2*sqrt_0;
        let numerator_3 = l2*sqrt_0 / a_x;

        let denominator_0 = a_f*cl2_f*l2*sp2 / a_x;
        let denominator_1 = a_y*cl2_f*cl2_f*sp2 / l2;

        let mut dx_pos = (numerator_0 + numerator_1 - numerator_2 - numerator_3) / (denominator_0 - denominator_1);
        if dx_pos > market2.limit_in_x() as f64 {
            dx_pos = market2.limit_in_x() as f64
        }
        let a_dy_pos = -dx_pos*a_y/(a_f*(dx_pos - a_x));
        let cl2_dy_pos = dx_pos*cl2_f*l2*sp2/(dx_pos*cl2_f + l2 / sp2);
        if dx_pos > 0.0 && a_dy_pos > 0.0 && cl2_dy_pos > 0.0 && cl2_dy_pos > a_dy_pos {
            return (a_dy_pos, dx_pos, cl2_dy_pos);
        }

        let mut dx_neg = (numerator_0 + numerator_1 + numerator_2 + numerator_3) / (denominator_0 - denominator_1);

        if dx_neg > market2.limit_in_x() as f64 {
            dx_neg = market2.limit_in_x() as f64
        }
        let a_dy_neg = -dx_neg*a_y/(a_f*(dx_neg - a_x));
        let cl2_dy_neg = dx_neg*cl2_f*l2*sp2/(dx_neg*cl2_f + l2 / sp2);
        return (a_dy_neg, dx_neg, cl2_dy_neg);

    } else if market1.market_type() == MarketType::NormalCMM
        && market2.market_type() == MarketType::NormalCLMM
    {
        let c_p = market1.price();
        let c_f = 1.0 - (market1.fee_numerator() as f64 / market1.fee_denominator() as f64);
        
        let sp2 = market2.sqrt_price();
        let l2 = market2.liquidity() as f64;
        let cl2_f = 1.0 - (market2.fee_numerator() as f64 / market2.fee_denominator() as f64);

        let numerator_0 = -l2 / sp2;
        let numerator_1 = l2*f64::sqrt(c_f*c_p*cl2_f);

        let denominator_0 = cl2_f;

        let mut dx_pos = (numerator_0 + numerator_1) / denominator_0;

        if dx_pos > market2.limit_in_x() as f64 {
            dx_pos = market2.limit_in_x() as f64
        }
        if dx_pos > market1.amount_x() as f64 {
            dx_pos = market1.amount_x() as f64
        }
        let common_0 = dx_pos*cl2_f*sp2;
        let c_dy_pos = dx_pos/(c_f*c_p);
        let cl2_dy_pos = (common_0*sp2/(common_0 + l2)) * l2;
        if dx_pos > 0.0 && c_dy_pos > 0.0 && cl2_dy_pos > 0.0 && cl2_dy_pos > c_dy_pos {
            return (c_dy_pos, dx_pos, cl2_dy_pos);
        }

        let mut dx_neg = (numerator_0 - numerator_1) / denominator_0;
        if dx_neg > market2.limit_in_x() as f64 {
            dx_neg = market2.limit_in_x() as f64
        }
        if dx_neg > market1.amount_x() as f64 {
            dx_neg = market1.amount_x() as f64
        }
        let common_0 = dx_neg*cl2_f*sp2;
        let c_dy_neg = dx_neg/(c_f*c_p);
        let cl2_dy_neg = (common_0*sp2/(common_0 + l2)) * l2;

        return (c_dy_neg, dx_neg, cl2_dy_neg)
    }
    (0.0, 0.0, 0.0)
}


pub fn create_market_list<'a, 'info>(base: &'a BaseModel<'a, 'info>,
            min_x_user_accounts: &'a Vec<&Option<UncheckedAccount<'info>>>,
            mint_x_mint_accounts: &'a Vec<&Option<UncheckedAccount<'info>>>,
            account_vec: &'a Vec<&Option<UncheckedAccount<'info>>>,
            market_type: &'a Vec<u8>,
            market_flag: &'a Vec<u8>) -> Result<Box<Vec<Box<dyn BaseMarketPool<'a, 'info> + 'a>>>> {
                let mut result: Vec<Box<dyn BaseMarketPool<'a, 'info>>> = Vec::new();

                let accounts_iter = Rc::new(RefCell::new(AccountsIter::new(account_vec)));
                for i in 0..market_type.len() {
                    let mint_x_index = (market_flag[i] & 0x7F) as usize;
                    match market_type[i] {
                        0 => {
                            // Meteora DLMM
                            result.push(
                                meteora_market::MeteorCMMMarketPool::create_market(
                                    &base,
                                    min_x_user_accounts[mint_x_index],
                                    mint_x_mint_accounts[mint_x_index], 
                                    accounts_iter.clone(),
                                    is_bit_set(market_flag[i], 7)
                                )
                            );

                        },
                        1 => {
                            result.push(raydium_market::RaydiumAMMMarketPool::create_market(
                                &base,
                                min_x_user_accounts[mint_x_index],
                                mint_x_mint_accounts[mint_x_index], 
                                accounts_iter.clone(),
                                is_bit_set(market_flag[i], 7)
                            ));
                        },
                        2 => {
                            result.push(pumpswap_market::PumpSwapMarketPool::create_market(
                                &base,
                                min_x_user_accounts[mint_x_index],
                                mint_x_mint_accounts[mint_x_index], 
                                accounts_iter.clone(),
                                is_bit_set(market_flag[i], 7)
                            ));
                        }
                        3 => {
                            result.push(
                                raydium_clmm_market::RaydiumCLMMMarketPool::create_market(
                                    &base,
                                    min_x_user_accounts[mint_x_index],
                                    mint_x_mint_accounts[mint_x_index],
                                    accounts_iter.clone(),
                                    is_bit_set(market_flag[i], 7)
                                )
                            );
                        }
                        4 => {
                            result.push(
                                raydium_cpmm_market::RaydiumCPMMMarketPool::create_market(
                                    &base,
                                    min_x_user_accounts[mint_x_index],
                                    mint_x_mint_accounts[mint_x_index],
                                    accounts_iter.clone(),
                                    is_bit_set(market_flag[i], 7)
                                )
                            );
                        }
                        // TODO(touyi): NEXT add meteora amm
                        5 => {
                            result.push(
                                meteora_amm_market::MeteoraAMMMarketPool::create_market(
                                    &base,
                                    min_x_user_accounts[mint_x_index],
                                    mint_x_mint_accounts[mint_x_index],
                                    accounts_iter.clone(),
                                    is_bit_set(market_flag[i], 7)
                                )
                            );
                        }
                        _ => {
                            return Err(error!(MyErrorCode::NoSupportMarket))
                        }
                    }
                }
                Ok(Box::new(result))
            }

pub fn main_process<'a, 'info>(base: &'a BaseModel<'a, 'info>, 
            min_x_user_accounts: &'a Vec<&Option<UncheckedAccount<'info>>>, 
            mint_x_mint_accounts: &'a Vec<&Option<UncheckedAccount<'info>>>, 
            account_vec: &'a Vec<&Option<UncheckedAccount<'info>>>, 
            market_type: &'a Vec<u8>,
            market_flag: &'a Vec<u8>,
            max_in: u64,
            min_profit: u64) -> Result<()> {
    for i in 0..min_x_user_accounts.len() {
        let min_x_account = min_x_user_accounts[i];
        let mint_x = mint_x_mint_accounts[i];
        create_associated_token_account(base.user.to_account_info(), 
                            mint_x.as_ref().unwrap().to_account_info(), 
                            base.token_program.to_account_info(), 
                            min_x_account.as_ref().unwrap().to_account_info(), 
                            base.sys_program.to_account_info(), 
                            base.associated_token_program.to_account_info())?;
    }
    
    // create all markets
    let market_list = create_market_list(base, min_x_user_accounts, mint_x_mint_accounts, account_vec, market_type, market_flag)?;
    
    let before_base_amount =
                unpack_token_account_ammount(base.user_token_base.as_ref())?;
    
    // find path to arbitrage
    process_arb(market_list.deref(), max_in, min_profit)?;

    let after_base_amount =
                unpack_token_account_ammount(base.user_token_base.as_ref())?;

    // check if has profit
    if after_base_amount < before_base_amount + min_profit {
        return Err(error!(MyErrorCode::FakeProfit))
    } else {
        let profit = after_base_amount - before_base_amount;
        let fee = profit.div_ceil(10);
        if fee > 0 {
            let cpi_accounts = TransferChecked {
                mint: base.token_base_mint.to_account_info(),
                from: base.user_token_base.to_account_info(),
                to: base.recipient.to_account_info(),
                authority: base.user.to_account_info(),
            };
            
            let cpi_program = base.token_program.to_account_info();
            let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
            token_interface::transfer_checked(cpi_context, fee, 9)?;
        }
        msg!("profit: {}", profit);
    }

    for i in 0..min_x_user_accounts.len() {
        let min_x_account = min_x_user_accounts[i];
        match close_token_account(base.user.to_account_info(), 
                                min_x_account.as_ref().unwrap().to_account_info(), 
                                base.token_program.to_account_info()) {
                                    Ok(_) => {
                                    } 
                                    Err(_) => {
                                        msg!("close min_x_account failed {}", min_x_account.as_ref().unwrap().key);
                                    }
                                }
    }
    Ok(())
}