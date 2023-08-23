use crate::setup::setup_accounts::{setup_accounts, MarketAccounts};
use crate::setup::setup_contracts::{setup_fair_burn, setup_marketplace, setup_royalty_registry};
use crate::setup::setup_infinity_contracts::{
    contract_infinity_pair, setup_infinity_factory, setup_infinity_global, setup_infinity_index,
    setup_infinity_router,
};

use anyhow::Error;
use cosmwasm_std::{coin, Addr, Timestamp};
use sg2::{
    msg::CollectionParams,
    tests::{mock_collection_params_1, mock_collection_two},
};
use sg_std::GENESIS_MINT_START_TIME;
use test_suite::common_setup::setup_accounts_and_block::setup_block_time;
use test_suite::common_setup::{
    contract_boxes::custom_mock_app,
    msg::{MinterCollectionResponse, MinterInstantiateParams, MinterTemplateResponse},
    setup_minter::{
        common::minter_params::minter_params_token,
        vending_minter::{
            mock_params::mock_create_minter,
            setup::{configure_minter, vending_minter_code_ids},
        },
    },
};

fn standard_minter_params_token(
    num_tokens: u32,
    collection_params: CollectionParams,
) -> MinterInstantiateParams {
    let mut init_msg = mock_create_minter(None, collection_params, None).init_msg;
    init_msg.num_tokens = num_tokens;
    init_msg.per_address_limit = num_tokens / 100;
    init_msg.mint_price = coin(100000000, "ustars");
    let mut minter_params = minter_params_token(num_tokens);
    minter_params.init_msg = Some(init_msg);
    minter_params
}

pub fn standard_minter_template(num_tokens: u32) -> MinterTemplateResponse<MarketAccounts> {
    let mut app = custom_mock_app();
    let (owner, bidder, creator) = setup_accounts(&mut app).unwrap();
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let minter_params = standard_minter_params_token(num_tokens, collection_params.clone());
    let code_ids = vending_minter_code_ids(&mut app);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut app,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    MinterTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: MarketAccounts {
            owner,
            bidder,
            creator,
        },
    }
}

pub fn minter_two_collections(num_tokens: u32) -> MinterTemplateResponse<MarketAccounts> {
    let mut app = custom_mock_app();
    let (owner, bidder, creator) = setup_accounts(&mut app).unwrap();
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let collection_params_two = mock_collection_two(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let minter_params_two = minter_params_token(num_tokens);
    let code_ids = vending_minter_code_ids(&mut app);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut app,
        creator.clone(),
        vec![collection_params, collection_params_two],
        vec![minter_params, minter_params_two],
        code_ids,
    );
    MinterTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: MarketAccounts {
            owner,
            bidder,
            creator,
        },
    }
}

pub struct InfinityTestSetup {
    pub vending_template: MinterTemplateResponse<MarketAccounts>,
    pub marketplace: Addr,
    pub infinity_global: Addr,
    pub infinity_index: Addr,
    pub infinity_factory: Addr,
    pub infinity_pair_code_id: u64,
}

fn increment_number_in_string(s: &str, increment: u32) -> String {
    let prefix: String = s.chars().take_while(|c| c.is_alphabetic()).collect();
    let number: String = s.chars().skip_while(|c| c.is_alphabetic()).collect();
    let incremented_number = number.parse::<u32>().unwrap_or(0) + increment;
    format!("{}{}", prefix, incremented_number)
}

pub fn setup_infinity_test(
    mut vt: MinterTemplateResponse<MarketAccounts>,
) -> Result<InfinityTestSetup, Error> {
    setup_block_time(&mut vt.router, GENESIS_MINT_START_TIME, None);

    let fair_burn = setup_fair_burn(&mut vt.router, &vt.accts.creator);
    let royalty_registry = setup_royalty_registry(&mut vt.router, &vt.accts.creator);
    let marketplace = setup_marketplace(&mut vt.router, &vt.accts.creator.clone());

    let pre_infinity_global =
        Addr::unchecked(increment_number_in_string(marketplace.as_ref(), 4));

    let infinity_factory =
        setup_infinity_factory(&mut vt.router, &vt.accts.creator.clone(), &pre_infinity_global);
    let infinity_index =
        setup_infinity_index(&mut vt.router, &vt.accts.creator.clone(), &pre_infinity_global);
    let infinity_router =
        setup_infinity_router(&mut vt.router, &vt.accts.creator.clone(), &pre_infinity_global);

    let infinity_pair_code_id = vt.router.store_code(contract_infinity_pair());

    let infinity_global = setup_infinity_global(
        &mut vt.router,
        vt.accts.creator.to_string(),
        fair_burn.to_string(),
        royalty_registry.to_string(),
        marketplace.to_string(),
        infinity_factory.to_string(),
        infinity_index.to_string(),
        infinity_router.to_string(),
        infinity_pair_code_id,
    );
    assert_eq!(infinity_global, pre_infinity_global);

    Ok(InfinityTestSetup {
        vending_template: vt,
        marketplace,
        infinity_global,
        infinity_index,
        infinity_factory,
        infinity_pair_code_id,
    })
}
