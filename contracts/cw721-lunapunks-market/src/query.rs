use crate::state::OwnerBidsResponse;
use cosmwasm_std::Addr;
use crate::state::RoyaltyInfoResponse;
use crate::state::AllNftPriceMapResponse;
use crate::state::Token;
use crate::state::NftMarketInfoResponse;
use crate::state::TokenMarketInfo;
use crate::state::AllNftMarketInfoResponse;

use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult};
use std::convert::{TryFrom,TryInto};
use std::marker::PhantomData;
use cw_storage_plus::Bound;

use crate::msg::{QueryMsg};
use crate::state::{MarketContract};

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

impl<'a> MarketContract<'a>
{
    pub fn calc_limit(&self, request: Option<u32>) -> usize {
        request.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize
    }
    pub fn calc_skip(&self, request: Option<u32>, limit: usize) -> usize {
        request.unwrap_or(0) as usize * limit
    }

    pub fn royalty_info(
        &self,
        deps: Deps,
    ) -> StdResult<RoyaltyInfoResponse> {
        let state = self.state.load(deps.storage)?;

        Ok(RoyaltyInfoResponse {
            royalty_fee: state.royalty_fee,
            royalty_wallet: state.royalty_wallet.to_string(),
        })
    }

    pub fn nft_market_info(
        &self,
        deps: Deps,
        env: Env,
        token_id: u32,
        include_expired: bool,
    ) -> StdResult<NftMarketInfoResponse> {
        let token = self.token_map.may_load(deps.storage, token_id.into())?.unwrap_or(Token {
            token_id: token_id.into(),
            ask: None,
            bids: vec![],
        });

        Ok(NftMarketInfoResponse {
            token: TokenMarketInfo {
                token_id: token_id,
                bids: token.bids
                .iter()
                .filter(|bid| (include_expired || !bid.expires.is_expired(&env.block)))
                .cloned()
                .collect(),
                ask: token.ask.filter(|ask| (include_expired || !ask.expires.is_expired(&env.block))),
            }
        })
    }

    pub fn all_nft_bids_info(
        &self,
        deps: Deps,
        env: Env,
        owner: String,
        include_expired: bool,
        start_after: Option<u32>,
        skip: Option<u32>,
        limit: Option<u32>,
    ) -> StdResult<OwnerBidsResponse> {
        let limit_usize = self.calc_limit(limit);
        let skip_usize = self.calc_skip(skip, limit_usize);
        let start = Some(Bound::Exclusive((start_after.unwrap_or(0).to_be_bytes().to_vec(), PhantomData)));
        let owner = deps.api.addr_validate(&owner)?;
        let tokens = self.bid_map
            .prefix(owner)
            .range(deps.storage, start, None, Order::Ascending)
            .skip(skip_usize)
            .take(limit_usize)
            .map(|item| item.ok().unwrap().1)
            .collect();

        Ok(OwnerBidsResponse{tokens})
    }

    pub fn all_nft_asks_info(
        &self,
        deps: Deps,
        env: Env,
        include_expired: bool,
        start_after: Option<u32>,
        skip: Option<u32>,
        limit: Option<u32>,
    ) -> StdResult<AllNftMarketInfoResponse> {
        let limit_usize = self.calc_limit(limit);
        let skip_usize = self.calc_skip(skip, limit_usize);
        let start = Some(Bound::Exclusive((start_after.unwrap_or(0), PhantomData)));

        let res: StdResult<Vec<TokenMarketInfo>> = self.token_map
            .range(deps.storage, start, None, Order::Ascending)
            .skip(skip_usize)
            .take(limit_usize)
            .map(|item| {
                item.and_then(|(key, token)| {
                    let token_id: [u8;4] = key.to_be_bytes();
                    let ask_block;
                    match token.ask {
                        Some(ask) => {
                            if include_expired || !ask.expires.is_expired(&env.block) {
                                ask_block = Some(ask)
                            } else {
                                ask_block = None
                            }
                        },
                        None => { ask_block = None }
                    };
                    Ok(TokenMarketInfo {
                        token_id: u32::from_be_bytes(token_id),
                        ask: ask_block,
                        bids: vec![]
                    })
                })
            })
            .collect();
        Ok(AllNftMarketInfoResponse {
            tokens: res?,
            limit: limit,
            count: Some(self.token_map.range(deps.storage, None, None, Order::Ascending).count().to_string()),
            start_after: start_after,
            is_ask: Some(true),
            is_bids: None,
        })
    }

    pub fn all_nft_sort_price(
        &self,
        deps: Deps,
        env: Env,
        ascending: Option<i32>,
        include_expired: bool,
        start_after: Option<String>,
        skip: Option<u32>,
        limit: Option<u32>,
    ) -> StdResult<AllNftPriceMapResponse> {
        let limit_usize = self.calc_limit(limit);
        let skip_usize = self.calc_skip(skip, limit_usize);
        let start = Some(Bound::Exclusive(((start_after.clone().unwrap_or("0".to_string()).parse::<u128>().unwrap().to_be_bytes().to_vec(), 0u32.to_be_bytes().to_vec()), PhantomData)));

        // let order = Order::try_from(1i32);
        let order = Order::try_from(ascending.unwrap_or(1i32))?;
        let res: StdResult<Vec<Token>> = self.token_map
            .idx
            .ask_price_token_id
            .range(deps.storage, start, None, order)
            .filter(|item| item.as_ref().and_then(|(_, token)| Ok(token.ask.is_some())).unwrap_or(false))
            .map(|item| item.and_then(|(_, token)| Ok(token)))
            .skip(skip_usize)
            .take(limit_usize)
            .collect();

        let tokens = res.unwrap();

        Ok(AllNftPriceMapResponse {
            tokens: tokens,
            limit: limit_usize.to_string(),
            skip: skip_usize.to_string(),
            count: Some(self.token_map.range(deps.storage, None, None, order).filter(|item| item.as_ref().and_then(|(_, token)| Ok(token.ask.is_some())).unwrap_or(false)).count().to_string()),
            start_after: start_after,
            is_ask: true,
            is_bids: false,
        })
    }

    // pub fn all_nft_sort_price2(
    //     &self,
    //     deps: Deps,
    //     env: Env,
    //     ascending: Option<i32>,
    //     include_expired: bool,
    //     start_after: Option<u32>,
    //     skip: Option<u32>,
    //     limit: Option<u32>,
    // ) -> StdResult<AllNftPriceMapResponse> {
    //     let limit_usize = self.calc_limit(limit);
    //     let skip_usize = self.calc_skip(skip);
    //     let start = Some(Bound::Exclusive(start_after.unwrap_or(0).to_be_bytes().to_vec()));

    //     // let order = Order::try_from(1i32);
    //     let order = Order::try_from(ascending.unwrap_or(1i32))?;
    //     let res: StdResult<Vec<(Vec<u8>, Token)>> = self.token_map
    //         .idx
    //         .ask_price
    //         .range(deps.storage, None, None, order)
    //         .take(limit_usize)
    //         .skip(skip_usize)
    //         .collect();

    //     let tokens = res.unwrap();

    //     Ok(AllNftPriceMapResponse {
    //         tokens: tokens,
    //         limit: limit_usize.to_string(),
    //         skip: skip_usize.to_string(),
    //         count: Some(self.ask_map.range(deps.storage, None, None, Order::Ascending).count().to_string()),
    //         start_after: start_after,
    //         is_ask: true,
    //         is_bids: false,
    //     })
    // }

    // pub fn all_nft_sort_price3(
    //     &self,
    //     deps: Deps,
    //     env: Env,
    //     ascending: Option<i32>,
    //     include_expired: bool,
    //     start_after: Option<u32>,
    //     skip: Option<u32>,
    //     limit: Option<u32>,
    // ) -> StdResult<AllNftPriceMapResponse> {
    //     let limit_usize = self.calc_limit(limit);
    //     let skip_usize = self.calc_skip(skip);
    //     let start = Some(Bound::Exclusive(start_after.unwrap_or(0).to_be_bytes().to_vec()));

    //     // let order = Order::try_from(1i32);
    //     let order = Order::try_from(ascending.unwrap_or(1i32))?;
    //     let res: StdResult<Vec<_>> = self.token_map
    //         .idx
    //         .price
    //         .range(deps.storage, None, None, order)
    //         .take(limit_usize)
    //         .skip(skip_usize)
    //         .collect();

    //     let tokens = res.unwrap();

    //     Ok(AllNftPriceMapResponse {
    //         tokens: tokens,
    //         limit: limit_usize.to_string(),
    //         skip: skip_usize.to_string(),
    //         count: Some(self.ask_map.range(deps.storage, None, None, Order::Ascending).count().to_string()),
    //         start_after: start_after,
    //         is_ask: true,
    //         is_bids: false,
    //     })
    // }

    pub fn query(&self, deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::RoyaltyInfo {} =>
                to_binary(&self.royalty_info(deps)?),
            QueryMsg::NftMarketInfo { token_id, include_expired } =>
                to_binary(&self.nft_market_info(deps, env, token_id, include_expired.unwrap_or(false))?),
            QueryMsg::AllNftBidsInfo { bidder, include_expired, start_after, skip, limit } =>
                to_binary(&self.all_nft_bids_info(deps, env, bidder, include_expired.unwrap_or(false), start_after, skip, limit)?),
            QueryMsg::AllNftAsksInfo { include_expired, start_after, skip, limit } =>
                to_binary(&self.all_nft_asks_info(deps, env, include_expired.unwrap_or(false), start_after, skip, limit)?),
            QueryMsg::AllNftAsksSortInfo { ascending, include_expired, start_after, skip, limit } =>
                to_binary(&self.all_nft_sort_price(deps, env, ascending, include_expired.unwrap_or(false), start_after, skip, limit)?),
            // QueryMsg::AllNftAsksSortInfo2 { ascending, include_expired, start_after, skip, limit } =>
            //     to_binary(&self.all_nft_sort_price2(deps, env, ascending, include_expired.unwrap_or(false), start_after, skip, limit)?),
            // QueryMsg::AllNftAsksSortInfo3 { ascending, include_expired, start_after, skip, limit } =>
            //     to_binary(&self.all_nft_sort_price3(deps, env, ascending, include_expired.unwrap_or(false), start_after, skip, limit)?),
        }
    }
}

