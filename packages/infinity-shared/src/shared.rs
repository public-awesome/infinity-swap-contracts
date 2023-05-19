use crate::InfinityError;

use cosmwasm_std::{ensure, Addr, Api, Empty, QuerierWrapper, StdResult};
use cw721_base::helpers::Cw721Contract;
use sg_marketplace::{
    msg::{ParamsResponse, QueryMsg as MarketplaceQueryMsg},
    state::SudoParams,
};
use std::marker::PhantomData;

/// Load the marketplace params
pub fn load_marketplace_params(
    querier: &QuerierWrapper,
    marketplace_addr: &Addr,
) -> StdResult<SudoParams> {
    let marketplace_params: ParamsResponse =
        querier.query_wasm_smart(marketplace_addr, &MarketplaceQueryMsg::Params {})?;
    Ok(marketplace_params.params)
}

pub fn only_nft_owner(
    querier: &QuerierWrapper,
    api: &dyn Api,
    sender: &Addr,
    collection: &Addr,
    token_id: &str,
) -> Result<Addr, InfinityError> {
    let response = Cw721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData)
        .owner_of(&querier, token_id, false)?;

    let owner = api.addr_validate(&response.owner)?;
    ensure!(&owner == sender, InfinityError::NotNftOwner(sender.to_string()));

    Ok(owner)
}
