use cosmwasm_std::{
    CheckedFromRatioError, CheckedMultiplyFractionError, DivideByZeroError, OverflowError, StdError,
};
use cw_utils::PaymentError;
use infinity_shared::InfinityError;
use sg_marketplace_common::MarketplaceStdError;
use stargaze_royalty_registry::ContractError as RoyaltyRegistryError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    CheckedMultiplyFractionError(#[from] CheckedMultiplyFractionError),

    #[error("{0}")]
    CheckedFromRatioError(#[from] CheckedFromRatioError),

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("{0}")]
    MarketplaceStdError(#[from] MarketplaceStdError),

    #[error("{0}")]
    RoyaltyRegistryError(#[from] RoyaltyRegistryError),

    #[error("{0}")]
    InfinityError(#[from] InfinityError),

    #[error("InvalidPair: {0}")]
    InvalidPair(String),

    #[error("InvalidPairQuote: {0}")]
    InvalidPairQuote(String),
}
