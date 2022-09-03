use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("token_id already in the midst of rewarding")]
    Claiming {},

    #[error("There are no claimable rewards")]
    NoClaimableRewards {},

    #[error("There are no matured rewards")]
    NotMaturedRewards {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Transaction has to be properly funded")]
    Unfunded {},

    #[error("Address is unknown in tx")]
    UnknownAddress {},

    #[error("Claimable Luna is too small to claim")]
    UnreachableWeight {},

    #[error("owner: {owner}, sender: {sender}")]
    UnauthorizedJan { owner: String,  sender: String},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("Invalid reply")]
    InvalidReplyId {}
}
