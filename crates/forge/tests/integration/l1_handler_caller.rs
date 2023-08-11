use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::Contract;
use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;
use std::path::Path;

#[test]
fn l1_handler_caller() {
    let test = test_case!(
        indoc!(
            r#"

            #[derive(Copy, Serde, Drop)]
            struct L1Data {
                balance: felt252,
                token_id: u256
            }

            #[starknet::interface]
            trait IBalanceToken<TContractState> {
                fn get_balance(self: @TContractState) -> felt252;
                fn get_token_id(self: @TContractState) -> u256;
            }

            use serde::Serde;
            use array::{ArrayTrait, SpanTrait};
            use core::result::ResultTrait;
            use snforge_std::{declare, deploy, PreparedContract, l1_handler_call, PreparedL1Handler};

            #[test]
            fn test_l1_handler_call() {
                let class_hash = declare('l1_handler_caller');

                let prepared = PreparedContract {
                    class_hash: class_hash,
                    constructor_calldata: @array![0x123]
                };

                let contract_address = deploy(prepared).unwrap();

                let l1_data = L1Data {
                    balance: 42,
                    token_id: 8888_u256,
                };

                let mut payload: Array<felt252> = ArrayTrait::new();
                l1_data.serialize(ref payload);

                let l1_handler_prepared = PreparedL1Handler {
                    contract_address,
                    selector: 0x01e6b389ca484cb6fb23cbbcaa2db5581a8d970e3c135e8170c2ea5fdc2d3d8e,
                    from_address: 0x123,
                    payload: payload.span(),
                };

                l1_handler_call(l1_handler_prepared);

                let dispatcher = IBalanceTokenDispatcher { contract_address };
                assert(dispatcher.get_balance() == 42, 'Invalid balance');
                assert(dispatcher.get_token_id() == 8888_u256, 'Invalid token id');
            }
        "#
        ),
        Contract::from_code_path(
            "l1_handler_caller".to_string(),
            Path::new("tests/data/contracts/l1_handler_call_checker.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}
