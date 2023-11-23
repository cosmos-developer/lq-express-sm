use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Astro {
        pair_address: String,
    },
    AddSupportedPool {
        pool_address: String,
        token_1: String,
        token_2: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetPairResponse)]
    GetPair { pool_address: String },
    #[returns(GetPoolAddrResponse)]
    GetPoolAddr { token_1: String, token_2: String },
}

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
