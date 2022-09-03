use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Coin;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub whitelist_password: Option<String>,
    pub public_mint_height: u64,
    pub price: u64,
    pub max_supply: u64,
    pub max_mint: u32,
    pub mint_limit: u32,
    pub contract: String,
    pub staking_contract: String,
    pub nft_name: String,
    pub nft_description: String,
    pub token_uri: Option<String>,
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
    UpdateConfig { config: UpdateConfigMsg },
    Release { },
    MintOnBehalf { password: Option<u32> },
    MultiMintOnBehalf { quantity: u32, password: Option<u32> },
    SetPrice { price: String },
    // OgClaimMint { token_id: u32 }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetMaxSupply {},
    GetContractBalance {},
    IsClaimed { token_id: u32 }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsClaimedResponse {
    pub claimed: bool,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MaxSupplyResponse {
    pub max_supply: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetContractBalanceResponse {
    pub amount: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct NumTokensResponse {
    pub count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub whitelist_password: Option<String>,
    pub public_mint_height: Option<u64>,
    pub price: Option<u64>,
    pub max_supply: Option<u32>,
    pub max_mint: Option<u32>,
    pub mint_limit: Option<u32>,
    pub contract: Option<String>,
    pub staking_contract: Option<String>,
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
