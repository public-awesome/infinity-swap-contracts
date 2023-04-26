use crate::setup::msg::MarketAccounts;
use crate::setup::setup_accounts::setup_accounts;
use cosmwasm_std::{coin, Timestamp};
use sg2::{
    msg::CollectionParams,
    tests::{mock_collection_params_1, mock_collection_two},
};
use sg_std::GENESIS_MINT_START_TIME;
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
