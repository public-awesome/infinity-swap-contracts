use crate::{pool::Pool, state::POOL_CONFIG, ContractError};

use cosmwasm_std::{Addr, QuerierWrapper, Storage};
use sg_std::NATIVE_DENOM;

pub fn load_pool(
    contract: &Addr,
    storage: &dyn Storage,
    querier: &QuerierWrapper,
) -> Result<Pool, ContractError> {
    let pool_config = POOL_CONFIG.load(storage)?;
    let total_tokens = querier.query_balance(contract, NATIVE_DENOM)?.amount;
    Ok(Pool::new(pool_config, total_tokens))
}
