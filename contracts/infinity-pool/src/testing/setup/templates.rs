use crate::testing::setup::msg::MarketAccounts;
use crate::testing::setup::setup_accounts::setup_accounts;
use cosmwasm_std::{coin, Decimal, Timestamp};
use sg2::{
    msg::CollectionParams,
    tests::{
        mock_collection_params_1, mock_collection_params_high_fee, mock_collection_two,
        mock_curator_payment_address,
    },
};
use sg721::{CollectionInfo, RoyaltyInfoResponse};
use sg_std::GENESIS_MINT_START_TIME;
use test_suite::common_setup::{
    contract_boxes::custom_mock_app,
    msg::{MinterCollectionResponse, MinterInstantiateParams, VendingTemplateResponse},
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

pub fn standard_minter_template(num_tokens: u32) -> VendingTemplateResponse<MarketAccounts> {
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
    VendingTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: MarketAccounts {
            owner,
            bidder,
            creator,
        },
    }
}

pub fn _minter_template_high_fee(num_tokens: u32) -> VendingTemplateResponse<MarketAccounts> {
    let mut app = custom_mock_app();
    let (owner, bidder, creator) = setup_accounts(&mut app).unwrap();
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_high_fee(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let code_ids = vending_minter_code_ids(&mut app);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut app,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    VendingTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: MarketAccounts {
            owner,
            bidder,
            creator,
        },
    }
}

pub fn _minter_template_owner_admin(num_tokens: u32) -> VendingTemplateResponse<MarketAccounts> {
    let mut app = custom_mock_app();
    let (owner, bidder, creator) = setup_accounts(&mut app).unwrap();
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let code_ids = vending_minter_code_ids(&mut app);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut app,
        owner.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    VendingTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: MarketAccounts {
            owner,
            bidder,
            creator,
        },
    }
}

pub fn _minter_with_curator(num_tokens: u32) -> VendingTemplateResponse<MarketAccounts> {
    let mut app = custom_mock_app();
    let (owner, bidder, creator) = setup_accounts(&mut app).unwrap();
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_curator_payment_address(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let code_ids = vending_minter_code_ids(&mut app);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut app,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    VendingTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: MarketAccounts {
            owner,
            bidder,
            creator,
        },
    }
}

pub fn _minter_two_collections(num_tokens: u32) -> VendingTemplateResponse<MarketAccounts> {
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
    VendingTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: MarketAccounts {
            owner,
            bidder,
            creator,
        },
    }
}

pub fn _minter_two_collections_with_time(
    num_tokens: u32,
    time_one: Timestamp,
    time_two: Timestamp,
) -> VendingTemplateResponse<MarketAccounts> {
    let mut app = custom_mock_app();
    let (owner, bidder, creator) = setup_accounts(&mut app).unwrap();
    let collection_params = mock_collection_params_1(Some(time_one));
    let collection_params_two = mock_collection_two(Some(time_two));
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
    VendingTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: MarketAccounts {
            owner,
            bidder,
            creator,
        },
    }
}

pub fn _mock_collection_params_30_pct_fee(
    start_trading_time: Option<Timestamp>,
) -> CollectionParams {
    CollectionParams {
        code_id: 1,
        name: String::from("Test Coin"),
        symbol: String::from("TEST"),
        info: CollectionInfo {
            creator: "creator".to_string(),
            description: String::from("Stargaze Monkeys"),
            image:
                "ipfs://bafybeigi3bwpvyvsmnbj46ra4hyffcxdeaj6ntfk5jpic5mx27x6ih2qvq/images/1.png"
                    .to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            royalty_info: Some(RoyaltyInfoResponse {
                payment_address: "creator".to_string(),
                share: Decimal::percent(30),
            }),
            start_trading_time,
            explicit_content: None,
        },
    }
}

pub fn _minter_template_30_pct_fee(num_tokens: u32) -> VendingTemplateResponse<MarketAccounts> {
    let mut app = custom_mock_app();
    let (owner, bidder, creator) = setup_accounts(&mut app).unwrap();
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = _mock_collection_params_30_pct_fee(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let code_ids = vending_minter_code_ids(&mut app);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut app,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    VendingTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: MarketAccounts {
            owner,
            bidder,
            creator,
        },
    }
}
