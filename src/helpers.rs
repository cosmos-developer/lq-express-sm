use astroport::{pair::PoolResponse, querier};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::ExecuteMsg;
use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, QuerierWrapper, StdResult, WasmMsg};

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
type CodeId = u64;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwTemplateContract(pub Addr, pub CodeId);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }
    pub fn code_id(&self) -> u64 {
        self.1
    }
    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}
