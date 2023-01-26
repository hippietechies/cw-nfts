#[cfg(not(feature = "library"))]
use crate::msg::MigrateMsg;
use crate::msg::{UpdateConfigMsg, IsClaimedResponse};
use cosmwasm_std::{entry_point, coins};
use cosmwasm_std::{
    coin, has_coins, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdResult, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GetContractBalanceResponse, InstantiateMsg, MaxSupplyResponse, NumTokensResponse,
    QueryMsg,
};
use crate::state::{State, MINTED, STATE, CLAIMED2};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:luna-punks";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use cw721_metadata_onchain::ExecuteMsg as CW721_MetaData_ExecuteMsg;
use cw721_metadata_onchain::MintMsg as CW721_MetaData_MintMsg;
use cw721_metadata_onchain::QueryMsg as CW721_MetaData_QueryMsg;
use cw721_metadata_onchain::{Extension, Metadata};

use cw721::{OwnerOfResponse, Cw721QueryMsg};

use std::collections::hash_map::DefaultHasher;
use std::convert::TryInto;
use std::hash::{Hash, Hasher};

fn whitelist_hash<T>(obj: T) -> u32
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    let retu: u64 = hasher.finish() / u32::MAX as u64;
    return retu.try_into().unwrap();
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg
) -> Result<Response, ContractError> {
    // let version = get_contract_version(deps.storage)?;
    // if version.contract != CONTRACT_NAME {
    //     return Err(ContractError::CannotMigrate {
    //         previous_contract: version.contract,
    //     });
    // }


    // STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
    //     state.price = 420000000u64;
    //     Ok(state)
    // })?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        whitelist_password: msg.whitelist_password.into(),
        public_mint_height: msg.public_mint_height.into(),
        contract: info.sender.clone(),
        staking_contract: deps.api.addr_validate(&msg.staking_contract)?,
        price: msg.price.clone(),
        token_uri: msg.token_uri.into(),
        max_supply: msg.max_supply.clone(),
        mint_limit: msg.mint_limit.into(),
        owner: info.sender.clone(),
        nft_name: msg.nft_name.into(),
        nft_description: msg.nft_description.into(),
        image_ipfs: msg.image_ipfs.into(),
        external_url: msg.external_url.into(),
        animation_url: msg.animation_url.into(),
        youtube_url: msg.youtube_url.into(),
        background_color: msg.background_color.into(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("price", msg.price.to_string())
        .add_attribute("max_supply", msg.max_supply.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetWhitelistPassword { password } => {
            set_whitelist_password(deps, info, password)
        }
        ExecuteMsg::SetContract { contract } => set_contract(deps, info, contract),
        ExecuteMsg::Release {} => release(deps, env, info),
        ExecuteMsg::SetPrice {price} => set_price(deps, info, price),
        ExecuteMsg::MintOnBehalf { password } => mint_on_behalf(deps, env, info, password),
        ExecuteMsg::MultiMintOnBehalf { quantity, password } => {
            mint_on_behalf_multi(deps, env, info, quantity, password)
        },
        ExecuteMsg::UpdateConfig { config } => update_config(deps, info, config),
        // ExecuteMsg::OgClaimMint { token_id } => og_claim_mint(deps, env, info, token_id),
    }
}

fn set_whitelist_password(
    deps: DepsMut,
    info: MessageInfo,
    whitelist_password: Option<String>,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    check_owner(state.owner, info.sender.clone())?;

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.whitelist_password = whitelist_password;
        Ok(state)
    })?;

    let resp = Response::new().add_attribute("action", "set_whitelist_password");
    Ok(resp)
}

pub fn set_contract(
    deps: DepsMut,
    info: MessageInfo,
    contract: String,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    check_owner(state.owner, info.sender.clone())?;

    let contract_addr = deps.api.addr_validate(&contract)?;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.contract = contract_addr;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "set_contract"))
}

pub fn set_price(deps: DepsMut, info: MessageInfo, price: String) -> Result<Response, ContractError> {
    let price = price.parse::<u64>().unwrap();
    let state = STATE.load(deps.storage)?;
    check_owner(state.owner, info.sender.clone())?;

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.price = price;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "set_price"))
}

pub fn update_config(deps: DepsMut, info: MessageInfo, update_config: UpdateConfigMsg) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;
    check_owner(state.owner.clone(), info.sender.clone())?;

    if update_config.animation_url.is_some() {
        state.animation_url = update_config.animation_url;
    }
    if update_config.background_color.is_some() {
        state.background_color = update_config.background_color;
    }
    if update_config.contract.is_some() {
        state.contract = deps.api.addr_validate(&update_config.contract.unwrap())?;
    }
    if update_config.staking_contract.is_some() {
        state.staking_contract = deps.api.addr_validate(&update_config.staking_contract.unwrap())?;
    }
    if update_config.external_url.is_some() {
        state.external_url = update_config.external_url;
    }
    if update_config.image_ipfs.is_some() {
        state.image_ipfs = update_config.image_ipfs;
    }
    if update_config.max_supply.is_some() {
        state.max_supply = update_config.max_supply.unwrap() as u64;
    }
    if update_config.mint_limit.is_some() {
        state.mint_limit = update_config.mint_limit.unwrap();
    }
    if update_config.nft_description.is_some() {
        state.nft_description = update_config.nft_description.unwrap();
    }
    if update_config.nft_name.is_some() {
        state.nft_name = update_config.nft_name.unwrap();
    }
    if update_config.price.is_some() {
        state.price = update_config.price.unwrap();
    }
    if update_config.public_mint_height.is_some() {
        state.public_mint_height = update_config.public_mint_height.unwrap();
    }
    if update_config.token_uri.is_some() {
        state.token_uri = update_config.token_uri;
    }
    if update_config.whitelist_password.is_some() {
        state.whitelist_password = update_config.whitelist_password;
    }
    if update_config.youtube_url.is_some() {
        state.youtube_url = update_config.youtube_url;
    }
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "update_config"))
}

pub fn release(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // code to ensure that if anything happens, funds can be withdrawn
    let state = STATE.load(deps.storage)?;
    check_owner(state.owner.clone(), info.sender.clone())?;

    let balance = deps.querier.query_all_balances(_env.contract.address)?;

    let resp = Response::new()
        .add_attribute("action", "release")
        .add_message(BankMsg::Send {
            to_address: state.owner.to_string(),
            amount: balance,
        });
    Ok(resp)
}

//Result<TokenInfo<T>, ContractError> Result<Response, ContractError> {
pub fn mint_on_behalf_multi(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    quantity: u32,
    password: Option<u32>,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    if state.public_mint_height > _env.block.height {
        if state.whitelist_password.is_some() {
            match password {
                Some(password) => {
                    let mut add = info.sender.to_string().to_owned();
                    add.push_str(&state.whitelist_password.unwrap_or_default());
                    if password != whitelist_hash(add) {
                        return Err(ContractError::Unauthorized {});
                    }
                }
                None => return Err(ContractError::Minttime {}),
            }
        } else {
            return Err(ContractError::Minttime {});
        }
    }

    let minter_addr = deps.api.addr_validate(&info.sender.as_str())?;
    let minted = MINTED
        .may_load(deps.storage, &minter_addr)?
        .unwrap_or_default();

    if minted + quantity > state.mint_limit {
        return Err(ContractError::Overmint {});
    }

    let price: u128 = state.price.into();

    let quantity_into: u128 = quantity.into();
    let price_total: u128 = (price * quantity_into).into();
    if !has_coins(&info.funds, &coin(price_total, "uluna")) {
        return Err(ContractError::Insufficient {});
    }

    let response = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.contract.to_string(),
        msg: to_binary(&CW721_MetaData_QueryMsg::NumTokens {}).unwrap(),
    });

    let reply: NumTokensResponse = deps.querier.query(&response).unwrap();
    let quantity_checker: u64 = quantity.into();
    let mut messages: Vec<CosmosMsg> = vec![];
    if (reply.count + quantity_checker) > state.max_supply {
        return Err(ContractError::Overmint {});
    }

    let mut counter: u64 = 0;
    let token_uri = state.token_uri;
    let image_ipfs = state.image_ipfs;
    let external_url = state.external_url;
    let animation_url = state.animation_url;
    let youtube_url = state.youtube_url;
    let background_color = state.background_color;

    for _ in 0..quantity {
        let token_id = reply.count + counter;

        if "LunaPunks".eq(&state.nft_name) {
            let extension = None;

            let data = &CW721_MetaData_ExecuteMsg::Mint(CW721_MetaData_MintMsg::<Extension> {
                token_id: (token_id).to_string(),
                token_uri: None,
                owner: info.sender.clone().to_string(),
                extension: extension,
            });

            messages.push(
                WasmMsg::Execute {
                    contract_addr: state.contract.to_string(),
                    msg: to_binary(data)?,
                    funds: Vec::new(),
                }
                .into(),
            );
        } else {
            let extension: Extension = Some(Metadata {
                name: Some(format!("{}: #{}", state.nft_name, token_id.to_string())),
                description: Some(format!("{}", state.nft_description)),
                image: image_ipfs.clone(),
                external_url: external_url.clone(),
                animation_url: animation_url.clone(),
                youtube_url: youtube_url.clone(),
                background_color: background_color.clone(),
                attributes: None,
                image_data: None,
            });

            let mut metadata = extension.unwrap();
            if metadata.image.is_some() {
                metadata.image = Some(format!(
                    "{}{}",
                    metadata.image.unwrap(),
                    token_id.to_string()
                ));
            };
            if metadata.external_url.is_some() {
                metadata.external_url = Some(format!(
                    "{}{}",
                    metadata.external_url.unwrap(),
                    token_id.to_string()
                ));
            };
            if metadata.animation_url.is_some() {
                metadata.animation_url = Some(format!(
                    "{}{}",
                    metadata.animation_url.unwrap(),
                    token_id.to_string()
                ));
            };
            if metadata.youtube_url.is_some() {
                metadata.youtube_url = Some(format!(
                    "{}{}",
                    metadata.youtube_url.unwrap(),
                    token_id.to_string()
                ));
            };

            let mut full_token_uri = None;
            let token_uri_clone = token_uri.clone();
            if token_uri_clone.is_some() {
                full_token_uri = Some(format!(
                    "{}{}",
                    token_uri_clone.unwrap(),
                    token_id.to_string()
                ));
            }

            let data = &CW721_MetaData_ExecuteMsg::Mint(CW721_MetaData_MintMsg::<Extension> {
                token_id: (token_id).to_string(),
                token_uri: full_token_uri,
                owner: info.sender.clone().to_string(),
                extension: Some(metadata),
            });

            messages.push(
                WasmMsg::Execute {
                    contract_addr: state.contract.to_string(),
                    msg: to_binary(data)?,
                    funds: Vec::new(),
                }
                .into(),
            );
        }
        counter += 1;
    }

    MINTED.save(deps.storage, &minter_addr, &(minted + quantity))?;

    let amount_to_stake = 100_000_000u128;
    let amount_to_earn = price_total - amount_to_stake;

    let balance_to_stake = BankMsg::Send {
        to_address: state.staking_contract.to_string(),
        amount: coins(amount_to_stake, "uluna"),
    };
    let balance_to_earn = BankMsg::Send {
        to_address: state.owner.to_string(),
        amount: coins(amount_to_earn, "uluna"),
    };
    messages.push(balance_to_earn.into());
    messages.push(balance_to_stake.into());

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "execute")
        .add_attribute("minter", info.sender)
        .add_attribute("quantity", quantity.to_string()))
}

pub fn mint_on_behalf(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    password: Option<u32>,
) -> Result<Response, ContractError> {
    return mint_on_behalf_multi(deps, env, info, 1u32, password);
}

// pub fn og_claim_mint(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     token_id: u32,
// ) -> Result<Response, ContractError> {
//     // add check if tokenId is within OG limit
//     if token_id < 0 && token_id > 831 {
//         return Err(ContractError::NotOGPunk {});
//     }
//     if !has_coins(&info.funds, &coin(1000000, "uluna")) {
//         return Err(ContractError::Insufficient {});
//     }
//     // add check if already claimed
//     let claim = CLAIMED2.may_load(deps.storage, token_id.into())?;
//     if claim.is_some() {
//         return Err(ContractError::OGClaimed1 {});
//     }
//     // add check if owner of
//     check_can_send(deps.as_ref(), token_id, None, info.sender.clone())?;
//     CLAIMED2.save(deps.storage, token_id.into(), &info.sender.clone())?;

//     let state = STATE.load(deps.storage)?;

//     let minter_addr = deps.api.addr_validate(&info.sender.as_str())?;
//     let minted = MINTED
//         .may_load(deps.storage, &minter_addr)?
//         .unwrap_or_default();

//     let quantity = 1;

//     let response = QueryRequest::Wasm(WasmQuery::Smart {
//         contract_addr: state.contract.to_string(),
//         msg: to_binary(&CW721_MetaData_QueryMsg::NumTokens {}).unwrap(),
//     });

//     let reply: NumTokensResponse = deps.querier.query(&response).unwrap();
//     let quantity_checker: u64 = quantity.into();
//     let mut messages: Vec<CosmosMsg> = vec![];
//     if (reply.count + quantity_checker) > state.max_supply {
//         return Err(ContractError::Overmint {});
//     }

//     let mut counter: u64 = 0;
//     let token_uri = state.token_uri;
//     let image_ipfs = state.image_ipfs;
//     let external_url = state.external_url;
//     let animation_url = state.animation_url;
//     let youtube_url = state.youtube_url;
//     let background_color = state.background_color;

//     for _ in 0..quantity {
//         let token_id = reply.count + counter;

//         let extension = Some(Metadata {
//             name: Some(format!("{}: #{}", state.nft_name, token_id.to_string())),
//             description: None,
//             image: None,
//             image_data: None,
//             external_url: None,
//             attributes: None,
//             background_color: None,
//             animation_url: None,
//             youtube_url: None,
//         });

//         let data = &CW721_MetaData_ExecuteMsg::Mint(CW721_MetaData_MintMsg::<Extension> {
//             token_id: (token_id).to_string(),
//             token_uri: None,
//             owner: info.sender.clone().to_string(),
//             extension: extension,
//         });

//         messages.push(
//             WasmMsg::Execute {
//                 contract_addr: state.contract.to_string(),
//                 msg: to_binary(data)?,
//                 funds: Vec::new(),
//             }
//             .into(),
//         );
//         counter += 1;
//     }

//     MINTED.save(deps.storage, &minter_addr, &(minted + quantity))?;

//     Ok(Response::new()
//         .add_messages(messages)
//         .add_attribute("action", "og_claim")
//         .add_attribute("minter", info.sender)
//         .add_attribute("quantity", quantity.to_string()))
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetMaxSupply {} => to_binary(&query_max_supply(deps)?),
        QueryMsg::GetContractBalance {} => to_binary(&query_contract_balance(deps, _env)?),
        QueryMsg::IsClaimed { token_id } => to_binary(&query_is_claimed(deps, token_id)?),
    }
}

fn query_is_claimed(deps: Deps, token_id: u32) -> StdResult<IsClaimedResponse> {
    // add check if already claimed
    let claim = CLAIMED2.may_load(deps.storage, token_id.into())?;
    if claim.is_some() {
        return Ok(IsClaimedResponse { claimed: true });
    }

    Ok(IsClaimedResponse { claimed: false })
}

fn query_max_supply(deps: Deps) -> StdResult<MaxSupplyResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(MaxSupplyResponse {
        max_supply: state.max_supply,
    })
}

fn query_contract_balance(deps: Deps, _env: Env) -> StdResult<GetContractBalanceResponse> {
    let balance = deps.querier.query_all_balances(_env.contract.address)?;

    Ok(GetContractBalanceResponse { amount: balance })
}

fn check_owner(owner: Addr, sender: Addr) -> Result<(), ContractError> {
    if sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok({})
}

pub fn check_can_send(
    deps: Deps,
    token_id: u32,
    include_expired: Option<bool>,
    sender: Addr,
) -> Result<OwnerOfResponse, ContractError> {
    let state = STATE.load(deps.storage)?;
    let response = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.contract.to_string(),
        msg: to_binary(&Cw721QueryMsg::OwnerOf { token_id: token_id.to_string(), include_expired: include_expired }).unwrap(),
    });

    let reply: OwnerOfResponse = deps.querier.query(&response).unwrap();

    if reply.owner.to_string() != sender.to_string() {
        return Err(ContractError::Unauthorized{});
    }

    Ok(reply)
}