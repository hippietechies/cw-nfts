use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    // pub launch_icon: String,
    // pub launch_banner: String,
    pub owner: Addr,
    pub whitelist_password: Option<String>,
    pub public_mint_height: u64,
    pub price: u64,
    pub max_supply: u64,
    pub mint_limit: u32,
    pub contract: Addr,
    pub staking_contract: Addr,
    pub nft_name: String,
    pub nft_description: String,
    pub token_uri: Option<String>,
    pub image_ipfs: Option<String>,
    pub external_url: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    pub background_color: Option<String>,
}

pub const STATE: Item<State> = Item::new("state");

pub const MINTED: Map<&Addr, u32> = Map::new("minted");

pub const CLAIMED: Map<u32, Addr> = Map::new("claimed");
pub const CLAIMED2: Map<u32, Addr> = Map::new("claimed");
