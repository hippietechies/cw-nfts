use cw721_base::Cw721Contract;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use cosmwasm_std::{Addr, BlockInfo, StdResult, Storage, Empty, Coin};

use cw721::{OwnerOfResponse, NftInfoResponse, ContractInfoResponse, Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

pub type Cw721ExtendedContract<'a> = Cw721Contract<'a, Extension, Empty>;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

// see: https://docs.opensea.io/docs/metadata-standards
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}

pub type Extension = Option<Metadata>;


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct BagOfCoins {
    /// When the Approval expires (maybe Expiration::never)
    pub owner: Addr,
    pub bag: Vec<Coin>,
    pub expires: Expiration,
}

/// Shows Address string of staking contract
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct StakingContractResponse {
    pub contract: String,
}

