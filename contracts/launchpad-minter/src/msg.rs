use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw0::Expiration;
use cosmwasm_std::Coin;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StakingMsg {
    Revest{},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub launch_owner: String,
    pub whitelist_password: Option<String>,
    pub start_after: Option<Expiration>,
    pub public_mint_height: u64,
    // pub price: u64,
    pub price_bag: Vec<Coin>,
    pub max_supply: u32,
    pub max_mint: u32,
    pub mint_limit: u32,
    pub contract: String,
    pub staking_contract: String,
    pub nft_name: String,
    pub nft_description: String,
    pub token_uri: Option<String>,
    pub attributes_ipfs: Option<String>,
    pub image_ipfs: Option<String>,
    pub external_url: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    pub background_color: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub launch_owner: Option<String>,
    pub whitelist_password: Option<String>,
    pub start_after: Option<Expiration>,
    pub public_mint_height: Option<u64>,
    pub price_bag: Option<Vec<Coin>>,
    pub max_supply: Option<u32>,
    pub max_mint: Option<u32>,
    pub mint_limit: Option<u32>,
    pub contract: Option<String>,
    pub nft_name: Option<String>,
    pub nft_description: Option<String>,
    pub token_uri: Option<String>,
    pub attributes_ipfs: Option<String>,
    pub image_ipfs: Option<String>,
    pub external_url: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    pub background_color: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetWhitelistPassword { password: Option<String> },
    SetContract { contract: String },
    Release { },
    MintOnBehalf { password: Option<u32> },
    MultiMintOnBehalf { quantity: u32, password: Option<u32> }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetMaxSupply {},
    GetMaxMint {},  /// gets total max mints per address
    GetMintInfo {},  /// gets total max mints per address
    GetContractBalance {}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MaxSupplyResponse {
    pub max_supply: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MaxMintResponse {
    pub max_mint: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintInfoResponse {
    pub price_bag: Option<Vec<Coin>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetContractBalanceResponse {
    pub amount: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct NumTokensResponse {
    pub count: u64,
}
