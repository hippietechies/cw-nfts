use crate::msg::{StakingMsg, MigrateMsg};
use cw2::get_contract_version;
use cw0::NativeBalance;
use crate::state::{Token, BagOfCoins, State};
use std::ops::Sub;
use cw_storage_plus::U32Key;

use cosmwasm_std::{Coin, Addr, Uint128, WasmQuery, WasmMsg, to_binary, CosmosMsg, QueryRequest, BankMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use cw2::set_contract_version;
use cw721::{OwnerOfResponse, Cw721ExecuteMsg, Expiration, Cw721QueryMsg};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{MarketContract};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lunapunks-launchpad-market";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

impl<'a> MarketContract<'a> {
    pub fn instantiate(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {

        let mut platform_fee = 100;
        if info.sender.to_string() != "terra1dkeg3uvglsgph0vqwz9ejyaye6nla3d8smlsxl" {
            return Err(ContractError::Unauthorized {});
            // return Err(ContractError::Unauthorized {});
        }

        let state = State {
            launch_owner: deps.api.addr_validate(&msg.launch_owner)?,
            contract: deps.api.addr_validate(&msg.contract)?,
            staking_contract: deps.api.addr_validate(&msg.staking_contract)?,
            owner: info.sender.clone(),
            platform_fee: 100, // 1% platform fee
            platform_wallet: deps.api.addr_validate(&msg.platform_wallet)?,
            royalty_fee: msg.royalty_fee.into(),
            royalty_wallet: deps.api.addr_validate(&msg.royalty_wallet)?,
            royalties: vec![],
        };
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        self.state.save(deps.storage, &state)?;

        Ok(Response::new()
            .add_attribute("method", "instantiate")
            .add_attribute("owner", info.sender))
    }

    pub fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::Release { release_funds } => self.release(deps, env, info, release_funds),
            ExecuteMsg::BidAddNft { token_id, expires } => self.bid_add_nft(deps, env, info, token_id, expires),
            ExecuteMsg::BidWithdrawNft { token_id } => self.bid_withdraw_nft(deps, info, token_id),
            ExecuteMsg::BidAcceptNft { token_id, bidder_address } => self.bid_accept_nft(deps, env, info, token_id, bidder_address),
            ExecuteMsg::AskAddNft { token_id, ask_funds, expires } => self.ask_add_nft(deps, env, info, token_id, ask_funds, expires),
            ExecuteMsg::AskWithdrawNft { token_id } => self.ask_withdraw_nft(deps, info, token_id),
            ExecuteMsg::AskAcceptNft { token_id } => self.ask_accept_nft(deps, env, info, token_id),
            ExecuteMsg::SetRoyaltyWallet { royalty_wallet } => self.set_royalty_wallet(deps, info, royalty_wallet),
            ExecuteMsg::SetRoyaltyFee { royalty_fee } => self.set_royalty_fee(deps, info, royalty_fee),
            ExecuteMsg::SetContract { contract } => self.set_contract(deps, info, contract),
        }
    }

    pub fn migrate(
        &self,
        deps: DepsMut,
        _env: Env,
        _msg: MigrateMsg
    ) -> Result<Response, ContractError> {
        let version = get_contract_version(deps.storage)?;
        if version.contract != CONTRACT_NAME {
            return Err(ContractError::CannotMigrate {
                previous_contract: version.contract,
            });
        }

        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        Ok(Response::default())
    }
}

// TODO pull this into some sort of trait extension??
impl<'a> MarketContract<'a>
{
    fn bid_add_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: u32,
        expires: Option<Expiration>,
    ) -> Result<Response, ContractError> {
        // check state for existing bids
        // let mut token = TOKENMAP.load(deps.storage, token_id.into())?;

        // check expiry
        let expires = expires.unwrap_or_default();
        if expires.is_expired(&env.block) {
            return Err(ContractError::Expired {});
        }
        // ensure bid has funds added
        if info.funds.is_empty() {
            return Err(ContractError::Unfunded {});
        }

        let mut token = self.token_map.may_load(deps.storage, token_id.into())?.unwrap_or(Token {
            token_id: token_id.into(),
            ask: None,
            bids: vec![],
        });

        let mut bag_of_coins = None;
        let mut index = None;
        let mut msgs: Vec<CosmosMsg> = vec![];

        // get previous bid to overwrite
        for (pos, bid) in token.bids.iter().enumerate() {
            if bid.owner == info.sender {
                bag_of_coins = Some(bid.bag.clone());
                index = Some(pos);
            }
        }
        // return previous bid
        if index.is_some() {
            token.bids.remove(index.unwrap());
            msgs.push(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: bag_of_coins.unwrap()
            }.into())
        }
        token.bids.push(BagOfCoins { owner: info.sender.clone(), bag: info.funds, expires: expires });
        self.token_map.save(deps.storage, token_id.into(), &token)?;
        self.bid_map.save(deps.storage, (info.sender.clone(), token_id.to_string().as_bytes().to_vec()), &token)?;

        // let state = &self.state.load(deps.storage)?;
        // msgs.push(self.get_revest_msg(state.staking_contract.to_string())?.into());

        // set new bid
        Ok(Response::new()
            .add_attribute("action", "bid_add_nft")
            .add_attribute("bidder", info.sender.clone())
            .add_attribute("token_id", token_id.to_string())
            .add_messages(msgs))
    }

    fn bid_withdraw_nft(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        token_id: u32,
    ) -> Result<Response, ContractError> {
        // check state for existing bids by sender
        let mut token = self.token_map.may_load(deps.storage, token_id.into())?.unwrap_or(Token {
            token_id: token_id.into(),
            ask: None,
            bids: vec![],
        });

        // remove bid
        let mut bag_of_coins = None;
        let mut index = None;

        // get previous bid to overwrite
        for (pos, bid) in token.bids.iter().enumerate() {
            if bid.owner == info.sender {
                bag_of_coins = Some(bid.bag.clone());
                index = Some(pos);
            }
        }
        if index.is_some() {
            token.bids.remove(index.unwrap());
        } else {
            return Err(ContractError::UnknownAddress {});
        }
        self.token_map.save(deps.storage, token_id.into(), &token)?;
        self.bid_map.remove(deps.storage, (info.sender.clone(), token_id.to_string().as_bytes().to_vec()));

        Ok(Response::new()
            .add_attribute("action", "bid_withdraw_nft")
            .add_attribute("bidder", info.sender.clone())
            .add_attribute("token_id", token_id.to_string())
            .add_message(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: bag_of_coins.unwrap()
            }))
    }
    fn bid_accept_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: u32,
        bidder_address: String,
    ) -> Result<Response, ContractError> {
        let bidder_address = deps.api.addr_validate(&bidder_address)?;
        // check state for existing bids
        let mut token = self.token_map.may_load(deps.storage, token_id.into())?.unwrap_or(Token {
            token_id: token_id.into(),
            ask: None,
            bids: vec![],
        });
        let state = self.state.load(deps.storage)?;

        let mut bag_of_coins = None;
        let mut index = None;
        let mut messages: Vec<CosmosMsg> = vec![];

        let owner_response = self.check_can_send(deps.as_ref(), token_id, Some(false), info.sender.clone(), false)?;
        for (pos, bid) in token.bids.iter().enumerate() {
            if bid.owner == bidder_address {
                if bid.expires.is_expired(&env.block) {
                    return Err(ContractError::Expired {});
                }
                bag_of_coins = Some(bid.clone());
                index = Some(pos);
            }
        }
        // remove accepted bid
        if index.is_some() {
            token.bids.remove(index.unwrap());
            self.token_map.save(deps.storage, token_id.into(), &token)?;
            self.bid_map.remove(deps.storage, (bidder_address.clone(), token_id.to_string().as_bytes().to_vec()));
        } else {
            // throw error if bid not present
            return Err(ContractError::UnknownAddress {});
        }

        let zero = Uint128::from(0u128);
        let base = Uint128::from(10000u128);
        let royalty_fee = Uint128::from(state.royalty_fee);
        let platform_fee = Uint128::from(state.platform_fee);
        let mut fee: Vec<Coin> = vec![];
        let mut royalty: Vec<Coin> = vec![];
        let mut earnings: Vec<Coin> = vec![];
        for (_, coin) in bag_of_coins.unwrap().bag.iter().enumerate() {
            let platform_amount = coin.amount.clone().checked_mul(platform_fee).ok().unwrap_or(zero).checked_div(base).ok().unwrap_or(zero);
            if platform_amount.gt(&Uint128::from(0u128)) {
                fee.push(Coin::new(platform_amount.u128(), coin.denom.to_string()));
            }

            let royalty_amount = coin.amount.clone().checked_mul(royalty_fee).ok().unwrap_or(zero).checked_div(base).ok().unwrap_or(zero);
            if royalty_amount.gt(&Uint128::from(0u128)) {
                royalty.push(Coin::new(royalty_amount.u128(), coin.denom.to_string()));
            }

            earnings.push(Coin::new(coin.amount.saturating_sub(platform_amount).saturating_sub(royalty_amount).u128(), coin.denom.to_string()));
        }

        // send funds of winning bid to owner of nft
        messages.push(BankMsg::Send {
            to_address: owner_response.owner.to_string(),
            amount: earnings
        }.into());

        if fee.len() > 0 {
            // send funds of winning bid to platform owner of nft
            messages.push(BankMsg::Send {
                to_address: state.platform_wallet.to_string(),
                amount: fee
            }.into());
        }
        if royalty.len() > 0 {
            // send funds of winning bid to platform owner of nft
            messages.push(BankMsg::Send {
                to_address: state.royalty_wallet.to_string(),
                amount: royalty
            }.into());
        }

        // remove owner's ask if present
        if token.ask.is_some() {
            token.ask = None;
            self.token_map.save(deps.storage, token_id.into(), &token)?;
        }

        // transfer NFT to bidder
        let transfer_msg = &Cw721ExecuteMsg::TransferNft {
            token_id: (token_id).to_string(),
            recipient: bidder_address.clone().to_string(),
        };
        messages.push(
            WasmMsg::Execute {
                contract_addr: state.contract.to_string(),
                msg: to_binary(transfer_msg)?,
                funds: Vec::new(),
            }
            .into(),
        );


        // let state = &self.state.load(deps.storage)?;
        // messages.push(self.get_revest_msg(state.staking_contract.to_string())?.into());

        // send coins to owner
        Ok(Response::new()
            .add_attribute("action", "bid_accept_nft")
            .add_attribute("winning_bid", info.sender.to_string())
            .add_attribute("token_id", token_id.to_string())
            .add_messages(messages)
        )
    }

    fn ask_add_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: u32,
        ask_funds: Vec<Coin>,
        expires: Option<Expiration>,
    ) -> Result<Response, ContractError> {
        // check state for existing bids
        let expires = expires.unwrap_or_default();
        if expires.is_expired(&env.block) {
            return Err(ContractError::Expired {});
        }

        self.check_can_send(deps.as_ref(), token_id, Some(false), info.sender.clone(), false)?;

        let bag = BagOfCoins { owner: info.sender.clone(), bag: ask_funds, expires: expires };
        // let token = self.token_map.may_load(deps.storage, token_id.into()).unwrap();

        let mut token = self.token_map.may_load(deps.storage, token_id.into())?.unwrap_or(Token {
            token_id: token_id.into(),
            ask: None,
            bids: vec![],
        });
        token.ask = Some(bag);
        self.token_map.save(deps.storage, U32Key::from(token_id), &token)?;

        // self.token_map
        //     .update(deps.storage, U32Key::from(token_id), |old| match old {
        //         Some(token) => {
        //             Err(ContractError::Claimed {})
        //             // token.ask = Some(bag);
        //             // Ok(token)
        //         },
        //         None => Ok(Token {
        //             token_id: token_id.into(),
        //             owner: info.sender.clone(),
        //             ask: Some(bag),
        //             bid: vec![],
        //         }),
        //     });


        Ok(Response::new()
            .add_attribute("action", "ask_add_nft")
            .add_attribute("asker", info.sender)
            .add_attribute("token_id", token_id.to_string())
        )
    }
    fn ask_withdraw_nft(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        token_id: u32,
    ) -> Result<Response, ContractError> {
        // check state for existing bids by sender
        self.check_can_send(deps.as_ref(), token_id, Some(false), info.sender.clone(), false)?;

        // save state
        let mut token = self.token_map.may_load(deps.storage, token_id.into())?.unwrap_or(Token {
            token_id: token_id.into(),
            ask: None,
            bids: vec![],
        });
        token.ask = None;
        self.token_map.save(deps.storage, U32Key::from(token_id), &token)?;

        Ok(Response::new()
            .add_attribute("action", "ask_withdraw_nft")
            .add_attribute("asker", info.sender)
            .add_attribute("token_id", token_id.to_string())
        )
    }
    // ask available, sender fulfills ask, nft to sender
    fn ask_accept_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: u32,
    ) -> Result<Response, ContractError> {
        // check state for existing bids
        // let token = &self.ask_map.load(deps.storage, token_id.into())?;
        let mut token = self.token_map.may_load(deps.storage, token_id.into())?.unwrap_or(Token {
            token_id: token_id.into(),
            ask: None,
            bids: vec![],
        });
        let state = &self.state.load(deps.storage)?;

        // get amount requested in ask
        let bag_of_coins = token.ask.clone();
        if bag_of_coins.is_none() {
            return Err(ContractError::UnknownAddress {});
        }

        // check if ask bag expired
        let bag_of_coins = bag_of_coins.unwrap().clone();
        if bag_of_coins.expires.is_expired(&env.block) {
            return Err(ContractError::Expired {});
        }

        let zero = Uint128::from(0u128);
        let base = Uint128::from(10000u128);
        let royalty_fee = Uint128::from(state.royalty_fee);
        let platform_fee = Uint128::from(state.platform_fee);
        let mut fee: Vec<Coin> = vec![];
        let mut royalty: Vec<Coin> = vec![];
        let mut earnings: Vec<Coin> = vec![];

        // ensure that buyer has enuogh funds as asked
        let buyer_funds = NativeBalance(info.funds.clone());
        for coin in &bag_of_coins.bag {
            if !buyer_funds.has(&coin) {
                return Err(ContractError::Unfunded {});
            }
            let platform_amount = coin.amount.clone().saturating_mul(platform_fee).checked_div(base).ok().unwrap_or(zero);
            if platform_amount.gt(&zero) {
                fee.push(Coin::new(platform_amount.u128(), coin.denom.to_string()));
            }

            let royalty_amount = coin.amount.clone().saturating_mul(royalty_fee).checked_div(base).ok().unwrap_or(zero);
            if royalty_amount.gt(&zero) {
                royalty.push(Coin::new(royalty_amount.u128(), coin.denom.to_string()));
            }
            earnings.push(Coin::new(coin.amount.saturating_sub(platform_amount).saturating_sub(royalty_amount).u128(), coin.denom.to_string()));
        }

        // ensure that owner of ask is current owner of NFT
        let owner = self.check_can_send(deps.as_ref(), token_id, Some(false), bag_of_coins.owner.clone(), true)?;

        // save state
        token.ask = None;
        self.token_map.save(deps.storage, U32Key::from(token_id), &token)?;

        let mut msgs: Vec<CosmosMsg> = vec![];
        // send funds of winning bid to owner of nft
        msgs.push(BankMsg::Send {
            to_address: owner.owner.to_string(),
            amount: earnings
        }.into());

        if fee.len() > 0 {
            // send funds of winning bid to platform owner of nft
            msgs.push(BankMsg::Send {
                to_address: state.platform_wallet.to_string(),
                amount: fee
            }.into());
        }
        if royalty.len() > 0 {
            // send funds of winning bid to platform owner of nft
            msgs.push(BankMsg::Send {
                to_address: state.royalty_wallet.to_string(),
                amount: royalty
            }.into());
        }

        // return back to buyer excess funds
        let refund = buyer_funds.sub(bag_of_coins.bag)?;
        if !refund.is_empty() {
            msgs.push(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: refund.into_vec()
            }.into())
        }

        // transfer NFT to bidder
        let transfer_msg = &Cw721ExecuteMsg::TransferNft {
            token_id: (token_id).to_string(),
            recipient: info.sender.clone().to_string(),
        };
        let state = &self.state.load(deps.storage)?;
        msgs.push(
            WasmMsg::Execute {
                contract_addr: state.contract.to_string(),
                msg: to_binary(transfer_msg)?,
                funds: Vec::new(),
            }
            .into(),
        );

        // msgs.push(self.get_revest_msg(state.staking_contract.to_string())?.into());

        // send coins to owner
        Ok(Response::new()
            .add_attribute("action", "ask_accept_nft")
            .add_attribute("buyer", info.sender)
            .add_attribute("token_id", token_id.to_string())
            .add_messages(msgs)
        )
    }
}

// helpers
impl<'a> MarketContract<'a>
{
    pub fn release(&self, deps: DepsMut, _env: Env, info: MessageInfo, release_funds: Vec<Coin>) -> Result<Response, ContractError> {
        // code to ensure that if anything happens, funds can be withdrawn
        let state = self.state.load(deps.storage)?;
        self.check_owner(state.launch_owner.clone(), state.owner.clone(), info.sender.clone())?;

        let balance;
        if release_funds.len() == 0 {
            balance = deps.querier.query_all_balances(_env.contract.address)?;
        } else {
            balance = release_funds;
        }

        let resp = Response::new()
            .add_attribute("action", "release")
            .add_message(BankMsg::Send {
                to_address: state.owner.to_string(),
                amount: balance,
            });
        Ok(resp)
    }

    pub fn check_owner(&self, launch_owner: Addr, owner: Addr, sender: Addr) -> Result<(), ContractError> {
        if sender == launch_owner || sender == owner {
            return Ok(())
        }
        return Err(ContractError::Unauthorized {});
    }

    pub fn check_can_send(
        &self,
        deps: Deps,
        token_id: u32,
        include_expired: Option<bool>,
        sender: Addr,
        is_ask: bool,
    ) -> Result<OwnerOfResponse, ContractError> {
        let state = self.state.load(deps.storage)?;
        let response = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: state.contract.to_string(),
            msg: to_binary(&Cw721QueryMsg::OwnerOf { token_id: token_id.to_string(), include_expired: include_expired }).unwrap(),
        });

        let reply: OwnerOfResponse = deps.querier.query(&response).unwrap();

        if reply.owner.to_string() != sender.to_string() {
            if is_ask {
                return Err(ContractError::UnknownAsk{});
            } else {
                return Err(ContractError::Unauthorized{});
            }
        }

        Ok(reply)
    }

    pub fn set_royalty_wallet(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        royalty_wallet: String,
    ) -> Result<Response, ContractError> {
        let state = self.state.load(deps.storage)?;
        self.check_owner(state.launch_owner, state.owner, info.sender.clone())?;

        let royalty_wallet_addr = deps.api.addr_validate(&royalty_wallet)?;
        self.state.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.royalty_wallet = royalty_wallet_addr.clone();
            Ok(state)
        })?;

        Ok(Response::new()
        .add_attribute("value", royalty_wallet_addr.to_string())
        .add_attribute("method", "set_royalty_address"))
    }

    pub fn set_royalty_fee(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        royalty_fee: u32,
    ) -> Result<Response, ContractError> {
        let state = self.state.load(deps.storage)?;
        self.check_owner(state.launch_owner,  state.owner, info.sender.clone())?;

        self.state.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.royalty_fee = royalty_fee;
            Ok(state)
        })?;

        Ok(Response::new()
        .add_attribute("value", royalty_fee.to_string())
        .add_attribute("method", "set_royalty_fee"))
    }

    pub fn set_contract(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        contract: String,
    ) -> Result<Response, ContractError> {
        let state = self.state.load(deps.storage)?;
        self.check_owner(state.launch_owner,  state.owner, info.sender.clone())?;

        let contract_addr = deps.api.addr_validate(&contract)?;
        self.state.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.contract = contract_addr;
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("method", "set_contract"))
    }

    pub fn get_revest_msg(&self, staking_contract: String) -> StdResult<WasmMsg> {
        let revest_msg = WasmMsg::Execute {
            contract_addr: staking_contract,
            msg: to_binary(&StakingMsg::Revest {})?,
            funds: Vec::new(),
        };
        Ok(revest_msg)
    }
}
