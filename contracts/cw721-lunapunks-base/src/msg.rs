use cw721_base::MintMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Binary, Coin};
use cw721::Expiration;
use cw721_base::msg::QueryMsg as CW721QueryMsg;
use cw721_base::msg::ExecuteMsg as CW721ExecuteMsg;

use crate::state::Extension;


#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum StakingMsg {
    Revest{}
}

/// This is like Cw721ExecuteMsg but we add a Mint command for an owner
/// to make this stand-alone. You will likely want to remove mint and
/// use other control logic in any contract that inherits this.
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum LunaPunkExecuteMsg<T> {
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft { recipient: String, token_id: String },
    /// Send is a base message to transfer a token to a contract and trigger an action
    /// on the receiving contract.
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    /// Allows operator to transfer / send the token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted Approval
    Revoke { spender: String, token_id: String },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll { operator: String },
    Release { bids: Vec<Coin> },

    /// Mint a new NFT, can only be called by the contract minter
    Mint(MintMsg<T>),
    Burn { token_id: String },
}

// impl From<LunaPunkExecuteMsg<Extension>> for CW721ExecuteMsg<Extension> {
//     fn from(msg: LunaPunkExecuteMsg<Empty>) -> CW721ExecuteMsg<Empty> {
//         match msg {
//             _ => panic!("cannot covert {:?} to CW721QueryMsg", msg),
//         }
//     }
// }

impl From<LunaPunkExecuteMsg<Extension>> for CW721ExecuteMsg<Extension> {
    fn from(msg: LunaPunkExecuteMsg<Extension>) -> CW721ExecuteMsg<Extension> {
        match msg {
            LunaPunkExecuteMsg::TransferNft { recipient, token_id }
            => CW721ExecuteMsg::TransferNft { recipient, token_id },
            LunaPunkExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => CW721ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            },
            LunaPunkExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => CW721ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            },
            LunaPunkExecuteMsg::Revoke { spender, token_id } => CW721ExecuteMsg::Revoke { spender, token_id },
            LunaPunkExecuteMsg::ApproveAll {
                operator,
                expires,
            } => CW721ExecuteMsg::ApproveAll {
                operator,
                expires,
            },
            LunaPunkExecuteMsg::RevokeAll { operator } => CW721ExecuteMsg::RevokeAll { operator },
            LunaPunkExecuteMsg::Mint(mint_msg) => CW721ExecuteMsg::Mint(mint_msg),
            LunaPunkExecuteMsg::Burn { token_id } => CW721ExecuteMsg::Burn { token_id },
            _ => panic!("cannot covert {:?} to CW721QueryMsg", msg),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LunaPunkQueryMsg {
    /// Return the owner of the given token, error if token does not exist
    /// Return type: OwnerOfResponse
    OwnerOf {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },
    /// List all operators that can access all of the owner's tokens
    /// Return type: `ApprovedForAllResponse`
    ApprovedForAll {
        owner: String,
        /// unset or false will filter out expired items, you must set to true to see them
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Total number of tokens issued
    NumTokens {},
    StakingContract {},
    OwnerTokens {
        owner: String,
        start_after: Option<String>,
    },

    /// With MetaData Extension.
    /// Returns top-level metadata about the contract: `ContractInfoResponse`
    ContractInfo {},
    /// With MetaData Extension.
    /// Returns metadata about one particular token, based on *ERC721 Metadata JSON Schema*
    /// but directly from the contract: `NftInfoResponse`
    NftInfo {
        token_id: String,
    },
    /// With MetaData Extension.
    /// Returns the result of both `NftInfo` and `OwnerOf` as one query as an optimization
    /// for clients: `AllNftInfo`
    AllNftInfo {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },


    /// With Enumerable extension.
    /// Returns all tokens owned by the given address, [] if unset.
    /// Return type: TokensResponse.
    Tokens {
        owner: String,
        start_after: Option<String>,
        skip: Option<u32>,
        limit: Option<u32>,
    },
    /// With Enumerable extension.
    /// Requires pagination. Lists all token_ids controlled by the contract.
    /// Return type: TokensResponse.
    AllTokens {
        start_after: Option<String>,
        skip: Option<u32>,
        limit: Option<u32>,
    },

    // Return the minter
    Minter {},

    AllOperators {
        owner: String,
        /// unset or false will filter out expired items, you must set to true to see them
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Return operator that can access all of the owner's tokens.
    /// Return type: `ApprovalResponse`
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },

    /// Return approvals that a token has
    /// Return type: `ApprovalsResponse`
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
}

impl From<LunaPunkQueryMsg> for CW721QueryMsg {
    fn from(msg: LunaPunkQueryMsg) -> CW721QueryMsg {
        match msg {
            LunaPunkQueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            } => CW721QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            },
            LunaPunkQueryMsg::Approvals {
                token_id,
                include_expired,
            } => CW721QueryMsg::Approvals {
                token_id,
                include_expired,
            },
            LunaPunkQueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => CW721QueryMsg::OwnerOf {
                token_id,
                include_expired,
            },
            LunaPunkQueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            } => CW721QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            },
            LunaPunkQueryMsg::NumTokens {} => CW721QueryMsg::NumTokens {},
            LunaPunkQueryMsg::ContractInfo {} => CW721QueryMsg::ContractInfo {},
            LunaPunkQueryMsg::NftInfo { token_id } => CW721QueryMsg::NftInfo { token_id },
            LunaPunkQueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => CW721QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            },
            LunaPunkQueryMsg::Tokens {
                owner,
                start_after,
                skip,
                limit,
            } => CW721QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            },
            _ => panic!("cannot covert {:?} to CW721QueryMsg", msg),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SuccessMsg {
    hello {}
}
