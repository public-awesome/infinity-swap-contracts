use cosmwasm_schema::{
    cw_serde,
    serde::{de::DeserializeOwned, Serialize},
};
use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Api, Binary, Event, StdError, Timestamp, Uint128,
};
use cw_utils::maybe_addr;
use sg_marketplace_common::TransactionFees;
use std::fmt;

#[cw_serde]
pub enum TransactionType {
    UserSubmitsNfts,
    UserSubmitsTokens,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cw_serde]
pub struct TokenPayment {
    pub label: String,
    pub amount: Uint128,
    pub address: String,
}

#[cw_serde]
pub struct NftPayment {
    pub label: String,
    pub collection: String,
    pub token_id: String,
    pub address: String,
}

#[cw_serde]
pub struct Swap {
    pub source: String,
    pub transaction_type: TransactionType,
    pub sale_price: Uint128,
    pub network_fee: Uint128,
    pub nft_payments: Vec<NftPayment>,
    pub token_payments: Vec<TokenPayment>,
    pub data: Option<Binary>,
}

impl Swap {
    pub fn set_data<T: Serialize>(&mut self, data: T) {
        self.data = Some(to_binary(&data).unwrap());
    }

    pub fn unpack_data<T: DeserializeOwned>(&self) -> Result<T, StdError> {
        from_binary(&self.data.as_ref().unwrap())
    }
}

impl From<&Swap> for Event {
    fn from(swap: &Swap) -> Self {
        let mut attributes = vec![
            attr("source", swap.source.to_string()),
            attr("transaction_type", swap.transaction_type.to_string()),
            attr("sale_price", swap.sale_price.to_string()),
            attr("network_fee", swap.network_fee.to_string()),
        ];
        for nft_payment in &swap.nft_payments {
            attributes.extend([
                attr(
                    format!("{}_nft_payment_token_id", nft_payment.label),
                    nft_payment.token_id.clone(),
                ),
                attr(
                    format!("{}_nft_payment_address", nft_payment.label),
                    nft_payment.address.clone(),
                ),
            ]);
        }
        for token_payment in &swap.token_payments {
            attributes.extend([
                attr(
                    format!("{}_token_payment_amount", token_payment.label),
                    token_payment.amount.clone(),
                ),
                attr(
                    format!("{}_token_payment_address", token_payment.label),
                    token_payment.address.clone(),
                ),
            ]);
        }
        Event::new("swap").add_attributes(attributes)
    }
}

pub fn tx_fees_to_swap(
    tx_fees: TransactionFees,
    transaction_type: TransactionType,
    collection: &Addr,
    token_id: &str,
    sale_price: Uint128,
    buyer: &Addr,
    source: &Addr,
) -> Swap {
    let mut token_payments: Vec<TokenPayment> = vec![];
    if let Some(finders_fee) = tx_fees.finders_fee {
        token_payments.push(TokenPayment {
            label: "finder".to_string(),
            address: finders_fee.recipient.to_string(),
            amount: finders_fee.coin.amount,
        });
    }
    if let Some(royalty_fee) = tx_fees.royalty_fee {
        token_payments.push(TokenPayment {
            label: "royalty".to_string(),
            address: royalty_fee.recipient.to_string(),
            amount: royalty_fee.coin.amount,
        });
    }
    token_payments.push(TokenPayment {
        label: "seller".to_string(),
        address: tx_fees.seller_payment.recipient.to_string(),
        amount: tx_fees.seller_payment.coin.amount,
    });

    Swap {
        source: source.to_string(),
        transaction_type,
        sale_price,
        network_fee: tx_fees.fair_burn_fee,
        nft_payments: vec![NftPayment {
            label: "buyer".to_string(),
            collection: collection.to_string(),
            token_id: token_id.to_string(),
            address: buyer.to_string(),
        }],
        token_payments,
        data: None,
    }
}

#[cw_serde]
pub struct SwapResponse {
    pub swaps: Vec<Swap>,
}

#[cw_serde]
pub struct NftOrder {
    pub token_id: String,
    pub amount: Uint128,
}

/// SwapParams contains the parameters for a swap
#[cw_serde]
pub struct SwapParams {
    /// The deadline after which the swap will be rejected
    pub deadline: Timestamp,
    /// Whether or not to revert the entire trade if one of the swaps fails
    pub robust: bool,
    /// The address to receive the assets from the swap, if not specified is set to sender
    pub asset_recipient: Option<String>,
    /// The address of the finder, will receive a portion of the fees equal to percentage set by maker
    pub finder: Option<String>,
}

pub struct SwapParamsInternal {
    pub deadline: Timestamp,
    pub robust: bool,
    pub asset_recipient: Option<Addr>,
    pub finder: Option<Addr>,
}

pub fn transform_swap_params(
    api: &dyn Api,
    swap_params: SwapParams,
) -> Result<SwapParamsInternal, StdError> {
    Ok(SwapParamsInternal {
        deadline: swap_params.deadline,
        robust: swap_params.robust,
        asset_recipient: maybe_addr(api, swap_params.asset_recipient)?,
        finder: maybe_addr(api, swap_params.finder)?,
    })
}
