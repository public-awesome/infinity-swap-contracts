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
    spot_price.checked_add(delta).map_err(|e| e.into())
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
    total_nfts: Uint128,
) -> Result<Uint128, ContractError> {
    ensure!(
        !total_nfts.is_zero(),
        ContractError::InvalidPair("pair must have at least 1 NFT".to_string(),)
    );
    Ok(total_tokens.checked_div(total_nfts + Uint128::one())?)
}

pub fn calc_cp_trade_buy_from_pair_price(
    total_tokens: Uint128,
    total_nfts: Uint128,
) -> Result<Uint128, ContractError> {
    ensure!(
        total_nfts > Uint128::one(),
        ContractError::InvalidPair("pair must have greater than 1 NFT".to_string(),)
    );
    let fraction = (total_nfts - Uint128::one(), Uint128::one());
    Ok(total_tokens.checked_mul_ceil(fraction)?)
}
