#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

use astroport::pair;
use cosmwasm_std::WasmMsg;
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lq-express-sm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: msg.count,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => execute::increment(deps),
        ExecuteMsg::Reset { count } => execute::reset(deps, info, count),
        ExecuteMsg::Astro {} => execute::astro_exec(deps, info),
    }
}

pub mod execute {
    use astroport::asset::Asset;
    use cosmwasm_std::{Coin, Decimal};

    use super::*;

    pub fn increment(deps: DepsMut) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.count += 1;
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("action", "increment"))
    }

    pub fn reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            if info.sender != state.owner {
                return Err(ContractError::Unauthorized {});
            }
            state.count = count;
            Ok(state)
        })?;
        Ok(Response::new().add_attribute("action", "reset"))
    }
    pub fn astro_exec(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let inj_amount = cw_utils::must_pay(&info, "inj")?.u128();
        // Pair of hINJ-INJ on testnet
        let pair = "inj1relf3d6yh2e6arv84gaajawskttdq3vptn8qrs";
        let swap_astro_msg = pair::ExecuteMsg::Swap {
            offer_asset: Asset::native("inj", inj_amount),
            ask_asset_info: None,
            belief_price: None,
            max_spread: Some(Decimal::percent(50)),
            to: None,
        };
        assert!(inj_amount > 0);
        let exec_cw20_mint_msg = WasmMsg::Execute {
            contract_addr: pair.into(),
            msg: to_json_binary(&swap_astro_msg)?,
            funds: coins(inj_amount, "inj"),
        };
        let res = Response::new()
            .add_message(exec_cw20_mint_msg)
            .add_attribute("action", "swap")
            .add_attribute("pair", pair)
            .add_attribute("offer_asset", "hinj")
            .add_attribute("ask_asset_info", "inj");
        Ok(res)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_json_binary(&query::count(deps)?),
    }
}

pub mod query {
    use super::*;

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::Addr;
    use cosmwasm_std::{coins, from_json};
    use cw_multi_test::{App, ContractWrapper, Executor};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(5, value.count);
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
                &InstantiateMsg { count: 17 },
                &[],
                "Contract",
                None,
            )
            .unwrap();
        let resp = app
            .execute_contract(
                Addr::unchecked("user"),
                addr,
                &ExecuteMsg::Astro {},
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
