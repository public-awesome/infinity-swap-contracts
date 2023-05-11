use crate::{
    pool::{EscrowNft, EscrowToken, NftPool, Pool, TokenPool, TradePool},
    state::{PoolType, POOL_CONFIG},
    ContractError,
};

use cosmwasm_std::{Addr, QuerierWrapper, Storage};
use sg_std::NATIVE_DENOM;

pub fn load_pool(
    contract: &Addr,
    storage: &dyn Storage,
    querier: &QuerierWrapper,
) -> Result<Box<dyn Pool>, ContractError> {
    let pool_config = POOL_CONFIG.load(storage)?;
    let total_tokens = querier.query_balance(contract, NATIVE_DENOM)?.amount;
    match pool_config.pool_type {
        PoolType::Token => Ok(Box::new(TokenPool::new(pool_config, total_tokens))),
        PoolType::Nft => Ok(Box::new(NftPool::new(pool_config, total_tokens))),
        PoolType::Trade => Ok(Box::new(TradePool::new(pool_config, total_tokens))),
    }
}

pub fn load_escrow_nft_pool(
    contract: &Addr,
    storage: &dyn Storage,
    querier: &QuerierWrapper,
) -> Result<Box<dyn EscrowNft>, ContractError> {
    let pool_config = POOL_CONFIG.load(storage)?;
    let total_tokens = querier.query_balance(contract, NATIVE_DENOM)?.amount;
    match pool_config.pool_type {
        PoolType::Token => Err(ContractError::InvalidPool("pool cannot escrow NFTs".to_string())),
        PoolType::Nft => Ok(Box::new(NftPool::new(pool_config, total_tokens))),
        PoolType::Trade => Ok(Box::new(TradePool::new(pool_config, total_tokens))),
    }
}

pub fn load_escrow_token_pool(
    contract: &Addr,
    storage: &dyn Storage,
    querier: &QuerierWrapper,
) -> Result<Box<dyn EscrowToken>, ContractError> {
    let pool_config = POOL_CONFIG.load(storage)?;
    let total_tokens = querier.query_balance(contract, NATIVE_DENOM)?.amount;
    match pool_config.pool_type {
        PoolType::Token => Ok(Box::new(TokenPool::new(pool_config, total_tokens))),
        PoolType::Nft => Err(ContractError::InvalidPool("pool cannot escrow tokens".to_string())),
        PoolType::Trade => Ok(Box::new(TradePool::new(pool_config, total_tokens))),
    }
}
