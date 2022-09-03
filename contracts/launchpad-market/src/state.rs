use cw_storage_plus::Map;
use cosmwasm_std::Uint128;
use cw_storage_plus::UniqueIndex;
use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};

use cw721::{Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub contract: Addr,
    pub staking_contract: Addr,
    // pub contract_info: ContractInfoResponse,
    pub launch_owner: Addr,
    pub owner: Addr,
    pub royalties: Vec<Coin>,
    pub royalty_fee: u32,
    pub royalty_wallet: Addr,
    pub platform_fee: u32,
    pub platform_wallet: Addr,

    //pub tokens: IndexedMap<&'a str, TokenInfo<Extension>, TokenIndexes<'a, Extension>>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct BagOfCoins {
    /// When the Approval expires (maybe Expiration::never)
    pub owner: Addr,
    pub bag: Vec<Coin>,
    pub expires: Expiration,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Bids {
    pub bids: Vec<BagOfCoins>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Ask {
    pub ask: Option<BagOfCoins>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token {
    pub token_id: u32,
    pub ask: Option<BagOfCoins>,
    pub bids: Vec<BagOfCoins>,
}


pub struct MarketContract<'a>
{
    pub state: Item<'a, State>,
    // pub ask_map: Map<'a, U32Key, Ask>,
    pub bid_map: Map<'a, (Addr, Vec<u8>), Token>,
    pub token_map: IndexedMap<'a, u32, Token, TokenIndexes<'a>>,
}


impl Default for MarketContract<'static>
{
    fn default() -> Self {
        Self::new(
            "state",
            "bidmap",
            "tokenmap",
            "tokenmap__ask__price__token__id",
        )
    }
}

impl<'a> MarketContract<'a>
{
    fn new(
        state_key: &'a str,
        bidmap_key: &'a str,
        tokenmap_key: &'a str,
        tokenmap_ask_price_token_id_key: &'a str,
    ) -> Self {
        let indexes = TokenIndexes {
            // ask_price: MultiIndex::new(token_ask_price_idx, tokenmap_key, tokenmap_ask_price_key),
            // price: UniqueIndex::new(token_price_idx, tokenmap_price_key),
            ask_price_token_id: UniqueIndex::new(token_ask_price_token_id_idx, tokenmap_ask_price_token_id_key),
        };
        Self {
            state: Item::new(state_key),
            // ask_map: Map::new(askmap_key),
            bid_map: Map::new(bidmap_key),
            token_map: IndexedMap::new(tokenmap_key, indexes),
        }
    }
}

pub struct TokenIndexes<'a>
{
    // pk goes to second tuple element
    // pub price: UniqueIndex<'a, U128Key, Token>,
    // pub ask_price: MultiIndex<'a, (U128Key, Vec<u8>), Token>,
    pub ask_price_token_id: UniqueIndex<'a, (Vec<u8>, Vec<u8>), Token>,
}

impl<'a> IndexList<Token> for TokenIndexes<'a>
{
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Token>> + '_> {
        let v: Vec<&dyn Index<Token>> = vec![&self.ask_price_token_id];//&self.price, &self.ask_price,
        Box::new(v.into_iter())
    }
}

/// get_coins returns coin of denom
fn get_coins(coins: &[Coin], denom: &str) -> u128 {
    coins
        .iter()
        .find(|c| c.denom == denom)
        .map(|m| m.amount)
        .unwrap_or(Uint128::new(0)).u128()
}

fn token_ask_price_token_id_idx(token: &Token) -> (Vec<u8>, Vec<u8>) {
    match token.ask.as_ref() {
        Some(ask) => {
            return (get_coins(&ask.bag,"uluna").to_be_bytes().to_vec(), token.token_id.to_be_bytes().to_vec())
        },
        None => {
            return (0u128.to_be_bytes().to_vec(), token.token_id.to_be_bytes().to_vec())
        }
    }
}


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokenMarketInfo {
    pub token_id: u32,
    pub ask: Option<BagOfCoins>,
    pub bids: Vec<BagOfCoins>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct NftMarketInfoResponse {
    pub token: TokenMarketInfo,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct BidderBidsResponse {
    pub tokens: Vec<Token>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AllNftMarketInfoResponse {
    pub tokens: Vec<TokenMarketInfo>,
    pub start_after: Option<u32>,
    pub limit: Option<u32>,
    pub count: Option<String>,
    pub is_bids: Option<bool>,
    pub is_ask: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AllNftPriceMapResponse {
    pub tokens: Vec<Token>,
    pub start_after: Option<String>,
    pub limit: String,
    pub skip: String,
    pub count: Option<String>,
    pub is_bids: bool,
    pub is_ask: bool,
}
