use near_sdk::{VMContext, Gas, test_utils::test_env::{alice, bob}};
use std::{convert::TryInto};

#[allow(dead_code)]
pub(crate) fn get_context() -> VMContext {
    VMContext {
        current_account_id: alice(),
        signer_account_id: bob(),
        signer_account_pk: vec![0u8; 33].try_into().unwrap(),
        predecessor_account_id: bob(),
        input: vec![],
        block_index: 0,
        block_timestamp: 0,
        epoch_height: 19,
        account_balance: 0,
        account_locked_balance: 0,
        storage_usage: 10_000,
        attached_deposit: 0,
        prepaid_gas: Gas(300 * 10u64.pow(12)),
        random_seed: [0u8; 32],
        view_config: None,
        output_data_receivers: vec![],
    }
}
