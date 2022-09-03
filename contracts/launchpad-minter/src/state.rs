use cosmwasm_std::Coin;
use cw0::NativeBalance;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw0::Expiration;

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub launch_owner: Addr,
    pub owner: Addr,
    pub whitelist_password: Option<String>,
    pub start_after: Expiration,
    pub public_mint_height: u64,
    // pub price: u64,
    pub price_bag: Vec<Coin>,
    pub unminted: Vec<u32>,
    pub max_supply: u32, // total NFTs in collection
    pub mint_limit: u32, // total mint per addresss
    pub contract: Addr, // nft contract
    pub staking_contract: Addr, // staking contract
    pub nft_name: String,
    pub nft_description: String,
    pub token_uri: Option<String>,
    pub attributes_ipfs: Option<String>,
    pub image_ipfs: Option<String>,
    pub external_url: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    pub background_color: Option<String>,
}

pub const STATE: Item<State> = Item::new("state");

pub const MINTED: Map<&Addr, u32> = Map::new("minted");
