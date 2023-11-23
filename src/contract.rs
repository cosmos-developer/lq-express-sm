#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};
use astroport::pair::{self};
use cosmwasm_std::{Addr, Order, StdError, WasmMsg};

use self::query::*;
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lq-express-sm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const SWAP_REPLY_ID: u64 = 2u64;
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Astro { pair_address } => execute::astro_exec(deps, info, pair_address),
        ExecuteMsg::AddSupportedPool {
            pool_address,
            token_1,
            token_2,
        } => execute::add_supported_pool(deps, info, pool_address, token_1, token_2),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        SWAP_REPLY_ID => handle_swap_reply(deps, _env, msg),
        id => Err(StdError::generic_err(format!("Unknown reply id: {}", id))),
    }
}
fn handle_swap_reply(_deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let data = msg.result.into_result().map_err(StdError::generic_err)?;

    // Search for the transfer event
    // If there are multiple transfers, you will need to find the right event to handle
    let swap_event = data
        .events
        .iter()
        .find(|e| {
            e.attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "swap")
                && e.attributes.iter().any(|attr| attr.key == "return_amount")
        })
        .ok_or_else(|| StdError::generic_err("unable to find swap action".to_string()))?;

    let _ = swap_event
        .attributes
        .iter()
        .find(|e| e.key == "return_amount")
        .ok_or_else(|| StdError::generic_err("unable to find coin spent event".to_string()))?;
    // let spender_address = coin_spent_event
    //     .attributes
    //     .iter()
    //     .find(|a| a.key == "spender")
    //     .unwrap()
    //     .value
    //     .clone();
    // let coin = Coin::from_str(&spend_amount).unwrap();
    // // transfer back to user
    // let msg = BankMsg::Send {
    //     to_address: spender_address,
    //     amount: vec![coin],
    // };
    Ok(Response::new())
}
pub mod execute {
    use astroport::asset::Asset;
    use cosmwasm_std::{Decimal, SubMsg};

    use crate::state::{PoolInfo, POOL_INFO};

    use super::*;

    pub fn astro_exec(
        deps: DepsMut,
        info: MessageInfo,
        pair_address: String,
    ) -> Result<Response, ContractError> {
        let pool_infos: Vec<_> = POOL_INFO
            .idx
            .address
            .prefix(pair_address.clone())
            .range(deps.storage, None, None, Order::Ascending)
            .take(1)
            .flatten()
            .collect();

        if pool_infos.is_empty() {
            return Err(ContractError::PoolNotExist {});
        }
        let pair_info = pool_infos
            .first()
            .map(|pool_info| (pool_info.1.token_1.clone(), pool_info.1.token_2.clone()))
            .unwrap();
        // Based on cw_utils::must_pay implementation
        let coin = cw_utils::one_coin(&info)?;
        let offer_asset = coin.denom.clone();
        let amount: u128 = if offer_asset != pair_info.0 && offer_asset != pair_info.1 {
            return Err(ContractError::Payment(
                cw_utils::PaymentError::MissingDenom(coin.denom.to_string()),
            ));
        } else {
            coin.amount.into()
        };
        let asked_asset = pair_info.1.clone();
        // Pair of hINJ-INJ on testnet

        let swap_astro_msg = pair::ExecuteMsg::Swap {
            offer_asset: Asset::native(&offer_asset, amount),
            ask_asset_info: None,
            belief_price: None,
            max_spread: Some(Decimal::percent(50)),
            to: Some(info.sender.to_string()),
        };

        let exec_cw20_mint_msg = WasmMsg::Execute {
            contract_addr: pair_address.clone(),
            msg: to_json_binary(&swap_astro_msg)?,
            funds: coins(amount, &offer_asset),
        };
        assert!(offer_asset == "inj");
        let submessage = SubMsg::reply_on_success(exec_cw20_mint_msg, SWAP_REPLY_ID);
        let res = Response::new()
            .add_submessage(submessage)
            .add_attribute("action", "swap")
            .add_attribute("pair", pair_address)
            .add_attribute("offer_asset", offer_asset)
            .add_attribute("ask_asset_info", asked_asset);
        Ok(res)
    }
    pub fn add_supported_pool(
        deps: DepsMut,
        info: MessageInfo,
        pool_address: String,
        token_1: String,
        token_2: String,
    ) -> Result<Response, ContractError> {
        // Check authorization
        if info.sender != STATE.load(deps.storage)?.owner {
            return Err(ContractError::Unauthorized {});
        }
        let pool_info = PoolInfo {
            address: Addr::unchecked(pool_address),
            token_1: token_1.clone(),
            token_2: token_2.clone(),
        };
        POOL_INFO.save(
            deps.storage,
            [token_1.clone(), token_2.clone()].join("_").as_str(),
            &pool_info,
        )?;

        Ok(Response::new()
            .add_attribute("method", "add_supported_pool")
            .add_attribute("pool_address", pool_info.address)
            .add_attribute("token 1", token_1)
            .add_attribute("token2", token_2))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPair { pool_address } => to_json_binary(&query_pair(deps, env, pool_address)?),
        QueryMsg::GetPoolAddr { token_1, token_2 } => {
            to_json_binary(&query_pool_addr(deps, env, token_1, token_2)?)
        }
    }
}

pub mod query {
    use crate::msg::{GetPairResponse, GetPoolAddrResponse};
    use crate::state::POOL_INFO;
    use crate::{contract::*, error};
    use cosmwasm_std::StdResult;

    pub fn query_pair(deps: Deps, env: Env, pool_address: String) -> StdResult<GetPairResponse> {
        let pair: Vec<_> = POOL_INFO
            .idx
            .address
            .prefix(pool_address)
            .range(deps.storage, None, None, Order::Ascending)
            .flatten()
            .collect();
        if pair.is_empty() {
            return Err(StdError::GenericErr {
                msg: "pool address not found".to_string(),
            });
        }
        let (token_1, token_2): (String, String) = pair
            .first()
            .iter()
            .map(|pair| (pair.1.token_1.clone(), pair.1.token_2.clone()))
            .unzip();
        let resp = GetPairResponse { token_1, token_2 };
        Ok(resp)
    }
    pub fn query_pool_addr(
        deps: Deps,
        env: Env,
        token_1: String,
        token_2: String,
    ) -> StdResult<GetPoolAddrResponse> {
        let token_1 = token_1.to_lowercase();
        let token_2 = token_2.to_lowercase();

        let pools = POOL_INFO
            .idx
            .pair
            .prefix((token_1, token_2))
            .range(deps.storage, None, None, Order::Ascending)
            .flatten()
            .collect::<Vec<_>>();

        if pools.is_empty() {
            return Err(StdError::GenericErr {
                msg: "No pool exist for this pair yet".to_string(),
            });
        }
        let pool_addresses = pools
            .iter()
            .map(|pool_info| pool_info.1.address.to_string().clone())
            .collect::<Vec<_>>();
        return Ok(GetPoolAddrResponse { pool_addresses });
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::GetPairResponse;

    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::Addr;
    use cw_multi_test::{App, ContractWrapper, Executor};

    #[test]
    fn test_add_pool() {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                owner.clone(),
                &InstantiateMsg {},
                &[],
                "Contract",
                None,
            )
            .unwrap();
        let msg = ExecuteMsg::AddSupportedPool {
            pool_address: "pool1".to_string(),
            token_1: "inj".to_string(),
            token_2: "atom".to_string(),
        };
        app.execute_contract(owner.clone(), addr.clone(), &msg, &[])
            .unwrap();

        app.update_block(|b| b.height += 1);

        let resp: GetPairResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetPair {
                    pool_address: "pool1".to_string(),
                },
            )
            .unwrap();
        assert_eq!(
            resp,
            GetPairResponse {
                token_1: "inj".to_string(),
                token_2: "atom".to_string()
            }
        )
    }
}
