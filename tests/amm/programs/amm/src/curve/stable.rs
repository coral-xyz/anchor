//! The curve.fi invariant calculator.
use {
    crate::{
        curve::calculator::{
            CurveCalculator, DynPack, RoundDirection, SwapWithoutFeesResult, TradeDirection,
            TradingTokenResult,
        },
        SwapError,
    },
    arrayref::{array_mut_ref, array_ref},
    solana_program::{
        program_error::ProgramError,
        program_pack::{IsInitialized, Pack, Sealed},
    },
    spl_math::{precise_number::PreciseNumber, uint::U256},
    std::convert::TryFrom,
};

const N_COINS: u8 = 2;
const N_COINS_SQUARED: u8 = 4;
const ITERATIONS: u8 = 32;

/// Returns self to the power of b
fn checked_u8_power(a: &U256, b: u8) -> Option<U256> {
    let mut result = *a;
    for _ in 1..b {
        result = result.checked_mul(*a)?;
    }
    Some(result)
}

/// Returns self multiplied by b
fn checked_u8_mul(a: &U256, b: u8) -> Option<U256> {
    let mut result = *a;
    for _ in 1..b {
        result = result.checked_add(*a)?;
    }
    Some(result)
}

/// StableCurve struct implementing CurveCalculator
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StableCurve {
    /// Amplifier constant
    pub amp: u64,
}

/// d = (leverage * sum_x + d_product * n_coins) * initial_d / ((leverage - 1) * initial_d + (n_coins + 1) * d_product)
fn calculate_step(initial_d: &U256, leverage: u64, sum_x: u128, d_product: &U256) -> Option<U256> {
    let leverage_mul = U256::from(leverage).checked_mul(sum_x.into())?;
    let d_p_mul = checked_u8_mul(d_product, N_COINS)?;

    let l_val = leverage_mul.checked_add(d_p_mul)?.checked_mul(*initial_d)?;

    let leverage_sub = initial_d.checked_mul((leverage.checked_sub(1)?).into())?;
    let n_coins_sum = checked_u8_mul(d_product, N_COINS.checked_add(1)?)?;

    let r_val = leverage_sub.checked_add(n_coins_sum)?;

    l_val.checked_div(r_val)
}

/// Compute stable swap invariant (D)
/// Equation:
/// A * sum(x_i) * n**n + D = A * D * n**n + D**(n+1) / (n**n * prod(x_i))
fn compute_d(leverage: u64, amount_a: u128, amount_b: u128) -> Option<u128> {
    let amount_a_times_coins =
        checked_u8_mul(&U256::from(amount_a), N_COINS)?.checked_add(U256::one())?;
    let amount_b_times_coins =
        checked_u8_mul(&U256::from(amount_b), N_COINS)?.checked_add(U256::one())?;
    let sum_x = amount_a.checked_add(amount_b)?; // sum(x_i), a.k.a S
    if sum_x == 0 {
        Some(0)
    } else {
        let mut d_previous: U256;
        let mut d: U256 = sum_x.into();

        // Newton's method to approximate D
        for _ in 0..ITERATIONS {
            let mut d_product = d;
            d_product = d_product
                .checked_mul(d)?
                .checked_div(amount_a_times_coins)?;
            d_product = d_product
                .checked_mul(d)?
                .checked_div(amount_b_times_coins)?;
            d_previous = d;
            //d = (leverage * sum_x + d_p * n_coins) * d / ((leverage - 1) * d + (n_coins + 1) * d_p);
            d = calculate_step(&d, leverage, sum_x, &d_product)?;
            // Equality with the precision of 1
            if d == d_previous {
                break;
            }
        }
        u128::try_from(d).ok()
    }
}

/// Compute swap amount `y` in proportion to `x`
/// Solve for y:
/// y**2 + y * (sum' - (A*n**n - 1) * D / (A * n**n)) = D ** (n + 1) / (n ** (2 * n) * prod' * A)
/// y**2 + b*y = c
fn compute_new_destination_amount(
    leverage: u64,
    new_source_amount: u128,
    d_val: u128,
) -> Option<u128> {
    // Upscale to U256
    let leverage: U256 = leverage.into();
    let new_source_amount: U256 = new_source_amount.into();
    let d_val: U256 = d_val.into();

    // sum' = prod' = x
    // c =  D ** (n + 1) / (n ** (2 * n) * prod' * A)
    let c = checked_u8_power(&d_val, N_COINS.checked_add(1)?)?
        .checked_div(checked_u8_mul(&new_source_amount, N_COINS_SQUARED)?.checked_mul(leverage)?)?;

    // b = sum' - (A*n**n - 1) * D / (A * n**n)
    let b = new_source_amount.checked_add(d_val.checked_div(leverage)?)?;

    // Solve for y by approximating: y**2 + b*y = c
    let mut y_prev: U256;
    let mut y = d_val;
    for _ in 0..ITERATIONS {
        y_prev = y;
        y = (checked_u8_power(&y, 2)?.checked_add(c)?)
            .checked_div(checked_u8_mul(&y, 2)?.checked_add(b)?.checked_sub(d_val)?)?;
        if y == y_prev {
            break;
        }
    }
    u128::try_from(y).ok()
}

impl CurveCalculator for StableCurve {
    /// Stable curve
    fn swap_without_fees(
        &self,
        source_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
        _trade_direction: TradeDirection,
    ) -> Option<SwapWithoutFeesResult> {
        let leverage = self.amp.checked_mul(N_COINS as u64)?;

        let new_source_amount = swap_source_amount.checked_add(source_amount)?;
        let new_destination_amount = compute_new_destination_amount(
            leverage,
            new_source_amount,
            compute_d(leverage, swap_source_amount, swap_destination_amount)?,
        )?;

        let amount_swapped = swap_destination_amount.checked_sub(new_destination_amount)?;

        Some(SwapWithoutFeesResult {
            source_amount_swapped: source_amount,
            destination_amount_swapped: amount_swapped,
        })
    }

    /// Re-implementation of `remove_liquidty`:
    ///
    /// <https://github.com/curvefi/curve-contract/blob/80bbe179083c9a7062e4c482b0be3bfb7501f2bd/contracts/pool-templates/base/SwapTemplateBase.vy#L513>
    fn pool_tokens_to_trading_tokens(
        &self,
        pool_tokens: u128,
        pool_token_supply: u128,
        swap_token_a_amount: u128,
        swap_token_b_amount: u128,
        round_direction: RoundDirection,
    ) -> Option<TradingTokenResult> {
        let mut token_a_amount = pool_tokens
            .checked_mul(swap_token_a_amount)?
            .checked_div(pool_token_supply)?;
        let mut token_b_amount = pool_tokens
            .checked_mul(swap_token_b_amount)?
            .checked_div(pool_token_supply)?;
        let (token_a_amount, token_b_amount) = match round_direction {
            RoundDirection::Floor => (token_a_amount, token_b_amount),
            RoundDirection::Ceiling => {
                let token_a_remainder = pool_tokens
                    .checked_mul(swap_token_a_amount)?
                    .checked_rem(pool_token_supply)?;

                if token_a_remainder > 0 && token_a_amount > 0 {
                    token_a_amount += 1;
                }
                let token_b_remainder = pool_tokens
                    .checked_mul(swap_token_b_amount)?
                    .checked_rem(pool_token_supply)?;
                if token_b_remainder > 0 && token_b_amount > 0 {
                    token_b_amount += 1;
                }
                (token_a_amount, token_b_amount)
            }
        };
        Some(TradingTokenResult {
            token_a_amount,
            token_b_amount,
        })
    }

    /// Get the amount of pool tokens for the given amount of token A or B.
    /// Re-implementation of `calc_token_amount`:
    ///
    /// <https://github.com/curvefi/curve-contract/blob/80bbe179083c9a7062e4c482b0be3bfb7501f2bd/contracts/pool-templates/base/SwapTemplateBase.vy#L267>
    fn deposit_single_token_type(
        &self,
        source_amount: u128,
        swap_token_a_amount: u128,
        swap_token_b_amount: u128,
        pool_supply: u128,
        trade_direction: TradeDirection,
    ) -> Option<u128> {
        if source_amount == 0 {
            return Some(0);
        }
        let leverage = self.amp.checked_mul(N_COINS as u64)?;
        let d0 = PreciseNumber::new(compute_d(
            leverage,
            swap_token_a_amount,
            swap_token_b_amount,
        )?)?;
        let (deposit_token_amount, other_token_amount) = match trade_direction {
            TradeDirection::AtoB => (swap_token_a_amount, swap_token_b_amount),
            TradeDirection::BtoA => (swap_token_b_amount, swap_token_a_amount),
        };
        let updated_deposit_token_amount = deposit_token_amount.checked_add(source_amount)?;
        let d1 = PreciseNumber::new(compute_d(
            leverage,
            updated_deposit_token_amount,
            other_token_amount,
        )?)?;
        let diff = d1.checked_sub(&d0)?;
        let final_amount =
            (diff.checked_mul(&PreciseNumber::new(pool_supply)?))?.checked_div(&d0)?;
        final_amount.floor()?.to_imprecise()
    }

    fn withdraw_single_token_type_exact_out(
        &self,
        source_amount: u128,
        swap_token_a_amount: u128,
        swap_token_b_amount: u128,
        pool_supply: u128,
        trade_direction: TradeDirection,
    ) -> Option<u128> {
        if source_amount == 0 {
            return Some(0);
        }
        let leverage = self.amp.checked_mul(N_COINS as u64)?;
        let d0 = PreciseNumber::new(compute_d(
            leverage,
            swap_token_a_amount,
            swap_token_b_amount,
        )?)?;
        let (withdraw_token_amount, other_token_amount) = match trade_direction {
            TradeDirection::AtoB => (swap_token_a_amount, swap_token_b_amount),
            TradeDirection::BtoA => (swap_token_b_amount, swap_token_a_amount),
        };
        let updated_deposit_token_amount = withdraw_token_amount.checked_sub(source_amount)?;
        let d1 = PreciseNumber::new(compute_d(
            leverage,
            updated_deposit_token_amount,
            other_token_amount,
        )?)?;
        let diff = d0.checked_sub(&d1)?;
        let final_amount =
            (diff.checked_mul(&PreciseNumber::new(pool_supply)?))?.checked_div(&d0)?;
        final_amount.ceiling()?.to_imprecise()
    }

    fn normalized_value(
        &self,
        swap_token_a_amount: u128,
        swap_token_b_amount: u128,
    ) -> Option<PreciseNumber> {
        #[cfg(not(any(test, feature = "fuzz")))]
        {
            let leverage = self.amp.checked_mul(N_COINS as u64)?;
            PreciseNumber::new(compute_d(
                leverage,
                swap_token_a_amount,
                swap_token_b_amount,
            )?)
        }
        #[cfg(any(test, feature = "fuzz"))]
        {
            use roots::{find_roots_cubic_normalized, Roots};
            let x = swap_token_a_amount as f64;
            let y = swap_token_b_amount as f64;
            let c = (4.0 * (self.amp as f64)) - 1.0;
            let d = 16.0 * (self.amp as f64) * x * y * (x + y);
            let roots = find_roots_cubic_normalized(0.0, c, d);
            let x0 = match roots {
                Roots::No(_) => panic!("No roots found for cubic equations"),
                Roots::One(x) => x[0],
                Roots::Two(_) => panic!("Two roots found for cubic, mathematically impossible"),
                Roots::Three(x) => x[1],
                Roots::Four(_) => panic!("Four roots found for cubic, mathematically impossible"),
            };

            let root_uint = (x0 * ((10f64).powf(11.0))).round() as u128;
            let precision = PreciseNumber::new(10)?.checked_pow(11)?;
            let two = PreciseNumber::new(2)?;
            PreciseNumber::new(root_uint)?
                .checked_div(&precision)?
                .checked_div(&two)
        }
    }

    fn validate(&self) -> Result<(), SwapError> {
        // TODO are all amps valid?
        Ok(())
    }
}

/// IsInitialized is required to use `Pack::pack` and `Pack::unpack`
impl IsInitialized for StableCurve {
    fn is_initialized(&self) -> bool {
        true
    }
}
impl Sealed for StableCurve {}
impl Pack for StableCurve {
    const LEN: usize = 8;
    fn pack_into_slice(&self, output: &mut [u8]) {
        (self as &dyn DynPack).pack_into_slice(output);
    }

    fn unpack_from_slice(input: &[u8]) -> Result<StableCurve, ProgramError> {
        let amp = array_ref![input, 0, 8];
        Ok(Self {
            amp: u64::from_le_bytes(*amp),
        })
    }
}

impl DynPack for StableCurve {
    fn pack_into_slice(&self, output: &mut [u8]) {
        let amp = array_mut_ref![output, 0, 8];
        *amp = self.amp.to_le_bytes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::calculator::{
        test::{
            check_curve_value_from_swap, check_deposit_token_conversion,
            check_pool_value_from_deposit, check_pool_value_from_withdraw,
            check_withdraw_token_conversion, total_and_intermediate,
            CONVERSION_BASIS_POINTS_GUARANTEE,
        },
        RoundDirection, INITIAL_SWAP_POOL_AMOUNT,
    };
    use proptest::prelude::*;
    use sim::StableSwapModel;

    #[test]
    fn initial_pool_amount() {
        let amp = 1;
        let calculator = StableCurve { amp };
        assert_eq!(calculator.new_pool_supply(), INITIAL_SWAP_POOL_AMOUNT);
    }

    fn check_pool_token_rate(
        token_a: u128,
        token_b: u128,
        deposit: u128,
        supply: u128,
        expected_a: u128,
        expected_b: u128,
    ) {
        let amp = 1;
        let calculator = StableCurve { amp };
        let results = calculator
            .pool_tokens_to_trading_tokens(
                deposit,
                supply,
                token_a,
                token_b,
                RoundDirection::Ceiling,
            )
            .unwrap();
        assert_eq!(results.token_a_amount, expected_a);
        assert_eq!(results.token_b_amount, expected_b);
    }

    #[test]
    fn trading_token_conversion() {
        check_pool_token_rate(2, 49, 5, 10, 1, 25);
        check_pool_token_rate(100, 202, 5, 101, 5, 10);
        check_pool_token_rate(5, 501, 2, 10, 1, 101);
    }

    proptest! {
        #[test]
        fn constant_product_swap_no_fee(
            swap_source_amount in 100..1_000_000_000_000_000_000u128,
            swap_destination_amount in 100..1_000_000_000_000_000_000u128,
            source_amount in 100..100_000_000_000u128,
            amp in 1..150u64
        ) {
            prop_assume!(source_amount < swap_source_amount);

            let curve = StableCurve { amp };

            let model: StableSwapModel = StableSwapModel::new(
                curve.amp.into(),
                vec![swap_source_amount, swap_destination_amount],
                N_COINS,
            );

            let result = curve.swap_without_fees(
                source_amount,
                swap_source_amount,
                swap_destination_amount,
                TradeDirection::AtoB,
            );

            let result = result.unwrap();
            let sim_result = model.sim_exchange(0, 1, source_amount);

            let diff =
                (sim_result as i128 - result.destination_amount_swapped as i128).abs();

            let tolerance = std::cmp::max(1, sim_result as i128 / 1_000_000_000);

            assert!(
                diff <= tolerance,
                "result={}, sim_result={}, amp={}, source_amount={}, swap_source_amount={}, swap_destination_amount={}, diff={}",
                result.destination_amount_swapped,
                sim_result,
                amp,
                source_amount,
                swap_source_amount,
                swap_destination_amount,
                diff
            );
        }
    }

    #[test]
    fn pack_curve() {
        let amp = 1;
        let curve = StableCurve { amp };

        let mut packed = [0u8; StableCurve::LEN];
        Pack::pack_into_slice(&curve, &mut packed[..]);
        let unpacked = StableCurve::unpack(&packed).unwrap();
        assert_eq!(curve, unpacked);

        let mut packed = vec![];
        packed.extend_from_slice(&amp.to_le_bytes());
        let unpacked = StableCurve::unpack(&packed).unwrap();
        assert_eq!(curve, unpacked);
    }

    proptest! {
        #[test]
        fn curve_value_does_not_decrease_from_deposit(
            pool_token_amount in 1..u64::MAX,
            pool_token_supply in 1..u64::MAX,
            swap_token_a_amount in 1..u64::MAX,
            swap_token_b_amount in 1..u64::MAX,
            amp in 1..100,
        ) {
            let pool_token_amount = pool_token_amount as u128;
            let pool_token_supply = pool_token_supply as u128;
            let swap_token_a_amount = swap_token_a_amount as u128;
            let swap_token_b_amount = swap_token_b_amount as u128;
            // Make sure we will get at least one trading token out for each
            // side, otherwise the calculation fails
            prop_assume!(pool_token_amount * swap_token_a_amount / pool_token_supply >= 1);
            prop_assume!(pool_token_amount * swap_token_b_amount / pool_token_supply >= 1);
            let curve = StableCurve {
                amp: amp as u64
            };
            check_pool_value_from_deposit(
                &curve,
                pool_token_amount,
                pool_token_supply,
                swap_token_a_amount,
                swap_token_b_amount,
            );
        }
    }

    proptest! {
        #[test]
        fn curve_value_does_not_decrease_from_withdraw(
            (pool_token_supply, pool_token_amount) in total_and_intermediate(),
            swap_token_a_amount in 1..u64::MAX,
            swap_token_b_amount in 1..u64::MAX,
            amp in 1..100,
        ) {
            let pool_token_amount = pool_token_amount as u128;
            let pool_token_supply = pool_token_supply as u128;
            let swap_token_a_amount = swap_token_a_amount as u128;
            let swap_token_b_amount = swap_token_b_amount as u128;
            // Make sure we will get at least one trading token out for each
            // side, otherwise the calculation fails
            prop_assume!(pool_token_amount * swap_token_a_amount / pool_token_supply >= 1);
            prop_assume!(pool_token_amount * swap_token_b_amount / pool_token_supply >= 1);
            let curve = StableCurve {
                amp: amp as u64
            };
            check_pool_value_from_withdraw(
                &curve,
                pool_token_amount,
                pool_token_supply,
                swap_token_a_amount,
                swap_token_b_amount,
            );
        }
    }

    proptest! {
        #[test]
        fn curve_value_does_not_decrease_from_swap(
            source_token_amount in 1..u64::MAX,
            swap_source_amount in 1..u64::MAX,
            swap_destination_amount in 1..u64::MAX,
            amp in 1..100,
        ) {
            let curve = StableCurve { amp: amp as u64 };
            check_curve_value_from_swap(
                &curve,
                source_token_amount as u128,
                swap_source_amount as u128,
                swap_destination_amount as u128,
                TradeDirection::AtoB
            );
        }
    }

    proptest! {
        #[test]
        fn deposit_token_conversion(
            // in the pool token conversion calcs, we simulate trading half of
            // source_token_amount, so this needs to be at least 2
            source_token_amount in 2..u64::MAX,
            swap_source_amount in 1..u64::MAX,
            swap_destination_amount in 2..u64::MAX,
            pool_supply in INITIAL_SWAP_POOL_AMOUNT..u64::MAX as u128,
            amp in 1..100u64,
        ) {
            let curve = StableCurve { amp };
            check_deposit_token_conversion(
                &curve,
                source_token_amount as u128,
                swap_source_amount as u128,
                swap_destination_amount as u128,
                TradeDirection::AtoB,
                pool_supply,
                CONVERSION_BASIS_POINTS_GUARANTEE * 100,
            );

            check_deposit_token_conversion(
                &curve,
                source_token_amount as u128,
                swap_source_amount as u128,
                swap_destination_amount as u128,
                TradeDirection::BtoA,
                pool_supply,
                CONVERSION_BASIS_POINTS_GUARANTEE * 100,
            );
        }
    }

    proptest! {
        #[test]
        fn withdraw_token_conversion(
            (pool_token_supply, pool_token_amount) in total_and_intermediate(),
            swap_token_a_amount in 1..u64::MAX,
            swap_token_b_amount in 1..u64::MAX,
            amp in 1..100u64,
        ) {
            let curve = StableCurve { amp };
            check_withdraw_token_conversion(
                &curve,
                pool_token_amount as u128,
                pool_token_supply as u128,
                swap_token_a_amount as u128,
                swap_token_b_amount as u128,
                TradeDirection::AtoB,
                CONVERSION_BASIS_POINTS_GUARANTEE
            );
            check_withdraw_token_conversion(
                &curve,
                pool_token_amount as u128,
                pool_token_supply as u128,
                swap_token_a_amount as u128,
                swap_token_b_amount as u128,
                TradeDirection::BtoA,
                CONVERSION_BASIS_POINTS_GUARANTEE
            );
        }
    }
}
