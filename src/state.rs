use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub owner: Addr,
}
pub struct InfoIndexes<'a> {
    pub address: MultiIndex<'a, String, PoolInfo, String>,
    pub pair: MultiIndex<'a, (String, String), PoolInfo, String>,
}
impl<'a> IndexList<PoolInfo> for InfoIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<PoolInfo>> + '_> {
        let v: Vec<&dyn Index<PoolInfo>> = vec![&self.address, &self.pair];
        Box::new(v.into_iter())
    }
}
pub const fn infos<'a>() -> IndexedMap<'a, &'a str, PoolInfo, InfoIndexes<'a>> {
    let indexes = InfoIndexes {
        address: MultiIndex::new(
            |_pk: &[u8], d: &PoolInfo| d.address.to_string(),
            "infos",
            "infos__address",
        ),
        pair: MultiIndex::new(
            |_pk: &[u8], d: &PoolInfo| (d.token_1.clone(), d.token_2.clone()),
            "infos",
            "infos__pair",
        ),
    };
    IndexedMap::new("infos", indexes)
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PoolInfo {
    pub address: Addr,
    pub token_1: String,
    pub token_2: String,
}
pub const STATE: Item<State> = Item::new("state");
pub const POOL_INFO: IndexedMap<&str, PoolInfo, InfoIndexes> = infos();
pub const POOL_CONTRACT_ADDR: Item<Addr> = Item::new("pool_contract_addr");
pub const REGISTRY_CONTRACT_ADDR: Item<Addr> = Item::new("registry_contract_addr");
pub const FACTORY_CONTRACT_ADDR: Item<Addr> = Item::new("factory_contract_addr");
pub const PAIR_CONTRACT_ADDR: Item<Addr> = Item::new("pair_contract_addr");
