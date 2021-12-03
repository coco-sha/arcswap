#![cfg(test)]

use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{coins, from_binary, Addr, BalanceResponse, BankQuery, Coin, Empty, Uint128};
use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg};
use cw_multi_test::{App, Contract, ContractWrapper, SimpleBank};

use crate::msg::{ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg};

fn mock_app() -> App {
    let env = mock_env();
    let api = Box::new(MockApi::default());
    let bank = SimpleBank {};

    App::new(api, env.block, bank, || Box::new(MockStorage::new()))
}

pub fn contract_arcswap() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub fn contract_cw20() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

fn get_info(router: &App, contract_addr: &Addr) -> InfoResponse {
    router
        .wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Info {})
        .unwrap()
}

fn create_arcswap(router: &mut App, owner: &Addr, cash: &Cw20Contract, native_denom: String) -> Addr {
    // set up arcswap contract
    let arcswap_id = router.store_code(contract_arcswap());
    let msg = InstantiateMsg {
        native_denom,
        token_denom: cash.meta(router).unwrap().symbol,
        token_address: cash.addr(),
    };
    let arcswap_addr = router
        .instantiate_contract(arcswap_id, owner.clone(), &msg, &[], "arcswap")
        .unwrap();
    arcswap_addr
}

// CreateCW20 create new cw20 with given initial balance belonging to owner
fn create_cw20(
    router: &mut App,
    owner: &Addr,
    name: String,
    symbol: String,
    balance: Uint128,
) -> Cw20Contract {
    // set up cw20 contract with some tokens
    let cw20_id = router.store_code(contract_cw20());
    let msg = cw20_base::msg::InstantiateMsg {
        name: name,
        symbol: symbol,
        decimals: 2,
        initial_balances: vec![Cw20Coin {
            address: owner.to_string(),
            amount: balance,
        }],
        mint: None,
    };
    let addr = router
        .instantiate_contract(cw20_id, owner.clone(), &msg, &[], "CASH")
        .unwrap();
    Cw20Contract(addr)
}

fn bank_balance(router: &mut App, addr: &Addr, denom: String) -> BalanceResponse {
    let query_res = router
        .query(
            cosmwasm_std::QueryRequest::Bank(BankQuery::Balance {
                address: addr.to_string(),
                denom,
            })
            .into(),
        )
        .unwrap();
    from_binary(&query_res).unwrap()
}

#[test]
// receive cw20 tokens and release upon approval
fn arcswap_add_and_remove_liquidity() {
    let mut router = mock_app();

    const NATIVE_TOKEN_DENOM: &str = "juno";

    let owner = Addr::unchecked("owner");
    let funds = coins(2000, NATIVE_TOKEN_DENOM);
    router.set_bank_balance(&owner, funds).unwrap();

    let cw20_token = create_cw20(
        &mut router,
        &owner,
        "token".to_string(),
        "CWTOKEN".to_string(),
        Uint128(5000),
    );

    let arcswap_addr = create_arcswap(&mut router, &owner, &cw20_token, NATIVE_TOKEN_DENOM.into());

    assert_ne!(cw20_token.addr(), arcswap_addr);

    // set up cw20 helpers
    let arcswap = Cw20Contract(arcswap_addr.clone());

    // check initial balances
    let owner_balance = cw20_token.balance(&router, owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128(5000));

    // send tokens to contract address
    let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: arcswap_addr.to_string(),
        amount: Uint128::from(100u128),
        expires: None,
    };
    let res = router
        .execute_contract(owner.clone(), cw20_token.addr(), &allowance_msg, &[])
        .unwrap();
    println!("{:?}", res.attributes);

    let add_liquidity_msg = ExecuteMsg::AddLiquidity {
        min_liquidity: Uint128(100),
        max_token: Uint128(100),
        expiration: None,
    };
    let res = router
        .execute_contract(
            owner.clone(),
            arcswap_addr.clone(),
            &add_liquidity_msg,
            &[Coin {
                denom: NATIVE_TOKEN_DENOM.into(),
                amount: Uint128(100),
            }],
        )
        .unwrap();
    println!("{:?}", res.attributes);

    // ensure balances updated
    let owner_balance = cw20_token.balance(&router, owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128(4900));
    let arcswap_balance = cw20_token.balance(&router, arcswap_addr.clone()).unwrap();
    assert_eq!(arcswap_balance, Uint128(100));
    let crust_balance = arcswap.balance(&router, owner.clone()).unwrap();
    assert_eq!(crust_balance, Uint128(100));

    // send tokens to contract address
    let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: arcswap_addr.to_string(),
        amount: Uint128::from(51u128),
        expires: None,
    };
    let res = router
        .execute_contract(owner.clone(), cw20_token.addr(), &allowance_msg, &[])
        .unwrap();
    println!("{:?}", res.attributes);
    assert_eq!(res.attributes.len(), 4);

    let add_liquidity_msg = ExecuteMsg::AddLiquidity {
        min_liquidity: Uint128(50),
        max_token: Uint128(51),
        expiration: None,
    };
    let res = router
        .execute_contract(
            owner.clone(),
            arcswap_addr.clone(),
            &add_liquidity_msg,
            &[Coin {
                denom: NATIVE_TOKEN_DENOM.into(),
                amount: Uint128(50),
            }],
        )
        .unwrap();
    println!("{:?}", res.attributes);

    // ensure balances updated
    let owner_balance = cw20_token.balance(&router, owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128(4849));
    let arcswap_balance = cw20_token.balance(&router, arcswap_addr.clone()).unwrap();
    assert_eq!(arcswap_balance, Uint128(151));
    let crust_balance = arcswap.balance(&router, owner.clone()).unwrap();
    assert_eq!(crust_balance, Uint128(150));

    let remove_liquidity_msg = ExecuteMsg::RemoveLiquidity {
        amount: Uint128(50),
        min_native: Uint128(50),
        min_token: Uint128(50),
        expiration: None,
    };
    let res = router
        .execute_contract(
            owner.clone(),
            arcswap_addr.clone(),
            &remove_liquidity_msg,
            &[Coin {
                denom: NATIVE_TOKEN_DENOM.into(),
                amount: Uint128(50),
            }],
        )
        .unwrap();
    println!("{:?}", res.attributes);

    // ensure balances updated
    let owner_balance = cw20_token.balance(&router, owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128(4899));
    let arcswap_balance = cw20_token.balance(&router, arcswap_addr.clone()).unwrap();
    assert_eq!(arcswap_balance, Uint128(101));
    let crust_balance = arcswap.balance(&router, owner.clone()).unwrap();
    assert_eq!(crust_balance, Uint128(100));
}

#[test]
fn swap_tokens_happy_path() {
    let mut router = mock_app();

    const NATIVE_TOKEN_DENOM: &str = "juno";

    let owner = Addr::unchecked("owner");
    let funds = coins(2000, NATIVE_TOKEN_DENOM);
    router.set_bank_balance(&owner, funds).unwrap();

    let cw20_token = create_cw20(
        &mut router,
        &owner,
        "token".to_string(),
        "CWTOKEN".to_string(),
        Uint128(5000),
    );

    let arcswap_addr = create_arcswap(
        &mut router,
        &owner,
        &cw20_token,
        NATIVE_TOKEN_DENOM.to_string(),
    );

    assert_ne!(cw20_token.addr(), arcswap_addr);

    // check initial balances
    let owner_balance = cw20_token.balance(&router, owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128(5000));

    // send tokens to contract address
    let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: arcswap_addr.to_string(),
        amount: Uint128::from(100u128),
        expires: None,
    };
    let res = router
        .execute_contract(owner.clone(), cw20_token.addr(), &allowance_msg, &[])
        .unwrap();
    println!("{:?}", res.attributes);

    let add_liquidity_msg = ExecuteMsg::AddLiquidity {
        min_liquidity: Uint128(100),
        max_token: Uint128(100),
        expiration: None,
    };
    let res = router
        .execute_contract(
            owner.clone(),
            arcswap_addr.clone(),
            &add_liquidity_msg,
            &[Coin {
                denom: NATIVE_TOKEN_DENOM.into(),
                amount: Uint128(100),
            }],
        )
        .unwrap();
    println!("{:?}", res.attributes);

    let info = get_info(&router, &arcswap_addr);
    assert_eq!(info.native_reserve, Uint128(100));
    assert_eq!(info.token_reserve, Uint128(100));

    let buyer = Addr::unchecked("buyer");
    let funds = coins(2000, NATIVE_TOKEN_DENOM);
    router.set_bank_balance(&buyer, funds).unwrap();

    let add_liquidity_msg = ExecuteMsg::SwapToken1ForToken2 {
        min_token: Uint128(9),
        expiration: None,
    };
    let res = router
        .execute_contract(
            buyer.clone(),
            arcswap_addr.clone(),
            &add_liquidity_msg,
            &[Coin {
                denom: NATIVE_TOKEN_DENOM.into(),
                amount: Uint128(10),
            }],
        )
        .unwrap();
    println!("{:?}", res.attributes);

    let info = get_info(&router, &arcswap_addr);
    assert_eq!(info.native_reserve, Uint128(110));
    assert_eq!(info.token_reserve, Uint128(91));

    // ensure balances updated
    let buyer_balance = cw20_token.balance(&router, buyer.clone()).unwrap();
    assert_eq!(buyer_balance, Uint128(9));

    // Check balances of owner and buyer reflect the sale transaction
    let balance: BalanceResponse =
        bank_balance(&mut router, &buyer, NATIVE_TOKEN_DENOM.to_string());
    assert_eq!(balance.amount.amount, Uint128(1990));

    let swap_msg = ExecuteMsg::SwapToken1ForToken2 {
        min_token: Uint128(7),
        expiration: None,
    };
    let res = router
        .execute_contract(
            buyer.clone(),
            arcswap_addr.clone(),
            &swap_msg,
            &[Coin {
                denom: NATIVE_TOKEN_DENOM.into(),
                amount: Uint128(10),
            }],
        )
        .unwrap();
    println!("{:?}", res.attributes);

    let info = get_info(&router, &arcswap_addr);
    assert_eq!(info.native_reserve, Uint128(120));
    assert_eq!(info.token_reserve, Uint128(84));

    // ensure balances updated
    let buyer_balance = cw20_token.balance(&router, buyer.clone()).unwrap();
    assert_eq!(buyer_balance, Uint128(16));

    // Check balances of owner and buyer reflect the sale transaction
    let balance: BalanceResponse =
        bank_balance(&mut router, &buyer, NATIVE_TOKEN_DENOM.to_string());
    assert_eq!(balance.amount.amount, Uint128(1980));

    // Swap token for native

    // send tokens to contract address
    let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: arcswap_addr.to_string(),
        amount: Uint128(16),
        expires: None,
    };
    let res = router
        .execute_contract(buyer.clone(), cw20_token.addr(), &allowance_msg, &[])
        .unwrap();
    println!("{:?}", res.attributes);

    let swap_msg = ExecuteMsg::SwapToken2ForToken1 {
        token_amount: Uint128(16),
        min_native: Uint128(19),
        expiration: None,
    };
    let res = router
        .execute_contract(buyer.clone(), arcswap_addr.clone(), &swap_msg, &vec![])
        .unwrap();
    println!("{:?}", res.attributes);

    let info = get_info(&router, &arcswap_addr);
    assert_eq!(info.native_reserve, Uint128(101));
    assert_eq!(info.token_reserve, Uint128(100));

    // ensure balances updated
    let buyer_balance = cw20_token.balance(&router, buyer.clone()).unwrap();
    assert_eq!(buyer_balance, Uint128(0));

    // Check balances of owner and buyer reflect the sale transaction
    let balance: BalanceResponse =
        bank_balance(&mut router, &buyer, NATIVE_TOKEN_DENOM.to_string());
    assert_eq!(balance.amount.amount, Uint128(1999));

    // check owner balance
    let owner_balance = cw20_token.balance(&router, owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128(4900));

    let swap_msg = ExecuteMsg::SwapNativeForTokenTo {
        recipient: owner.clone(),
        min_token: Uint128(3),
        expiration: None,
    };
    let res = router
        .execute_contract(
            buyer.clone(),
            arcswap_addr.clone(),
            &swap_msg,
            &[Coin {
                denom: NATIVE_TOKEN_DENOM.into(),
                amount: Uint128(10),
            }],
        )
        .unwrap();
    println!("{:?}", res.attributes);

    let info = get_info(&router, &arcswap_addr);
    assert_eq!(info.native_reserve, Uint128(111));
    assert_eq!(info.token_reserve, Uint128(92));

    // ensure balances updated
    let owner_balance = cw20_token.balance(&router, owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128(4908));

    // Check balances of owner and buyer reflect the sale transaction
    let balance = bank_balance(&mut router, &buyer, NATIVE_TOKEN_DENOM.to_string());
    assert_eq!(balance.amount.amount, Uint128(1989));
}

#[test]
fn token_to_token_swap() {
    let mut router = mock_app();

    const NATIVE_TOKEN_DENOM: &str = "juno";

    let owner = Addr::unchecked("owner");
    let funds = coins(2000, NATIVE_TOKEN_DENOM);
    router.set_bank_balance(&owner, funds).unwrap();

    let token1 = create_cw20(
        &mut router,
        &owner,
        "token1".to_string(),
        "TOKENONE".to_string(),
        Uint128(5000),
    );
    let token2 = create_cw20(
        &mut router,
        &owner,
        "token2".to_string(),
        "TOKENTWO".to_string(),
        Uint128(5000),
    );

    let arcswap1 = create_arcswap(&mut router, &owner, &token1, NATIVE_TOKEN_DENOM.to_string());
    let arcswap2 = create_arcswap(&mut router, &owner, &token2, NATIVE_TOKEN_DENOM.to_string());

    // Add initial liquidity to both pools
    let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: arcswap1.to_string(),
        amount: Uint128(100),
        expires: None,
    };
    let res = router
        .execute_contract(owner.clone(), token1.addr(), &allowance_msg, &[])
        .unwrap();
    println!("{:?}", res.attributes);

    let add_liquidity_msg = ExecuteMsg::AddLiquidity {
        min_liquidity: Uint128(100),
        max_token: Uint128(100),
        expiration: None,
    };
    router
        .execute_contract(
            owner.clone(),
            arcswap1.clone(),
            &add_liquidity_msg,
            &[Coin {
                denom: NATIVE_TOKEN_DENOM.into(),
                amount: Uint128(100),
            }],
        )
        .unwrap();

    let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: arcswap2.to_string(),
        amount: Uint128(100),
        expires: None,
    };
    let res = router
        .execute_contract(owner.clone(), token2.addr(), &allowance_msg, &[])
        .unwrap();
    println!("{:?}", res.attributes);

    let add_liquidity_msg = ExecuteMsg::AddLiquidity {
        min_liquidity: Uint128(100),
        max_token: Uint128(100),
        expiration: None,
    };
    router
        .execute_contract(
            owner.clone(),
            arcswap2.clone(),
            &add_liquidity_msg,
            &[Coin {
                denom: NATIVE_TOKEN_DENOM.into(),
                amount: Uint128(100),
            }],
        )
        .unwrap();

    // Swap token1 for token2
    let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: arcswap1.to_string(),
        amount: Uint128(10),
        expires: None,
    };
    let res = router
        .execute_contract(owner.clone(), token1.addr(), &allowance_msg, &[])
        .unwrap();
    println!("{:?}", res.attributes);

    let swap_msg = ExecuteMsg::SwapTokenForToken {
        output_arcswap_address: arcswap2.clone(),
        input_token_amount: Uint128(10),
        output_min_token: Uint128(8),
        expiration: None,
    };
    let res = router
        .execute_contract(owner.clone(), arcswap1.clone(), &swap_msg, &[])
        .unwrap();

    println!("{:?}", res.attributes);

    // ensure balances updated
    let token1_balance = token1.balance(&router, owner.clone()).unwrap();
    assert_eq!(token1_balance, Uint128(4890));

    let token2_balance = token2.balance(&router, owner.clone()).unwrap();
    assert_eq!(token2_balance, Uint128(4908));

    let arcswap1_native_balance = bank_balance(&mut router, &arcswap1, NATIVE_TOKEN_DENOM.to_string());
    assert_eq!(arcswap1_native_balance.amount.amount, Uint128(91));

    let arcswap2_native_balance = bank_balance(&mut router, &arcswap2, NATIVE_TOKEN_DENOM.to_string());
    assert_eq!(arcswap2_native_balance.amount.amount, Uint128(109));

    // Swap token2 for token1
    let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: arcswap2.to_string(),
        amount: Uint128(10),
        expires: None,
    };
    let res = router
        .execute_contract(owner.clone(), token2.addr(), &allowance_msg, &[])
        .unwrap();
    println!("{:?}", res.attributes);

    let swap_msg = ExecuteMsg::SwapTokenForToken {
        output_arcswap_address: arcswap1.clone(),
        input_token_amount: Uint128(10),
        output_min_token: Uint128(1),
        expiration: None,
    };
    let res = router
        .execute_contract(owner.clone(), arcswap2.clone(), &swap_msg, &[])
        .unwrap();

    println!("{:?}", res.attributes);

    // ensure balances updated
    let token1_balance = token1.balance(&router, owner.clone()).unwrap();
    assert_eq!(token1_balance, Uint128(4900));

    let token2_balance = token2.balance(&router, owner.clone()).unwrap();
    assert_eq!(token2_balance, Uint128(4898));

    let arcswap1_native_balance = bank_balance(&mut router, &arcswap1, NATIVE_TOKEN_DENOM.to_string());
    assert_eq!(arcswap1_native_balance.amount.amount, Uint128(101));

    let arcswap2_native_balance = bank_balance(&mut router, &arcswap2, NATIVE_TOKEN_DENOM.to_string());
    assert_eq!(arcswap2_native_balance.amount.amount, Uint128(99));

    // assert internal state is consistent
    let info_arcswap1 = get_info(&router, &arcswap1);
    let token1_balance = token1.balance(&router, arcswap1.clone()).unwrap();
    assert_eq!(info_arcswap1.token_reserve, token1_balance);
    assert_eq!(info_arcswap1.native_reserve, arcswap1_native_balance.amount.amount);

    let info_arcswap2 = get_info(&router, &arcswap2);
    let token2_balance = token2.balance(&router, arcswap2.clone()).unwrap();
    assert_eq!(info_arcswap2.token_reserve, token2_balance);
    assert_eq!(info_arcswap2.native_reserve, arcswap2_native_balance.amount.amount);
}
