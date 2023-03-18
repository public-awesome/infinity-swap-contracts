use super::chain::SigningAccount;
use super::constants::{
    BASE_FACTORY_NAME, BASE_MINTER_NAME, CREATION_FEE, INFINITY_POOL_NAME, LISTING_FEE,
    MARKETPLACE_NAME, MINT_PRICE, SG721_NAME,
};
use base_factory::msg::ExecuteMsg as BaseFactoryExecuteMsg;
use base_factory::msg::InstantiateMsg as BaseFactoryInstantiateMsg;
use base_factory::state::BaseMinterParams;
use cosm_orc::orchestrator::cosm_orc::CosmOrc;
use cosm_orc::orchestrator::error::ProcessError;
use cosm_orc::orchestrator::{
    Coin as OrcCoin, CosmosgRPC, ExecResponse, InstantiateResponse, SigningKey,
};
use cosmwasm_std::{Coin, Empty, Uint128};
use cw_utils::Duration;
use infinity_pool::msg::InstantiateMsg as PoolInstantiateMsg;
use sg2::msg::{CollectionParams, CreateMinterMsg};
use sg721::CollectionInfo;
use sg_marketplace::msg::InstantiateMsg as MarketplaceInstantiateMsg;
use sg_marketplace::ExpiryRange;

pub fn instantiate_base_factory(
    orc: &mut CosmOrc<CosmosgRPC>,
    user: &SigningAccount,
    denom: &str,
) -> Result<InstantiateResponse, ProcessError> {
    orc.instantiate(
        BASE_FACTORY_NAME,
        &format!("{}_inst", BASE_FACTORY_NAME),
        &BaseFactoryInstantiateMsg {
            params: BaseMinterParams {
                code_id: orc.contract_map.code_id(BASE_MINTER_NAME).unwrap(),
                creation_fee: Coin {
                    denom: denom.to_string(),
                    amount: Uint128::from(CREATION_FEE),
                },
                min_mint_price: Coin {
                    denom: denom.to_string(),
                    amount: Uint128::from(MINT_PRICE),
                },
                // Setting this to 100% because I think there is a bug in the base minter contract
                mint_fee_bps: 10000,
                max_trading_offset_secs: 0,
                extension: None,
            },
        },
        &user.key,
        Some(user.account.address.parse().unwrap()),
        vec![],
    )
}

pub fn instantiate_minter(
    orc: &mut CosmOrc<CosmosgRPC>,
    creator_addr: String,
    signer: &SigningKey,
    denom: &str,
) -> Result<ExecResponse, ProcessError> {
    let init_minter_msg = BaseFactoryExecuteMsg::CreateMinter(CreateMinterMsg {
        init_msg: Some(Empty {}),
        collection_params: CollectionParams {
            code_id: orc.contract_map.code_id(SG721_NAME).unwrap(),
            name: "Collection".to_string(),
            symbol: "SYM".to_string(),
            info: CollectionInfo {
                creator: creator_addr,
                description: "Description".to_string(),
                image: "https://example.com/image.png".to_string(),
                start_trading_time: None,
                external_link: None,
                explicit_content: None,
                royalty_info: None,
            },
        },
    });

    let res = orc
        .execute(
            BASE_FACTORY_NAME,
            "base_factory_exec_minter_inst",
            &init_minter_msg,
            signer,
            vec![OrcCoin {
                amount: CREATION_FEE,
                denom: denom.parse().unwrap(),
            }],
        )
        .unwrap();

    let tags = res
        .res
        .find_event_tags("instantiate".to_string(), "_contract_address".to_string());

    let (minter_addr, sg721_addr) = (tags[0].value.to_string(), tags[1].value.to_string());
    orc.contract_map
        .add_address(BASE_MINTER_NAME, minter_addr)
        .unwrap();
    orc.contract_map
        .add_address(SG721_NAME, sg721_addr)
        .unwrap();

    Ok(res)
}

pub fn instantiate_marketplace(
    orc: &mut CosmOrc<CosmosgRPC>,
    user: &SigningAccount,
) -> Result<InstantiateResponse, ProcessError> {
    orc.instantiate(
        MARKETPLACE_NAME,
        &format!("{}_inst", MARKETPLACE_NAME),
        // Parameters copied from mainnet 2023-02-23
        &MarketplaceInstantiateMsg {
            // "trading_fee_percent":"2"
            trading_fee_bps: 200,
            // "ask_expiry":{"min":86400,"max":15552000}
            ask_expiry: ExpiryRange {
                min: 86400,
                max: 15552000,
            },
            // "bid_expiry":{"min":86400,"max":15552000}
            bid_expiry: ExpiryRange {
                min: 86400,
                max: 15552000,
            },
            // "operators":["stars1j7cddngyhwkr7w74qac960lee9uh9y0hgcsnfa"]
            operators: vec![user.account.address.clone()],
            sale_hook: None,
            // "max_finders_fee_percent":"2"
            max_finders_fee_bps: 200,
            // "min_price":"5000000"
            min_price: Uint128::from(10u64),
            // "stale_bid_duration":{"time":7776000}
            stale_bid_duration: Duration::Time(7776000),
            // "bid_removal_reward_percent":"0.5"
            bid_removal_reward_bps: 50,
            // "listing_fee":"500000"
            listing_fee: Uint128::from(LISTING_FEE),
        },
        &user.key,
        Some(user.account.address.parse().unwrap()),
        vec![],
    )
}

pub fn instantiate_infinity_pools(
    orc: &mut CosmOrc<CosmosgRPC>,
    denom: &str,
    user: &SigningAccount,
) -> Result<InstantiateResponse, ProcessError> {
    orc.instantiate(
        INFINITY_POOL_NAME,
        &format!("{}_inst", INFINITY_POOL_NAME),
        &PoolInstantiateMsg {
            denom: denom.to_string(),
            marketplace_addr: orc.contract_map.address(MARKETPLACE_NAME)?.to_string(),
            developer: None,
        },
        &user.key,
        Some(user.account.address.parse().unwrap()),
        vec![],
    )
}
