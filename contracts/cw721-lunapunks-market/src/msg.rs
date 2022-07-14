use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw721::Expiration;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum StakingMsg {
    Revest{}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Address of the NFT contract
    pub contract: String,
    /// 1000 = 10% max royalty fee, 1000, with 2 decimals ~0.01
    pub royalty_fee: u32,
    pub royalty_wallet: String,
}

/// This is like Cw721ExecuteMsg but we add a Mint command for an owner
/// to make this stand-alone. You will likely want to remove mint and
/// use other control logic in any contract that inherits this.
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Release { release_funds: Vec<Coin> },
    /// Sets a bid with bag of coins for NFT
    BidAddNft { token_id: u32, expires: Option<Expiration> },
    /// Withdraw a bid with bag of coins for NFT
    BidWithdrawNft { token_id: u32,  },
    /// Accept highest bid with bag of coins for NFT
    BidAcceptNft { token_id: u32, bidder_address: String },

    /// Sets an offer for bag of coins for NFT
    AskAddNft { token_id: u32, ask_funds: Vec<Coin>, expires: Option<Expiration> },
    /// Withdraw an offer for bag of coins for NFT
    AskWithdrawNft { token_id: u32 },
    /// Accepts a bid for bag of coins for NFT
    AskAcceptNft { token_id: u32 },

    SetRoyaltyWallet { royalty_wallet: String },
    SetRoyaltyFee { royalty_fee: u32 },
    // /// Transfers NFT to escrow contract
    // OfferTradeNft { token_id_offerer: u32, token_id_offeree: u32, offeree_address: String },
    // /// return NFTs and removes escrow
    // WithdrawTradeNft { token_id: u32 },
    // /// return NFTs and removes escrow
    // AcceptTradeNft { token_id_offerer: u32, token_id_offeree: u32, offerer_address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    RoyaltyInfo { },
    NftMarketInfo {
        token_id: u32,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },
    AllNftBidsInfo {
        /// unset or false will filter out expired approvals, you must set to true to see them
        bidder: String,
        include_expired: Option<bool>,
        start_after: Option<u32>,
        skip: Option<u32>,
        limit: Option<u32>,
    },
    AllNftAsksInfo {
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,

        start_after: Option<u32>,
        skip: Option<u32>,
        limit: Option<u32>,
    },
    AllNftAsksSortInfo {
        /// unset or false will filter out expired approvals, you must set to true to see them
        ascending: Option<i32>,
        include_expired: Option<bool>,
        start_after: Option<u32>,
        skip: Option<u32>,
        limit: Option<u32>,
    },
    // AllNftAsksSortInfo2 {
    //     /// unset or false will filter out expired approvals, you must set to true to see them
    //     ascending: Option<i32>,
    //     include_expired: Option<bool>,
    //     start_after: Option<u32>,
    //     skip: Option<u32>,
    //     limit: Option<u32>,
    // },
    // AllNftAsksSortInfo3 {
    //     /// unset or false will filter out expired approvals, you must set to true to see them
    //     ascending: Option<i32>,
    //     include_expired: Option<bool>,
    //     start_after: Option<u32>,
    //     skip: Option<u32>,
    //     limit: Option<u32>,
    // },
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct MigrateMsg {}
