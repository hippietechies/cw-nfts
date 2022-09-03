#[cfg(not(feature = "library"))]
use cosmwasm_std::Uint128;
use crate::msg::ExecuteMsg::Release;
use crate::msg::{MintInfoResponse};
use cosmwasm_std::Coin;
use cw0::NativeBalance;
use crate::msg::UpdateConfigMsg;
use crate::msg::MaxMintResponse;
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, has_coins, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdResult, WasmMsg, WasmQuery,
};

use cw0::Expiration;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GetContractBalanceResponse, InstantiateMsg, MaxSupplyResponse, NumTokensResponse,
    QueryMsg,
};
use crate::state::{State, MINTED, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lunapunks-launchpad-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use cw721_metadata_onchain::ExecuteMsg as CW721_MetaData_ExecuteMsg;
use cw721_metadata_onchain::MintMsg as CW721_MetaData_MintMsg;
use cw721_metadata_onchain::QueryMsg as CW721_MetaData_QueryMsg;
use cw721_metadata_onchain::{Extension, Metadata};

use std::cmp::max;
use std::collections::hash_map::DefaultHasher;
use std::convert::TryInto;
use std::ops::Rem;
use std::hash::{Hash, Hasher};

use seahash::hash;

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
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    if info.sender.to_string() != "terra1dkeg3uvglsgph0vqwz9ejyaye6nla3d8smlsxl" {
        return Err(ContractError::Unauthorized {});
    }

    let state = State {
        launch_owner: deps.api.addr_validate(&msg.launch_owner)?,
        whitelist_password: msg.whitelist_password.into(),
        start_after: msg.start_after.clone().unwrap_or(Expiration::AtHeight(env.block.height)),
        public_mint_height: msg.public_mint_height.into(),
        contract: info.sender.clone(),
        staking_contract: deps.api.addr_validate(&msg.staking_contract)?,
        price_bag: msg.price_bag.clone(),
        token_uri: msg.token_uri.into(),
        unminted: (1..=msg.max_supply).collect(),
        max_supply: msg.max_supply.clone(),
        mint_limit: msg.mint_limit.into(),
        owner: info.sender.clone(),
        nft_name: msg.nft_name.into(),
        nft_description: msg.nft_description.into(),
        image_ipfs: msg.image_ipfs.into(),
        attributes_ipfs: msg.attributes_ipfs.into(),
        external_url: msg.external_url.into(),
        animation_url: msg.animation_url.into(),
        youtube_url: msg.youtube_url.into(),
        background_color: msg.background_color.into(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("launch_owner", msg.launch_owner)
        .add_attribute("owner", info.sender)
        // .add_attribute("price", msg.price_bag)
        .add_attribute("max_supply", msg.max_supply.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetWhitelistPassword { password } => {
            set_whitelist_password(deps, info, password)
        }
        ExecuteMsg::SetContract { contract } => set_contract(deps, info, contract),
        ExecuteMsg::Release {} => release(deps, _env, info),
        ExecuteMsg::MintOnBehalf { password } => mint_on_behalf(deps, _env, info, password),
        ExecuteMsg::MultiMintOnBehalf { quantity, password } => {
            mint_on_behalf_multi(deps, _env, info, quantity, password)
        }
    }
}

fn set_whitelist_password(
    deps: DepsMut,
    info: MessageInfo,
    whitelist_password: Option<String>,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    check_owner(state.launch_owner,  state.owner, info.sender.clone())?;

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
    check_owner(state.launch_owner,  state.owner, info.sender.clone())?;

    let contract_addr = deps.api.addr_validate(&contract)?;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.contract = contract_addr;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "set_contract"))
}

pub fn update_config(deps: DepsMut, info: MessageInfo, update_config: UpdateConfigMsg) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;
    check_owner(state.launch_owner.clone(),  state.owner.clone(), info.sender.clone())?;

    if update_config.animation_url.is_some() {
        state.animation_url = update_config.animation_url;
    }
    if update_config.attributes_ipfs.is_some() {
        state.attributes_ipfs = update_config.attributes_ipfs;
    }
    if update_config.background_color.is_some() {
        state.background_color = update_config.background_color;
    }
    if update_config.contract.is_some() {
        state.contract = deps.api.addr_validate(&update_config.contract.unwrap())?;
    }
    if update_config.external_url.is_some() {
        state.external_url = update_config.external_url;
    }
    if update_config.image_ipfs.is_some() {
        state.image_ipfs = update_config.image_ipfs;
    }
    if update_config.launch_owner.is_some() {
        state.launch_owner = deps.api.addr_validate(&update_config.launch_owner.unwrap())?;
    }
    if update_config.max_supply.is_some() {
        state.max_supply = update_config.max_supply.unwrap();
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
    if update_config.price_bag.is_some() {
        state.price_bag = update_config.price_bag.unwrap();
    }
    if update_config.public_mint_height.is_some() {
        state.public_mint_height = update_config.public_mint_height.unwrap();
    }
    if update_config.start_after.is_some() {
        state.start_after = update_config.start_after.unwrap();
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

pub fn set_price(deps: DepsMut, info: MessageInfo, price_bag: Vec<Coin>) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    check_owner(state.launch_owner,  state.owner, info.sender.clone())?;

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.price_bag = price_bag;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "set_price"))
}

// instead of calling release, auto send funds to wallet
pub fn release(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // code to ensure that if anything happens, funds can be withdrawn
    let state = STATE.load(deps.storage)?;

    let balance = deps.querier.query_all_balances(_env.contract.address)?;


    let zero = Uint128::from(0u128);
    let base = Uint128::from(10000u128);
    let platform_fee = Uint128::from(1000u128);
    let mut fee: Vec<Coin> = vec![];
    let mut earnings: Vec<Coin> = vec![];
    for (_, coin) in balance.iter().enumerate() {
        let platform_amount = coin.amount.clone().checked_mul(platform_fee).ok().unwrap_or(zero).checked_div(base).ok().unwrap_or(zero);
        if platform_amount.gt(&Uint128::from(0u128)) {
            fee.push(Coin::new(platform_amount.u128(), coin.denom.to_string()));
        }

        earnings.push(Coin::new(coin.amount.saturating_sub(platform_amount).u128(), coin.denom.to_string()));
    }

    let mut messages: Vec<BankMsg> = vec![];

    if fee.len() > 0 {
        messages.push(BankMsg::Send {
            to_address: state.staking_contract.to_string(),
            amount: fee
        });
    }
    if earnings.len() > 0 {
        messages.push(BankMsg::Send {
            to_address: state.launch_owner.to_string(),
            amount: earnings
        });
    }

    let resp = Response::new()
        .add_attribute("action", "release")
        .add_messages(messages);
    Ok(resp)
}

//Result<TokenInfo<T>, ContractError> Result<Response, ContractError> {
pub fn mint_on_behalf_multi(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    quantity: u32,
    password: Option<u32>,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    if !state.start_after.is_expired(&env.block) {
        if state.whitelist_password.is_some() {
            match password {
                Some(password) => {
                    let mut add = info.sender.to_string().to_owned();
                    add.push_str(&state.whitelist_password.clone().unwrap_or_default());
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

    let quantity_into: u128 = quantity.into();
    for coin in &state.price_bag {
        if !has_coins(&info.funds, &Coin::new(coin.amount.u128().saturating_mul(quantity_into), coin.denom.clone())) {
            return Err(ContractError::Insufficient {});
        }
    }

    let unminted = state.unminted.len() as u64;
    let quantity_checker: u64 = quantity.into();
    if (unminted - quantity_checker) < 0 {
        return Err(ContractError::Overmint {});
    }

    let token_uri = state.token_uri.clone();

    let mut messages: Vec<CosmosMsg> = vec![];
    for count in 0..quantity {
        let random_lot = hash((info.sender.to_string() + &count.to_string() + &env.block.time.subsec_nanos().to_string()).as_bytes());
        let unminted_index = random_lot.wrapping_rem_euclid(state.unminted.len() as u64) as usize;
        let token_id = state.unminted.swap_remove(unminted_index);
        STATE.save(deps.storage, &state)?;

        // let extension: Extension = Some(Metadata {
        //     name: Some(state.nft_name.clone()),
        //     description: Some(state.nft_description.clone()),
        //     image: state.image_ipfs.clone(),
        //     external_url: state.external_url.clone(),
        //     animation_url: state.animation_url.clone(),
        //     youtube_url: state.youtube_url.clone(),
        //     background_color: state.background_color.clone(),
        //     attributes: None,
        //     image_data: None,
        // });

        // let mut metadata = extension.unwrap();
        // if metadata.image.is_some() {
        //     metadata.image = Some(format!(
        //         "{}{}",
        //         metadata.image.unwrap(),
        //         token_id.to_string()
        //     ));
        // };
        // if metadata.external_url.is_some() {
        //     metadata.external_url = Some(format!(
        //         "{}{}",
        //         metadata.external_url.unwrap(),
        //         token_id.to_string()
        //     ));
        // };
        // if metadata.animation_url.is_some() {
        //     metadata.animation_url = Some(format!(
        //         "{}{}",
        //         metadata.animation_url.unwrap(),
        //         token_id.to_string()
        //     ));
        // };
        // if metadata.youtube_url.is_some() {
        //     metadata.youtube_url = Some(format!(
        //         "{}{}",
        //         metadata.youtube_url.unwrap(),
        //         token_id.to_string()
        //     ));
        // };

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
            extension: None,
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

    messages.push(
        WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&Release{})?,
            funds: Vec::new(),
        }
        .into(),
    );

    // messages.push(
    //     WasmMsg::Execute {
    //         contract_addr: ,
    //         msg: to_binary(&StakingMsg::Revest {})?,
    //         funds: Vec::new(),
    //     }
    //     .into(),
    // );

    MINTED.save(deps.storage, &minter_addr, &(minted + quantity))?;

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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetMaxSupply {} => to_binary(&query_max_supply(deps)?),
        QueryMsg::GetMaxMint {} => to_binary(&query_max_mint(deps)?),
        QueryMsg::GetMintInfo {} => to_binary(&query_mint_info(deps)?),
        QueryMsg::GetContractBalance {} => to_binary(&query_contract_balance(deps, _env)?),
    }
}

fn query_max_supply(deps: Deps) -> StdResult<MaxSupplyResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(MaxSupplyResponse {
        max_supply: state.max_supply,
    })
}
fn query_max_mint(deps: Deps) -> StdResult<MaxMintResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(MaxMintResponse {
        max_mint: state.mint_limit,
    })
}
fn query_mint_info(deps: Deps) -> StdResult<MintInfoResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(MintInfoResponse {
        price_bag: Some(state.price_bag),
    })
}

fn query_contract_balance(deps: Deps, _env: Env) -> StdResult<GetContractBalanceResponse> {
    let balance = deps.querier.query_all_balances(_env.contract.address)?;

    Ok(GetContractBalanceResponse { amount: balance })
}

fn check_owner(launch_owner: Addr, owner:Addr, sender: Addr) -> Result<(), ContractError> {
    if sender == owner || sender == launch_owner {
        return Ok({});
    }
    return Err(ContractError::Unauthorized {});
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
//     use cosmwasm_std::{coins, from_binary};

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies(&[]);

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(1000, "earth"));

//         // we can just call .unwrap() to assert this was a success
//         let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // it worked, let's query the state
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(17, value.count);
//     }

//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies(&coins(2, "token"));

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Increment {};
//         let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // should increase counter by 1
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(18, value.count);
//     }

//     #[test]
//     fn reset() {
//         let mut deps = mock_dependencies(&coins(2, "token"));

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let unauth_info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
//         match res {
//             Err(ContractError::Unauthorized {}) => {}
//             _ => panic!("Must return unauthorized error"),
//         }

//         // only the original creator can reset the counter
//         let auth_info = mock_info("creator", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

//         // should now be 5
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(5, value.count);
//     }
// }
