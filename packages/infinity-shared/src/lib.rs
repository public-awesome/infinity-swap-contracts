pub use crate::error::InfinityError;

mod error;

use cosmwasm_std::{ensure_eq, Addr, Empty, MessageInfo, QuerierWrapper, StdResult};
use cw721::OwnerOfResponse;
use cw721_base::helpers::Cw721Contract;
use std::marker::PhantomData;

/// Invoke `owner_of` to get the owner of an NFT.
pub fn owner_of(
    querier: &QuerierWrapper,
    collection: &Addr,
    token_id: &str,
) -> StdResult<OwnerOfResponse> {
    Cw721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData)
        .owner_of(querier, token_id, false)
}

/// Invoke `only_nft_owner` to check that the sender is the owner of the NFT.
pub fn only_nft_owner(
    querier: &QuerierWrapper,
    info: &MessageInfo,
    collection: &Addr,
    token_id: &str,
) -> Result<(), InfinityError> {
    let owner_of_response = owner_of(querier, collection, token_id)
        .map_err(|_| InfinityError::InternalError("failed to get owner of nft".to_string()))?;
    ensure_eq!(
        info.sender,
        owner_of_response.owner,
        InfinityError::Unauthorized("sender is not the owner of the nft".to_string())
    );
    Ok(())
}
