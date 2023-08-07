use crate::{
    pair::Pair,
    state::{PAIR_CONFIG, PAIR_IMMUTABLE, PAIR_INTERNAL},
    ContractError,
};

use cosmwasm_std::{ensure_eq, Addr, QuerierWrapper, Storage};
use infinity_shared::InfinityError;

pub fn only_pair_owner(sender: &Addr, pair: &Pair) -> Result<(), ContractError> {
    ensure_eq!(
        sender,
        &pair.immutable.owner,
        InfinityError::Unauthorized("sender is not the owner of the pair".to_string())
    );
    Ok(())
}

pub fn only_active(pair: &Pair) -> Result<(), ContractError> {
    ensure_eq!(
        pair.config.is_active,
        true,
        ContractError::InvalidPair("pair is inactive".to_string())
    );
    Ok(())
}

pub fn load_pair(
    contract: &Addr,
    storage: &dyn Storage,
    querier: &QuerierWrapper,
) -> Result<Pair, ContractError> {
    let immutable = PAIR_IMMUTABLE.load(storage)?;
    let config = PAIR_CONFIG.load(storage)?;
    let internal = PAIR_INTERNAL.load(storage)?;
    let total_tokens = querier.query_balance(contract, immutable.denom.clone())?.amount;
    Ok(Pair::new(immutable, config, internal, total_tokens))
}
