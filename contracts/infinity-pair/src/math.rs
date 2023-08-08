use crate::ContractError;

use cosmwasm_std::{ensure, Decimal, Uint128};

pub fn calc_linear_spot_price_user_submits_nft(
    spot_price: Uint128,
    delta: Uint128,
) -> Result<Uint128, ContractError> {
    Ok(spot_price.checked_sub(delta)?)
}

pub fn calc_linear_spot_price_user_submits_tokens(
    spot_price: Uint128,
    delta: Uint128,
) -> Result<Uint128, ContractError> {
    Ok(spot_price.checked_add(delta)?)
}

pub fn calc_exponential_spot_price_user_submits_nft(
    spot_price: Uint128,
    delta: Decimal,
) -> Result<Uint128, ContractError> {
    let net_delta = Decimal::one().checked_div(Decimal::one().checked_add(delta)?)?;
    Ok(spot_price.mul_floor(net_delta))
}

pub fn calc_exponential_spot_price_user_submits_tokens(
    spot_price: Uint128,
    delta: Decimal,
) -> Result<Uint128, ContractError> {
    Ok(spot_price.mul_ceil(Decimal::one().checked_add(delta)?))
}

pub fn calc_linear_trade_buy_from_pair_price(
    spot_price: Uint128,
    delta: Uint128,
) -> Result<Uint128, ContractError> {
    Ok(spot_price.checked_add(delta)?)
}

pub fn calc_exponential_trade_buy_from_pair_price(
    spot_price: Uint128,
    delta: Decimal,
) -> Result<Uint128, ContractError> {
    Ok(spot_price.checked_mul_ceil(Decimal::one() + delta)?)
}

pub fn calc_cp_trade_sell_to_pair_price(
    total_tokens: Uint128,
    total_nfts: u64,
) -> Result<Uint128, ContractError> {
    ensure!(
        total_nfts != 0u64,
        ContractError::InvalidPair("pair must have at least 1 NFT".to_string(),)
    );
    let fraction = (Uint128::from(total_nfts + 1u64), Uint128::one());
    Ok(total_tokens.checked_div_ceil(fraction)?)
}

pub fn calc_cp_trade_buy_from_pair_price(
    total_tokens: Uint128,
    total_nfts: u64,
) -> Result<Uint128, ContractError> {
    ensure!(
        total_nfts > 1u64,
        ContractError::InvalidPair("pair must have greater than 1 NFT".to_string(),)
    );
    let fraction = (Uint128::from(total_nfts - 1u64), Uint128::one());
    Ok(total_tokens.checked_div_floor(fraction)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::{Decimal, Uint128};

    #[test]
    fn try_calc_linear_spot_price() {
        let spot_price = Uint128::from(250_000_000u128);
        let delta = Uint128::from(10_000_000u128);
        let spot_price_user_submits_nft =
            calc_linear_spot_price_user_submits_nft(spot_price, delta).unwrap();
        assert_eq!(spot_price_user_submits_nft, Uint128::from(240_000_000u128));

        let spot_price_user_submits_tokens =
            calc_linear_spot_price_user_submits_tokens(spot_price_user_submits_nft, delta).unwrap();
        assert_eq!(spot_price_user_submits_tokens, spot_price);
    }

    #[test]
    fn try_calc_exponential_spot_price() {
        let spot_price = Uint128::from(250_000_000u128);
        let delta = Decimal::percent(2);
        let spot_price_user_submits_nft =
            calc_exponential_spot_price_user_submits_nft(spot_price, delta).unwrap();
        assert_eq!(spot_price_user_submits_nft, Uint128::from(245_098_039u128));

        let spot_price_user_submits_tokens =
            calc_exponential_spot_price_user_submits_tokens(spot_price_user_submits_nft, delta)
                .unwrap();
        assert_eq!(spot_price_user_submits_tokens, spot_price);
    }

    #[test]
    fn try_calc_linear_trade_buy_from_pair_price() {
        let spot_price = Uint128::from(250_000_000u128);
        let delta = Uint128::from(10_000_000u128);
        let buy_from_pair_price = calc_linear_trade_buy_from_pair_price(spot_price, delta).unwrap();
        assert_eq!(buy_from_pair_price, Uint128::from(260_000_000u128));

        let buy_from_pair_price =
            calc_linear_trade_buy_from_pair_price(buy_from_pair_price, delta).unwrap();
        assert_eq!(buy_from_pair_price, Uint128::from(270_000_000u128));
    }

    #[test]
    fn try_calc_exponential_trade_buy_from_pair_price() {
        let spot_price = Uint128::from(250_000_000u128);
        let delta = Decimal::percent(2);
        let buy_from_pair_price =
            calc_exponential_trade_buy_from_pair_price(spot_price, delta).unwrap();
        assert_eq!(buy_from_pair_price, Uint128::from(255_000_000u128));

        let buy_from_pair_price =
            calc_exponential_trade_buy_from_pair_price(buy_from_pair_price, delta).unwrap();
        assert_eq!(buy_from_pair_price, Uint128::from(260_100_000u128));
    }

    #[test]
    fn try_calc_cp_trade_prices() {
        let result = calc_cp_trade_sell_to_pair_price(Uint128::from(250_000_000u128), 0u64);
        assert!(result.is_err());

        let sell_to_pair_price =
            calc_cp_trade_sell_to_pair_price(Uint128::from(250_000_000u128), 20u64).unwrap();
        assert_eq!(sell_to_pair_price, Uint128::from(11_904_762u128));

        let result = calc_cp_trade_buy_from_pair_price(Uint128::from(250_000_000u128), 1u64);
        assert!(result.is_err());

        let buy_from_pair_price =
            calc_cp_trade_buy_from_pair_price(Uint128::from(250_000_000u128), 20u64).unwrap();
        assert_eq!(buy_from_pair_price, Uint128::from(13_157_894u128));
    }
}
