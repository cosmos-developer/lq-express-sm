use cosmwasm_std::StdError;
use cosmwasm_std::Uint128;
use cw_utils::PaymentError;
use thiserror::Error;
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    #[error("{0}")]
    Payment(#[from] PaymentError),
    #[error("Exceed mintable block height")]
    ExceedMintableBlock {},
    #[error("Exceed maximum mintable amount")]
    ExceedMaximumMintableAmount {},
    #[error("Pool not exists")]
    PoolNotExist {},
    #[error("Offer and ask token should not be identical")]
    DoublingAssets {},
    #[error("Insufficient funds available in the pool to complete the swap: {asked_amount} > {available_amount}")]
    InsufficientFunds {
        asked_amount: Uint128,
        available_amount: Uint128,
    },
}
