use crate::{
    pool::{EscrowNft, NftPool, Pool, TradePool},
    state::{PoolType, POOL_CONFIG},
    ContractError,
};

use cosmwasm_std::Storage;

pub fn load_escrow_nft_pool(storage: &dyn Storage) -> Result<Box<dyn EscrowNft>, ContractError> {
    let pool_config = POOL_CONFIG.load(storage)?;
    match pool_config.pool_type {
        PoolType::Nft => Ok(Box::new(NftPool::new(pool_config))),
        PoolType::Trade => Ok(Box::new(TradePool::new(pool_config))),
        PoolType::Token => Err(ContractError::InvalidPool("pool cannot escrow NFTs".to_string())),
    }
}
