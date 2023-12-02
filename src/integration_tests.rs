#[allow(clippy::inconsistent_digit_grouping, unused_variables)]
#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::InstantiateMsg;
    use astroport::asset::{Asset, AssetInfo, PairInfo};
    use astroport::factory::{InstantiateMsg as FactoryInitMsg, PairConfig, PairType};
    use astroport::pair::{
        ExecuteMsg as PairExecuteMsg, InstantiateMsg as PairInitMsg, QueryMsg as PairQueryMsg,
        XYKPoolUpdateParams,
    };
    use astroport::token::{Cw20Coin, InstantiateMsg as TokenInitMsg, MinterResponse};
    use cosmwasm_std::{to_json_binary, Addr, Coin, Decimal, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    pub fn pair_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            astroport_pair::contract::execute,
            astroport_pair::contract::instantiate,
            astroport_pair::contract::query,
        )
        .with_reply_empty(astroport_pair::contract::reply);
        Box::new(contract)
    }

    pub fn token_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(
            astroport_token::contract::execute,
            astroport_token::contract::instantiate,
            astroport_token::contract::query,
        );
        Box::new(contract)
    }

    pub fn factory_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            astroport_factory::contract::execute,
            astroport_factory::contract::instantiate,
            astroport_factory::contract::query,
        )
        .with_reply_empty(astroport_factory::contract::reply);
        Box::new(contract)
    }

    pub fn whitelist_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            astroport_whitelist::contract::execute,
            astroport_whitelist::contract::instantiate,
            astroport_whitelist::contract::query,
        );
        Box::new(contract)
    }

    pub fn registry_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            astroport_native_coin_registry::contract::execute,
            astroport_native_coin_registry::contract::instantiate,
            astroport_native_coin_registry::contract::query,
        );
        Box::new(contract)
    }

    pub fn my_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply_empty(crate::contract::reply);
        Box::new(contract)
    }
    const USER: &str = "user";
    const ADMIN: &str = "admin";
    const NATIVE_DENOM: &str = "inj";
    const CW20_TEST_TOKEN: &str = "ttt";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(ADMIN),
                    vec![
                        Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::new(10_000_000_000000 + 1_000000),
                        },
                        Coin {
                            denom: "ttt".to_string(),
                            amount: Uint128::new(10_000_000_000_000),
                        },
                        Coin {
                            denom: "abc".to_string(),
                            amount: Uint128::new(10_000_000_000_000),
                        },
                    ],
                )
                .unwrap();
        })
    }
    fn instantiate_token(app: &mut App) -> CwTemplateContract {
        let token_id = app.store_code(token_contract());
        let msg = TokenInitMsg {
            name: "ttt".to_string(),
            symbol: CW20_TEST_TOKEN.to_string(),
            decimals: 18,
            initial_balances: vec![Cw20Coin {
                address: Addr::unchecked(ADMIN).to_string(),
                amount: Uint128::from(1_000_000_000_000u128),
            }],
            mint: Some(MinterResponse {
                minter: String::from(ADMIN),
                cap: None,
            }),
            marketing: None,
        };
        let token_contract_addr = app
            .instantiate_contract(
                token_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "Test Token Contract",
                None,
            )
            .unwrap();

        CwTemplateContract(token_contract_addr, token_id)
    }
    fn instantiate_registry(app: &mut App) -> CwTemplateContract {
        let registry_id = app.store_code(registry_contract());
        let msg = astroport::native_coin_registry::InstantiateMsg {
            owner: ADMIN.to_string(),
        };
        let registry_contract_addr = app
            .instantiate_contract(
                registry_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "Registry",
                Some(ADMIN.to_string()),
            )
            .unwrap();

        CwTemplateContract(registry_contract_addr, registry_id)
    }
    fn instantiate_factory(
        app: &mut App,
        token_contract: &CwTemplateContract,
        registry_contract: &CwTemplateContract,
    ) -> CwTemplateContract {
        let factory_id = app.store_code(factory_contract());
        let pair_id = app.store_code(pair_contract());
        let whitelist_id = app.store_code(whitelist_contract());
        let msg = FactoryInitMsg {
            pair_configs: vec![PairConfig {
                code_id: pair_id,
                pair_type: PairType::Xyk {},
                total_fee_bps: 30,
                maker_fee_bps: 3333,
                is_disabled: false,
                is_generator_disabled: false,
            }],
            token_code_id: token_contract.code_id(),
            fee_address: None,
            generator_address: None,
            owner: Addr::unchecked(ADMIN).to_string(),
            whitelist_code_id: whitelist_id,
            coin_registry_address: registry_contract.addr().to_string(),
        };
        let factory_contract_addr = app
            .instantiate_contract(
                factory_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "Factory contract",
                None,
            )
            .unwrap();
        CwTemplateContract(factory_contract_addr, factory_id)
    }
    fn instantiate_pair(
        app: &mut App,
        token_contract: &CwTemplateContract,
        factory_contract: &CwTemplateContract,
    ) -> CwTemplateContract {
        let pair_id = app.store_code(pair_contract());
        let msg = PairInitMsg {
            asset_infos: [
                AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                AssetInfo::NativeToken {
                    denom: "ttt".to_string(),
                },
            ]
            .to_vec(),
            token_code_id: token_contract.code_id(),
            factory_addr: factory_contract.addr().to_string(),
            init_params: None,
        };
        let pair_contract_addr = app
            .instantiate_contract(
                pair_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "Pair contract",
                None,
            )
            .unwrap();
        CwTemplateContract(pair_contract_addr, pair_id)
    }
    fn instantiate_my_contract(app: &mut App) -> CwTemplateContract {
        let my_contract_id = app.store_code(my_contract());
        let msg = InstantiateMsg { fee_rate: 5 };
        let my_contract_addr = app
            .instantiate_contract(
                my_contract_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "My Contract",
                Some(ADMIN.to_string()),
            )
            .unwrap();
        CwTemplateContract(my_contract_addr, my_contract_id)
    }
    fn instantiate_contracts(
        app: &mut App,
    ) -> (CwTemplateContract, CwTemplateContract, CwTemplateContract) {
        let token_contract = instantiate_token(app);
        let registry_contract = instantiate_registry(app);
        let factory_contract = instantiate_factory(app, &token_contract, &registry_contract);
        let pair_contract = instantiate_pair(app, &token_contract, &factory_contract);
        (token_contract, factory_contract, pair_contract)
    }
    fn provide_liquidity_msg(
        ttt_amount: Uint128,
        inj_amount: Uint128,
        receiver: Option<String>,
        slippage_tolerance: Option<Decimal>,
        token_contract: &CwTemplateContract,
    ) -> (PairExecuteMsg, [Coin; 2]) {
        let msg = PairExecuteMsg::ProvideLiquidity {
            assets: vec![
                Asset {
                    info: AssetInfo::NativeToken {
                        denom: "inj".to_string(),
                    },
                    amount: inj_amount,
                },
                Asset {
                    info: AssetInfo::NativeToken {
                        denom: "ttt".to_string(),
                    },
                    amount: ttt_amount,
                },
            ],
            slippage_tolerance,
            auto_stake: None,
            receiver,
        };

        let coins = [
            Coin {
                denom: "inj".to_string(),
                amount: inj_amount,
            },
            Coin {
                denom: "ttt".to_string(),
                amount: ttt_amount,
            },
        ];

        (msg, coins)
    }
    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());
        let pair_id = app.store_code(pair_contract());
        let msg = InstantiateMsg { fee_rate: 5 };
        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr, cw_template_id);
        (app, cw_template_contract)
    }
    #[allow(unused_variables, clippy::inconsistent_digit_grouping)]
    #[test]
    fn instantiate_and_query_happy_path() {
        let owner = Addr::unchecked(ADMIN);
        let user = Addr::unchecked(USER);
        let (mut app, cw_contract) = proper_instantiate();
        let (token_contract, factory_contract, pair_contract) = instantiate_contracts(&mut app);
        let my_contract = instantiate_my_contract(&mut app);
        let inj_amount = Uint128::new(1_000_000_000000);
        let ttt_amount = Uint128::new(1_000_000_000000);
        let inj_offer = Uint128::new(1_000_000);

        let (msg, coins) =
            provide_liquidity_msg(ttt_amount, inj_amount, None, None, &token_contract);

        app.execute_contract(owner.clone(), pair_contract.addr(), &msg, &coins)
            .unwrap();
        let msg = PairExecuteMsg::UpdateConfig {
            params: to_json_binary(&XYKPoolUpdateParams::EnableAssetBalancesTracking).unwrap(),
        };
        app.execute_contract(owner.clone(), pair_contract.addr(), &msg, &[])
            .unwrap();

        app.update_block(|b| b.height += 1);

        assert_eq!(
            app.wrap().query_balance(owner.clone(), "inj").unwrap(),
            Coin {
                amount: Uint128::from(9000001000000u128),
                denom: "inj".to_string()
            }
        );
        assert_eq!(
            app.wrap()
                .query_wasm_smart::<Option<PairInfo>>(pair_contract.addr(), &PairQueryMsg::Pair {})
                .unwrap(),
            Some(PairInfo {
                asset_infos: vec![
                    AssetInfo::NativeToken {
                        denom: "inj".to_string()
                    },
                    AssetInfo::NativeToken {
                        denom: "ttt".to_string()
                    },
                ],
                contract_addr: Addr::unchecked("contract4"),
                liquidity_token: Addr::unchecked("contract5"),
                pair_type: PairType::Xyk {}
            })
        );
        let res: Option<Uint128> = app
            .wrap()
            .query_wasm_smart(
                pair_contract.addr(),
                &PairQueryMsg::AssetBalanceAt {
                    asset_info: AssetInfo::NativeToken {
                        denom: "inj".to_string(),
                    },
                    block_height: app.block_info().height.into(),
                },
            )
            .unwrap();
        assert_eq!(res.unwrap(), Uint128::new(1_000_000_000000));

        let res: Option<Uint128> = app
            .wrap()
            .query_wasm_smart(
                pair_contract.addr(),
                &PairQueryMsg::AssetBalanceAt {
                    asset_info: AssetInfo::NativeToken {
                        denom: "ttt".to_string(),
                    },
                    block_height: app.block_info().height.into(),
                },
            )
            .unwrap();
        assert_eq!(res.unwrap(), Uint128::new(1_000_000_000000));
        let swap_msg = crate::msg::ExecuteMsg::MySwap {
            pool_address: pair_contract.addr(),
        };
        let send_funds = vec![Coin {
            denom: "inj".to_string(),
            amount: Uint128::new(1_000000),
        }];
        app.execute_contract(owner.clone(), my_contract.addr(), &swap_msg, &send_funds)
            .unwrap();
        app.update_block(|b| b.height += 1);
        // Check pool balances
        let res: Option<Uint128> = app
            .wrap()
            .query_wasm_smart(
                pair_contract.addr(),
                &PairQueryMsg::AssetBalanceAt {
                    asset_info: AssetInfo::NativeToken {
                        denom: "ttt".to_string(),
                    },
                    block_height: app.block_info().height.into(),
                },
            )
            .unwrap();
        assert_eq!(res.unwrap(), Uint128::new(999999003499));
        // Check pool balances
        let res: Option<Uint128> = app
            .wrap()
            .query_wasm_smart(
                pair_contract.addr(),
                &PairQueryMsg::AssetBalanceAt {
                    asset_info: AssetInfo::NativeToken {
                        denom: "inj".to_owned(),
                    },
                    block_height: app.block_info().height.into(),
                },
            )
            .unwrap();
        assert_eq!(res.unwrap(), Uint128::new(1000000999500));
        // Check current balance of owner
        assert_eq!(
            app.wrap()
                .query_balance(owner.clone(), "inj")
                .unwrap()
                .amount,
            Uint128::from(9_000_000_000_000u128)
        );
        // Check if owner receive token from pool
        assert_eq!(
            app.wrap()
                .query_balance(owner.clone(), "ttt")
                .unwrap()
                .amount,
            Uint128::from(9000000996501u128)
        );
        // Check if contract collected fee
        assert_eq!(
            app.wrap()
                .query_balance(my_contract.addr(), "inj")
                .unwrap()
                .amount,
            Uint128::from(500u128)
        );
    }
    #[test]
    fn test_pool_insufficient_balance_for_swap() {
        let owner = Addr::unchecked(ADMIN);
        let user = Addr::unchecked(USER);
        let (mut app, cw_contract) = proper_instantiate();
        let (token_contract, factory_contract, pair_contract) = instantiate_contracts(&mut app);
        let my_contract = instantiate_my_contract(&mut app);
        let inj_amount = Uint128::new(1_000_000);
        let ttt_amount = Uint128::new(1_000_000);
        let inj_offer = Uint128::new(10_000_000);

        let (msg, coins) =
            provide_liquidity_msg(ttt_amount, inj_amount, None, None, &token_contract);

        app.execute_contract(owner.clone(), pair_contract.addr(), &msg, &coins)
            .unwrap();
        // Enable pool balance tracking
        let msg = PairExecuteMsg::UpdateConfig {
            params: to_json_binary(&XYKPoolUpdateParams::EnableAssetBalancesTracking).unwrap(),
        };
        app.execute_contract(owner.clone(), pair_contract.addr(), &msg, &[])
            .unwrap();

        app.update_block(|b| b.height += 1);

        // Check pool balances
        let res: Option<Uint128> = app
            .wrap()
            .query_wasm_smart(
                pair_contract.addr(),
                &PairQueryMsg::AssetBalanceAt {
                    asset_info: AssetInfo::NativeToken {
                        denom: "ttt".to_string(),
                    },
                    block_height: app.block_info().height.into(),
                },
            )
            .unwrap();
        assert_eq!(res.unwrap(), Uint128::new(1_000_000));
        // Check pool balances
        let res: Option<Uint128> = app
            .wrap()
            .query_wasm_smart(
                pair_contract.addr(),
                &PairQueryMsg::AssetBalanceAt {
                    asset_info: AssetInfo::NativeToken {
                        denom: "inj".to_owned(),
                    },
                    block_height: app.block_info().height.into(),
                },
            )
            .unwrap();
        assert_eq!(res.unwrap(), Uint128::new(1_000_000));
        // Perform swap of two native tokens
        let swap_msg = crate::msg::ExecuteMsg::MySwap {
            pool_address: pair_contract.addr(),
        };
        let send_funds = vec![Coin {
            denom: "inj".to_string(),
            amount: inj_offer,
        }];
        let res = app
            .execute_contract(owner.clone(), my_contract.addr(), &swap_msg, &send_funds)
            .unwrap_err();
        assert_eq!(
            crate::error::ContractError::InsufficientFunds {
                asked_amount: inj_offer,
                available_amount: Uint128::new(1_000_000)
            },
            res.downcast().unwrap(),
        );
    }
}
