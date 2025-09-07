use std::cell::RefCell;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;
use hex::FromHex;
// use base64::{engine::general_purpose, Engine};
use crate::base_market::*;
use crate::common::accounts_iter::*;
use crate::common::big_num::*;
use crate::model::errors::*;
use crate::markets::mock_reverse_market::*;
use crate::utils::utils::unpack_token_account_ammount;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke};

const TICK_ARRAY_SIZE: i32 = 60;
const TICK_ARRAY_BITMAP_SIZE: i32 = 512;
pub const MIN_TICK: i32 = -443636;
pub const MAX_TICK: i32 = -MIN_TICK;
const NUM_64: U128 = U128([64, 0]);
const RESOLUTION: u8 = 64;
const Q64: u128 = (u64::MAX as u128) + 1; // 2^64
pub const FEE_RATE_DENOMINATOR_VALUE: u32 = 1_000_000;

pub struct RaydiumCLMMMarketPool<'a, 'info> {
    pub program: &'a AccountInfo<'info>,
    pub user: &'a AccountInfo<'info>,
    pub amm_config: &'a AccountInfo<'info>,
    pub pool_state: &'a AccountInfo<'info>,
    pub user_token_account_x: &'a AccountInfo<'info>,
    pub user_token_account_y: &'a AccountInfo<'info>,
    pub pool_token_account_x: &'a AccountInfo<'info>,
    pub pool_token_account_y: &'a AccountInfo<'info>,
    pub observation_state: &'a AccountInfo<'info>,
    pub token_program: &'a AccountInfo<'info>,
    pub token_program_2022: &'a AccountInfo<'info>,
    pub memo_program: &'a AccountInfo<'info>,
    pub token_x_mint: &'a AccountInfo<'info>,
    pub token_y_mint: &'a AccountInfo<'info>,
    pub ex_bit_map: &'a AccountInfo<'info>,
    pub tick_array_vec: Vec<&'a AccountInfo<'info>>,

    pub fee_numerator: u32,
    pub limit_in_x: u64,
    pub limit_in_y: u64,
    pub amount_x: u128,
    pub amount_y: u128,
    pub liquidity: u128,
    pub sqrt_price_x64: u128,
    pub sqrt_price: f64,
    
}

impl<'a, 'info> RaydiumCLMMMarketPool<'a, 'info> {
    pub fn get_next_sqrt_price_from_amount_0_rounding_up(
        &self,
        sqrt_price_x64: u128,
        liquidity: u128,
        amount: u64,
        add: bool,
    ) -> u128 {
        if amount == 0 {
            return sqrt_price_x64;
        };
        let numerator_1 = (U256::from(liquidity)) << RESOLUTION;
    
        if add {
            if let Some(product) = U256::from(amount).checked_mul(U256::from(sqrt_price_x64)) {
                let denominator = numerator_1 + U256::from(product);
                if denominator >= numerator_1 {
                    return numerator_1
                                .mul(U256::from(sqrt_price_x64))
                                .add(denominator.sub(1))
                                .div(denominator).as_u128();
                };
            }
            
            U256::div_rounding_up(
                numerator_1,
                (numerator_1 / U256::from(sqrt_price_x64))
                    .checked_add(U256::from(amount))
                    .unwrap(),
            )
            .as_u128()
        } else {
            let product = U256::from(
                U256::from(amount)
                    .checked_mul(U256::from(sqrt_price_x64))
                    .unwrap(),
            );
            let denominator = numerator_1.checked_sub(product).unwrap();
            numerator_1
                .mul(U256::from(sqrt_price_x64))
                .add(denominator.sub(1))
                .div(denominator).as_u128()
        }
    }

    pub fn get_next_sqrt_price_from_amount_1_rounding_down(
        &self,
        sqrt_price_x64: u128,
        liquidity: u128,
        amount: u64,
        add: bool,
    ) -> u128 {
        if add {
            let quotient = U256::from(u128::from(amount) << RESOLUTION) / liquidity;
            sqrt_price_x64.checked_add(quotient.as_u128()).unwrap()
        } else {
            let quotient = U256::div_rounding_up(
                U256::from(u128::from(amount) << RESOLUTION),
                U256::from(liquidity),
            );
            sqrt_price_x64.checked_sub(quotient.as_u128()).unwrap()
        }
    }
    
    
    pub fn get_next_sqrt_price_from_input(
        &self,
        sqrt_price_x64: u128,
        liquidity: u128,
        amount_in: u64,
        zero_for_one: bool,
    ) -> u128 {
    
        // round to make sure that we don't pass the target price
        if zero_for_one {
            self.get_next_sqrt_price_from_amount_0_rounding_up(sqrt_price_x64, liquidity, amount_in, true)
        } else {
            self.get_next_sqrt_price_from_amount_1_rounding_down(sqrt_price_x64, liquidity, amount_in, true)
        }
    }

    pub fn find_tick_state_low_hight(
        &self,
        array_start_tick_index: i32,
        current_index: i32,
        tick_spacing: u16,
    ) -> (i32, i32) {
        for i in 0..self.tick_array_vec.len() {
            let array_state_data = self.tick_array_vec[i].data.borrow();
            let mut start_tick_index_buffer = [0u8; 4];
            start_tick_index_buffer.copy_from_slice(&array_state_data[8 + 32..8 + 32 + 4]);
            let start_tick_index = i32::from_le_bytes(start_tick_index_buffer);

            let is_initialized = |offset_inarray: i32| {
                let base_offset = (8 + 32 + 4) as usize;
                let tick_offset = (offset_inarray * 168) as usize;
                let mut liquidity_gross_buffer = [0u8; 16];
                liquidity_gross_buffer.copy_from_slice(
                    &array_state_data[base_offset + tick_offset..base_offset + tick_offset + 16],
                );
                let liquidity_gross = u128::from_le_bytes(liquidity_gross_buffer);
                return liquidity_gross != 0;
            };

            if start_tick_index == array_start_tick_index {
                // found
                let current_array_offset =
                    (current_index - array_start_tick_index) / tick_spacing as i32;
                let mut low_current_array_offset = current_array_offset;
                // find low
                while low_current_array_offset >= 0 {
                    if is_initialized(low_current_array_offset) {
                        break;
                    }
                    low_current_array_offset -= 1;
                }
                // find hight
                let mut hight_current_array_offset = current_array_offset + 1;
                while hight_current_array_offset < TICK_ARRAY_SIZE {
                    if is_initialized(hight_current_array_offset) {
                        break;
                    }
                    hight_current_array_offset += 1;
                }
                return (
                    low_current_array_offset * tick_spacing as i32 + array_start_tick_index,
                    hight_current_array_offset * tick_spacing as i32 + array_start_tick_index,
                );
            }
        }
        msg!("find_tick_state_low_hight finish ERROR:not found");
        return (0, 0);
    }

    /// Gets the delta amount_0 for given liquidity and price range
    ///
    /// # Formula
    ///
    /// * `Δx = L * (1 / √P_lower - 1 / √P_upper)`
    /// * i.e. `L * (√P_upper - √P_lower) / (√P_upper * √P_lower)`
    pub fn get_delta_amount_0_unsigned(
        &self,
        mut sqrt_ratio_a_x64: u128,
        mut sqrt_ratio_b_x64: u128,
        liquidity: u128,
        round_up: bool,
    ) -> u64 {
        // sqrt_ratio_a_x64 should hold the smaller value
        if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
            std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
        };

        let numerator_1 = U256::from(liquidity) << RESOLUTION;
        let numerator_2 = U256::from(sqrt_ratio_b_x64 - sqrt_ratio_a_x64);

        assert!(sqrt_ratio_a_x64 > 0);

        let result = if round_up {
            U256::div_rounding_up(
                numerator_1
                .mul(numerator_2)
                .add(U256::from(sqrt_ratio_b_x64).sub(1))
                .div(U256::from(sqrt_ratio_b_x64)),
                U256::from(sqrt_ratio_a_x64),
            )
        } else {
            numerator_1
                .mul(numerator_2).div(U256::from(sqrt_ratio_b_x64))
                / U256::from(sqrt_ratio_a_x64)
        };
        return result.as_u64();
    }

    /// Gets the delta amount_1 for given liquidity and price range
    /// * `Δy = L (√P_upper - √P_lower)`
    pub fn get_delta_amount_1_unsigned(
        &self,
        mut sqrt_ratio_a_x64: u128,
        mut sqrt_ratio_b_x64: u128,
        liquidity: u128,
        round_up: bool,
    ) -> u64 {
        // sqrt_ratio_a_x64 should hold the smaller value
        if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
            std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
        };

        let result = if round_up {
            U256::from(liquidity)
                .mul(U256::from(sqrt_ratio_b_x64 - sqrt_ratio_a_x64))
                .add(Q64.sub(1))
                .div(U256::from(Q64))
        } else {
            U256::from(liquidity)
                .mul(U256::from(sqrt_ratio_b_x64 - sqrt_ratio_a_x64))
                .div(U256::from(Q64))
        };
        return result.as_u64();
    }

    pub fn get_sqrt_price_at_tick_x64(&self, tick: i32) -> u128 {
        let abs_tick = tick.abs() as u32;

        // i = 0
        let mut ratio = if abs_tick & 0x1 != 0 {
            U128([0xfffcb933bd6fb800, 0])
        } else {
            // 2^64
            U128([0, 1])
        };
        // i = 1
        if abs_tick & 0x2 != 0 {
            ratio = (ratio * U128([0xfff97272373d4000, 0])) >> NUM_64
        };
        // i = 2
        if abs_tick & 0x4 != 0 {
            ratio = (ratio * U128([0xfff2e50f5f657000, 0])) >> NUM_64
        };
        // i = 3
        if abs_tick & 0x8 != 0 {
            ratio = (ratio * U128([0xffe5caca7e10f000, 0])) >> NUM_64
        };
        // i = 4
        if abs_tick & 0x10 != 0 {
            ratio = (ratio * U128([0xffcb9843d60f7000, 0])) >> NUM_64
        };
        // i = 5
        if abs_tick & 0x20 != 0 {
            ratio = (ratio * U128([0xff973b41fa98e800, 0])) >> NUM_64
        };
        // i = 6
        if abs_tick & 0x40 != 0 {
            ratio = (ratio * U128([0xff2ea16466c9b000, 0])) >> NUM_64
        };
        // i = 7
        if abs_tick & 0x80 != 0 {
            ratio = (ratio * U128([0xfe5dee046a9a3800, 0])) >> NUM_64
        };
        // i = 8
        if abs_tick & 0x100 != 0 {
            ratio = (ratio * U128([0xfcbe86c7900bb000, 0])) >> NUM_64
        };
        // i = 9
        if abs_tick & 0x200 != 0 {
            ratio = (ratio * U128([0xf987a7253ac65800, 0])) >> NUM_64
        };
        // i = 10
        if abs_tick & 0x400 != 0 {
            ratio = (ratio * U128([0xf3392b0822bb6000, 0])) >> NUM_64
        };
        // i = 11
        if abs_tick & 0x800 != 0 {
            ratio = (ratio * U128([0xe7159475a2caf000, 0])) >> NUM_64
        };
        // i = 12
        if abs_tick & 0x1000 != 0 {
            ratio = (ratio * U128([0xd097f3bdfd2f2000, 0])) >> NUM_64
        };
        // i = 13
        if abs_tick & 0x2000 != 0 {
            ratio = (ratio * U128([0xa9f746462d9f8000, 0])) >> NUM_64
        };
        // i = 14
        if abs_tick & 0x4000 != 0 {
            ratio = (ratio * U128([0x70d869a156f31c00, 0])) >> NUM_64
        };
        // i = 15
        if abs_tick & 0x8000 != 0 {
            ratio = (ratio * U128([0x31be135f97ed3200, 0])) >> NUM_64
        };
        // i = 16
        if abs_tick & 0x10000 != 0 {
            ratio = (ratio * U128([0x9aa508b5b85a500, 0])) >> NUM_64
        };
        // i = 17
        if abs_tick & 0x20000 != 0 {
            ratio = (ratio * U128([0x5d6af8dedc582c, 0])) >> NUM_64
        };
        // i = 18
        if abs_tick & 0x40000 != 0 {
            ratio = (ratio * U128([0x2216e584f5fa, 0])) >> NUM_64
        }

        // Divide to obtain 1.0001^(2^(i - 1)) * 2^32 in numerator
        if tick > 0 {
            ratio = U128::MAX / ratio;
        }

        ratio.as_u128()
    }

    pub fn tick_count(&self, tick_spacing: u16) -> i32 {
        tick_spacing as i32 * TICK_ARRAY_SIZE
    }

    pub fn get_array_start_index(&self, tick_index: i32, tick_spacing: u16) -> i32 {
        let ticks_in_array = self.tick_count(tick_spacing);
        let mut start = tick_index / ticks_in_array;
        if tick_index < 0 && tick_index % ticks_in_array != 0 {
            start = start - 1
        }
        start * ticks_in_array
    }

    pub fn max_tick_in_tickarray_bitmap(&self, tick_spacing: u16) -> i32 {
        tick_spacing as i32 * TICK_ARRAY_SIZE * TICK_ARRAY_BITMAP_SIZE
    }

    pub fn tick_array_start_index_range(&self, tick_spacing: u16) -> (i32, i32) {
        let mut max_tick_boundary = self.max_tick_in_tickarray_bitmap(tick_spacing);
        let mut min_tick_boundary = -max_tick_boundary;
        if max_tick_boundary > MAX_TICK {
            max_tick_boundary = self.get_array_start_index(MAX_TICK, tick_spacing);
            // find the next tick array start index
            max_tick_boundary = max_tick_boundary + self.tick_count(tick_spacing);
        }
        if min_tick_boundary < MIN_TICK {
            min_tick_boundary = self.get_array_start_index(MIN_TICK, tick_spacing);
        }
        (min_tick_boundary, max_tick_boundary)
    }

    pub fn is_overflow_default_tickarray_bitmap(&self, tick_index: i32, tick_spacing: u16) -> bool {
        let (min_tick_array_start_index_boundary, max_tick_array_index_boundary) =
            self.tick_array_start_index_range(tick_spacing);
        let tick_array_start_index = self.get_array_start_index(tick_index, tick_spacing);
        if tick_array_start_index >= max_tick_array_index_boundary
            || tick_array_start_index < min_tick_array_start_index_boundary
        {
            return true;
        }
        false
    }
    pub fn tick_array_offset_in_bitmap(
        &self,
        tick_array_start_index: i32,
        tick_spacing: u16,
    ) -> i32 {
        let m = tick_array_start_index.abs() % self.max_tick_in_tickarray_bitmap(tick_spacing);
        let mut tick_array_offset_in_bitmap = m / self.tick_count(tick_spacing);
        if tick_array_start_index < 0 && m != 0 {
            tick_array_offset_in_bitmap = TICK_ARRAY_BITMAP_SIZE - tick_array_offset_in_bitmap;
        }
        tick_array_offset_in_bitmap
    }

    fn count_continuous_zeros(&self, num: u64, bit_pos: u64) -> (i32, i32) {
        let mut left_count = 0;
        let mut right_count = 0;

        // 计算左边连续 0 的数量
        for i in bit_pos + 1..64 {
            if num & (1 << i) == 0 {
                left_count += 1;
            } else {
                break;
            }
        }

        // 计算右边连续 0 的数量
        for i in (0..bit_pos).rev() {
            if num & (1 << i) == 0 {
                right_count += 1;
            } else {
                break;
            }
        }

        return (left_count, right_count);
    }

    pub fn check_current_tick_array_is_initialized_for_ex_bitmap(
        &self,
        tick_current: i32,
        tick_spacing: u16,
    ) -> (bool, i32, i32, i32) {
        let ticks_in_one_bitmap = self.max_tick_in_tickarray_bitmap(tick_spacing);
        let tick_array_start_index = self.get_array_start_index(tick_current, tick_spacing);

        let mut offset = tick_array_start_index.abs() / ticks_in_one_bitmap - 1;
        if tick_array_start_index < 0 && tick_array_start_index.abs() % ticks_in_one_bitmap == 0 {
            offset -= 1;
        }
        let taget_bit_map_index = (offset * 64) as usize;

        let tick_array_offset_in_bitmap =
            self.tick_array_offset_in_bitmap(tick_array_start_index, tick_spacing);

        let target_bit_map_u64_index = (tick_array_offset_in_bitmap / 64) as usize;
        let bit_offset = tick_array_offset_in_bitmap % 64;

        let bit_map_data = self.ex_bit_map.data.borrow();

        let offset_if_negative: usize = if tick_current < 0 { 14 * 8 * 8 } else { 0 };
        let mut bit_data_buffer = [0u8; 8];
        bit_data_buffer.copy_from_slice(
            &bit_map_data[8
                + 32
                + offset_if_negative
                + taget_bit_map_index
                + target_bit_map_u64_index * 8
                ..8 + 32
                    + offset_if_negative
                    + taget_bit_map_index
                    + target_bit_map_u64_index * 8
                    + 8],
        );
        let bit_data = u64::from_le_bytes(bit_data_buffer);

        let mask = 1u64 << bit_offset;
        let masked = bit_data & mask;
        let initialized = masked != 0;
        if initialized {
            return (true, tick_array_start_index, 0, 0);
        } else {
            let max_continues = self.count_continuous_zeros(bit_data, bit_offset as u64);
            if tick_current < 0 {
                return (
                    false,
                    tick_array_start_index,
                    max_continues.0,
                    max_continues.1,
                );
            } else {
                return (
                    false,
                    tick_array_start_index,
                    max_continues.1,
                    max_continues.0,
                );
            }
        }
    }

    /// Given a tick, calculate whether the tickarray it belongs to has been initialized.
    pub fn check_current_tick_array_is_initialized(
        &self,
        bit_map: [u64; 16],
        tick_current: i32,
        tick_spacing: u16,
    ) -> (bool, i32, i32, i32) {
        let multiplier = i32::from(tick_spacing) * TICK_ARRAY_SIZE;
        let mut compressed = tick_current / multiplier + 512;
        if tick_current < 0 && tick_current % multiplier != 0 {
            compressed -= 1;
        }
        let bit_pos = compressed.abs();
        let word_shift = bit_pos / 64;
        let bit_shift = bit_pos % 64;
        let word = bit_map[word_shift as usize];
        let mask = 1u64 << bit_shift;
        let masked = word & mask;
        let initialized = masked != 0;
        if initialized {
            return (true, (compressed - 512) * multiplier, 0, 0);
        }
        // the current bit is not initialized
        let max_continues = self.count_continuous_zeros(word, bit_shift as u64);
        return (
            false,
            (compressed - 512) * multiplier,
            max_continues.1,
            max_continues.0,
        );
    }

    pub fn init(&mut self) {
        let pool_data = self.pool_state.data.borrow();
        let config_data = self.amm_config.data.borrow();

        let mut fee_numerator_buffer = [0u8; 4];
        fee_numerator_buffer.copy_from_slice(&config_data[8 + 3 + 32 + 4..8 + 3 + 32 + 4 + 4]);
        self.fee_numerator = u32::from_le_bytes(fee_numerator_buffer);

        let observation_state_offset = 9 + 32 + 32 + 32 + 32 + 32 + 32 + 32;

        let mut liquidity_buffer = [0u8; 16];
        liquidity_buffer.copy_from_slice(
            &pool_data[observation_state_offset + 4..observation_state_offset + 4 + 16],
        );
        self.liquidity = u128::from_le_bytes(liquidity_buffer);

        let mut sqrt_price_x64_buffer = [0u8; 16];
        sqrt_price_x64_buffer.copy_from_slice(
            &pool_data[observation_state_offset + 4 + 16..observation_state_offset + 4 + 16 + 16],
        );
        self.sqrt_price_x64 = u128::from_le_bytes(sqrt_price_x64_buffer);
        
        {
            // do lqp cqp hqp
            let bitmap_data = &pool_data[observation_state_offset + 164 + 507
                ..observation_state_offset + 164 + 507 + 16 * 8];
            let mut bit_map = [0u64; 16];
            for i in 0..16 {
                let mut buffer = [0u8; 8];
                buffer.copy_from_slice(&bitmap_data[i * 8..i * 8 + 8]);
                let value = u64::from_le_bytes(buffer);
                bit_map[i] = value;
            }

            let mut tick_spacing_buffer = [0u8; 2];
            tick_spacing_buffer.copy_from_slice(
                &pool_data[observation_state_offset + 2..observation_state_offset + 2 + 2],
            );
            let tick_spacing = u16::from_le_bytes(tick_spacing_buffer);

            let mut tick_current_buffer = [0u8; 4];
            tick_current_buffer.copy_from_slice(
                &pool_data[observation_state_offset + 2 + 2 + 32
                    ..observation_state_offset + 2 + 2 + 32 + 4],
            );
            let tick_current = i32::from_le_bytes(tick_current_buffer);
            let mut low_tick_index: i32 = 0;
            let mut hight_tick_index: i32 = 0;
            {
                if self.is_overflow_default_tickarray_bitmap(tick_current, tick_spacing) {
                    let (initialized, array_start_tick_index, low_max_offset, hight_max_offset) =
                        self.check_current_tick_array_is_initialized_for_ex_bitmap(
                            tick_current,
                            tick_spacing,
                        );
                    if !initialized {
                        low_tick_index = array_start_tick_index
                            - self.tick_count(tick_spacing) * low_max_offset
                            - 1;
                        hight_tick_index = array_start_tick_index
                            + self.tick_count(tick_spacing) * (hight_max_offset + 1);
                    } else {
                        let res = self.find_tick_state_low_hight(
                            array_start_tick_index,
                            tick_current,
                            tick_spacing,
                        );
                        low_tick_index = res.0;
                        hight_tick_index = res.1;
                    }
                } else {
                    let (initialized, array_start_tick_index, low_max_offset, hight_max_offset) =
                        self.check_current_tick_array_is_initialized(
                            bit_map,
                            tick_current,
                            tick_spacing,
                        );
                    if !initialized {
                        low_tick_index = array_start_tick_index
                            - self.tick_count(tick_spacing) * low_max_offset
                            - 1;
                        hight_tick_index = array_start_tick_index
                            + self.tick_count(tick_spacing) * (hight_max_offset + 1);
                    } else {
                        let res = self.find_tick_state_low_hight(
                            array_start_tick_index,
                            tick_current,
                            tick_spacing,
                        );
                        low_tick_index = res.0;
                        hight_tick_index = res.1;
                    }
                }

                if low_tick_index < MIN_TICK {
                    low_tick_index = MIN_TICK;
                }
                if hight_tick_index > MAX_TICK {
                    hight_tick_index = MAX_TICK;
                }

                // msg!(
                //     "low_tick_index:{} current:{} hight_tick_index:{} l:{} p:{}\n",
                //     low_tick_index, tick_current, hight_tick_index, self.liquidity, self.sqrt_price_x64
                // );
            }
            // field

            self.amount_x = U256::from(self.liquidity)
                .mul(U256::from(Q64))
                .div(U256::from(self.sqrt_price_x64))
                .as_u128();
            self.amount_y = U256::from(self.liquidity)
                .mul(U256::from(self.sqrt_price_x64))
                .div(U256::from(Q64))
                .as_u128();

            let low_tick_price_x64 = self.get_sqrt_price_at_tick_x64(low_tick_index);
            let hight_tick_price_x64 = self.get_sqrt_price_at_tick_x64(hight_tick_index);

            self.limit_in_x = self.get_delta_amount_0_unsigned(
                low_tick_price_x64,
                self.sqrt_price_x64,
                self.liquidity,
                true,
            );
            self.limit_in_y = self.get_delta_amount_1_unsigned(
                self.sqrt_price_x64,
                hight_tick_price_x64,
                self.liquidity,
                true,
            );
        }
        self.sqrt_price = self.sqrt_price_x64 as f64 / Q64 as f64;

        // msg!(
        //     "limit_in_x:{} limit_in_y:{} sqrt_price:{} l:{} p:{} f:{}\n",
        //     self.limit_in_x, self.limit_in_y, self.sqrt_price, self.liquidity, self.sqrt_price_x64, self.fee_numerator
        // );
    }
}

impl<'a, 'info> BaseMarketPool<'a, 'info> for RaydiumCLMMMarketPool<'a, 'info> {

    fn get_real_time_user_token_amount_x(&self) -> u64 {
        unpack_token_account_ammount(self.user_token_account_x).unwrap()
    }

    fn get_real_time_user_token_amount_y(&self) -> u64 {
        unpack_token_account_ammount(self.user_token_account_y).unwrap()
    }

    fn valid(&self) -> bool {
        self.liquidity > 0
    }

    fn liquidity(&self) -> u128 {
        self.liquidity
    }

    fn sqrt_price(&self) -> f64 {
        self.sqrt_price
    }

    #[cfg(feature = "debug-out")]
    fn out_x(&self, in_y: u64) -> u64 {
        let amount_remaining_less_fee = in_y.mul(self.fee_denominator() - self.fee_numerator()).div(self.fee_denominator());

        let next_sqrt_price = self.get_next_sqrt_price_from_input(self.sqrt_price_x64, self.liquidity, amount_remaining_less_fee, false);
        self.get_delta_amount_0_unsigned(
            self.sqrt_price_x64,
            next_sqrt_price,
            self.liquidity,
            false,
        )
    }

    #[cfg(feature = "debug-out")]
    fn out_y(&self, in_x: u64) -> u64 {
        let amount_remaining_less_fee = in_x.mul(self.fee_denominator() - self.fee_numerator()).div(self.fee_denominator());
        let next_sqrt_price = self.get_next_sqrt_price_from_input(self.sqrt_price_x64, self.liquidity, amount_remaining_less_fee, true);
        self.get_delta_amount_1_unsigned(
            next_sqrt_price,
            self.sqrt_price_x64,
            self.liquidity,
            false,
        )
    }

    fn limit_in_x(&self) -> u64 {
        self.limit_in_x
    }

    fn limit_in_y(&self) -> u64 {
        self.limit_in_y
    }

    fn x_mint(&self) -> &Pubkey {
        &self.token_x_mint.key
    }
    fn y_mint(&self) -> &Pubkey {
        &self.token_y_mint.key
    }
    fn fee_denominator(&self) -> u64 {
        FEE_RATE_DENOMINATOR_VALUE as u64
    }
    fn fee_numerator(&self) -> u64 {
        self.fee_numerator as u64
    }
    fn amount_x(&self) -> u64 {
        self.amount_x as u64
    }
    fn amount_y(&self) -> u64 {
        self.amount_y as u64
    }
    
    fn swap(&self, y2x: bool, amount_in: u64) -> Result<()> {
        let swap_data = match Vec::from_hex("2b04ed0b1ac91e62") {
            Ok(mut v) => {
                
                let other_amount_threshold:u64 = 0;
                let sqrt_price_limit_x64:u128 = 0;
                let base_in = 1u8;
                v.append(borsh::to_vec(&amount_in).unwrap().as_mut());
                v.append(borsh::to_vec(&other_amount_threshold).unwrap().as_mut());
                v.append(borsh::to_vec(&sqrt_price_limit_x64).unwrap().as_mut());
                v.append(borsh::to_vec(&base_in).unwrap().as_mut());
                v
            },
            Err(e) => {
                msg!("Error decoding hex: {:?}", e);
                return Err(error!(MyErrorCode::InvalidTokenAccount));
            }
        };
        let mut accounts_meta = vec![
            AccountMeta::new(self.user.key(), true), // payer
            AccountMeta::new_readonly(self.amm_config.key(), false), // amm config
            AccountMeta::new(self.pool_state.key(), false), // pool state
            AccountMeta::new(match y2x {
                true => self.user_token_account_y.key(),
                false => self.user_token_account_x.key(),
            }, false), // user input
            AccountMeta::new(match y2x {
                true => self.user_token_account_x.key(),
                false => self.user_token_account_y.key(),
            }, false), // user output
            AccountMeta::new(match y2x {
                true => self.pool_token_account_y.key(),
                false => self.pool_token_account_x.key(),
            }, false),
            AccountMeta::new(match y2x {
                true => self.pool_token_account_x.key(),
                false => self.pool_token_account_y.key(),
            }, false),
            AccountMeta::new(self.observation_state.key(), false),
            AccountMeta::new_readonly(self.token_program.key(), false),
            AccountMeta::new_readonly(self.token_program_2022.key(), false),
            AccountMeta::new_readonly(self.memo_program.key(), false),
            AccountMeta::new_readonly(match y2x {
                true => self.token_y_mint.key(),
                false => self.token_x_mint.key(),
            }, false),
            AccountMeta::new_readonly(match y2x {
                true => self.token_x_mint.key(),
                false => self.token_y_mint.key(),
            }, false),
            AccountMeta::new(self.ex_bit_map.key(), false),
          ];

        

        let mut accounts_info = vec![
            self.user.to_account_info(),
            self.amm_config.to_account_info(),
            self.pool_state.to_account_info(),
            match y2x {
                true => self.user_token_account_y.to_account_info(),
                false => self.user_token_account_x.to_account_info(),
            },
            match y2x {
                true => self.user_token_account_x.to_account_info(),
                false => self.user_token_account_y.to_account_info(),
            },
            match y2x {
                true => self.pool_token_account_y.to_account_info(),
                false => self.pool_token_account_x.to_account_info(),
            },
            match y2x {
                true => self.pool_token_account_x.to_account_info(),
                false => self.pool_token_account_y.to_account_info(),
            },
            self.observation_state.to_account_info(),
            self.token_program.to_account_info(),
            self.token_program_2022.to_account_info(),
            self.memo_program.to_account_info(),
            match y2x {
                true => self.token_y_mint.to_account_info(),
                false => self.token_x_mint.to_account_info(),
            },
            match y2x {
                true => self.token_x_mint.to_account_info(),
                false => self.token_y_mint.to_account_info(),
            },
            self.ex_bit_map.to_account_info(),
        ];
        for account in self.tick_array_vec.iter() {
            accounts_meta.push(AccountMeta::new(account.key(), false));
            accounts_info.push(account.to_account_info());
        }

        let swap_ix = Instruction {
          program_id: self.program.key(),
          accounts: accounts_meta,
          data: swap_data,
        };
        invoke(&swap_ix, &accounts_info)?;
        Ok(())
    }
    fn market_type(&self) -> MarketType {
        MarketType::NormalCLMM
    }
}

impl<'a, 'info> CreateMarket<'a, 'info> for RaydiumCLMMMarketPool<'a, 'info> {
    fn create_market(
        base: &'a crate::model::base_model::BaseModel<'a, 'info>,
        min_x_user_account: &'a Option<UncheckedAccount<'info>>,
        mint_x_mint_account: &'a Option<UncheckedAccount<'info>>,
        accounts_iter: Rc<RefCell<AccountsIter<'a, 'info>>>,
        reverse: bool,
    ) -> Box<dyn BaseMarketPool<'a, 'info> + 'a> {
        let accounts = accounts_iter.borrow_mut().take(12);

        let mut tick_array_vec: Vec<&'a AccountInfo<'info>> = Vec::new();
        for i in 9..12 {
            if accounts[i].is_some() {
                let account = accounts[i].as_ref().unwrap();
                tick_array_vec.push(account);   
            }
        }
        let mut meteora = Box::new(RaydiumCLMMMarketPool {
            program: accounts[0].as_ref().unwrap(),
            amm_config: accounts[1].as_ref().unwrap(),
            pool_state: accounts[2].as_ref().unwrap(),
            pool_token_account_x: accounts[3].as_ref().unwrap(),
            pool_token_account_y: accounts[4].as_ref().unwrap(),
            observation_state: accounts[5].as_ref().unwrap(),
            token_program_2022: accounts[6].as_ref().unwrap(),
            memo_program: accounts[7].as_ref().unwrap(),
            ex_bit_map: accounts[8].as_ref().unwrap(),
            tick_array_vec: tick_array_vec,
            
            user_token_account_x: match reverse {
                true => base.user_token_base,
                false => min_x_user_account.as_ref().unwrap(),
            },
            user_token_account_y: match reverse {
                true => min_x_user_account.as_ref().unwrap(),
                false => base.user_token_base,
            },
            token_x_mint: match reverse {
                true => base.token_base_mint,
                false => mint_x_mint_account.as_ref().unwrap(),
            },
            token_y_mint: match reverse {
                true => mint_x_mint_account.as_ref().unwrap(),
                false => base.token_base_mint,
            },

            token_program: base.token_program,
            user: base.user,

            fee_numerator: 0,
            amount_x: 0,
            amount_y: 0,
            liquidity: 0,
            sqrt_price_x64: 0,
            limit_in_x: 0,
            limit_in_y: 0,
            sqrt_price: 0.0,
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
