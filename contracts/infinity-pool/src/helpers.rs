use crate::state::{Pool, pools, pool_key};
use cosmwasm_std::{StdError, Storage};

pub fn save_pool(store: &mut dyn Storage, pool: &Pool) -> Result<(), StdError> {
    let pool_key = pool_key(&pool.collection, pool.id);
    pools().save(store, pool_key, &pool)
}