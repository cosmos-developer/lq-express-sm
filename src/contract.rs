#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};
use astroport::pair;
use cosmwasm_std::StdError;
use cosmwasm_std::WasmMsg;
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

    use super::*;

    pub fn astro_exec(
        _deps: DepsMut,
        info: MessageInfo,
        pair_address: String,
    ) -> Result<Response, ContractError> {
        let inj_amount = cw_utils::must_pay(&info, "inj").unwrap().u128();

        // Pair of hINJ-INJ on testnet
        let swap_astro_msg = pair::ExecuteMsg::Swap {
            offer_asset: Asset::native("inj", inj_amount),
            ask_asset_info: None,
            belief_price: None,
            max_spread: Some(Decimal::percent(50)),
            to: Some(info.sender.to_string()),
        };

        let exec_cw20_mint_msg = WasmMsg::Execute {
            contract_addr: pair_address.clone(),
            msg: to_json_binary(&swap_astro_msg)?,
            funds: coins(inj_amount, "inj"),
        };
        let submessage = SubMsg::reply_on_success(exec_cw20_mint_msg, SWAP_REPLY_ID);
        let res = Response::new()
            .add_submessage(submessage)
            .add_attribute("action", "swap")
            .add_attribute("pair", pair_address)
            .add_attribute("offer_asset", "hinj")
            .add_attribute("ask_asset_info", "inj");
        Ok(res)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

pub mod query {}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::Addr;
    use cw_multi_test::{App, ContractWrapper, Executor};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn astro_test() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("user"), coins(1000, "uinj"))
                .unwrap()
        });
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));
        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("user"),
                &InstantiateMsg {},
                &[],
                "Contract",
                None,
            )
            .unwrap();
        let _ = app
            .execute_contract(
                Addr::unchecked("user"),
                addr,
                &ExecuteMsg::Astro {
                    pair_address: "pair".to_string(),
                },
                &coins(10, "uinj"),
            )
            .unwrap();
        // let wasm = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();
        // assert_eq!(
        //     wasm.attributes
        //         .iter()
        //         .find(|attr| attr.key == "action")
        //         .unwrap()
        //         .value,
        //     "swap"
        // );
    }
}
