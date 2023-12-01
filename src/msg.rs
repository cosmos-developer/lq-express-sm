use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Astro { pair_address: String },
    MySwap { pool_address: Addr },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

//We define a custom struct for each query response
#[cw_serde]
pub struct GetPairResponse {
    pub token_1: String,
    pub token_2: String,
}

#[cw_serde]
pub struct GetPoolAddrResponse {
    pub pool_addresses: Vec<String>,
}
