//! Uniswap-style Swap Example
//!
//! Demonstrates the concept of proving DEX swaps off-chain.
//! Uses placeholder contracts to show the structure.

use shadow_evm_core::prelude::*;

fn main() {
    println!("Shadow-EVM: DEX Swap\n");

    let trader = Address::repeat_byte(0x01);
    let router = Address::repeat_byte(0xAA);
    let weth = Address::repeat_byte(0xBB);
    let usdc = Address::repeat_byte(0xCC);

    // Setup state
    let mut state = InMemoryDB::new();

    // Trader account
    state.insert_account(
        trader,
        AccountState::new_with_balance(U256::from(10u128.pow(18))),
    );

    // WETH contract
    let mut weth_acc = AccountState::new_with_balance(U256::ZERO);
    weth_acc.code = vec![0x60, 0x01, 0xf3];
    weth_acc.set_storage(U256::from(1), U256::from(10u128.pow(18)));
    state.insert_account(weth, weth_acc);

    // USDC contract
    let mut usdc_acc = AccountState::new_with_balance(U256::ZERO);
    usdc_acc.code = vec![0x60, 0x01, 0xf3];
    state.insert_account(usdc, usdc_acc);

    // Router contract
    let mut router_acc = AccountState::new_with_balance(U256::ZERO);
    router_acc.code = vec![0x60, 0x01, 0xf3];
    state.insert_account(router, router_acc);

    // Swap calldata
    let mut calldata = vec![0x38, 0xed, 0x17, 0x39]; // swapExactTokensForTokens
    calldata.extend_from_slice(&U256::from(5 * 10u128.pow(17)).to_be_bytes::<32>()); // 0.5 WETH
    calldata.extend_from_slice(&U256::from(1000 * 10u128.pow(6)).to_be_bytes::<32>()); // min 1000 USDC

    let input = ExecutionInput::new(
        BlockEnv::default(),
        TxInput::call(trader, router, calldata).with_gas_limit(300_000),
        state,
    );

    println!("Trader: {:?}", trader);
    println!("Router: {:?}", router);
    println!("Swap:   0.5 WETH -> USDC (min 1000)");

    let (output, commitment) = ShadowExecutor::execute(input).expect("execution failed");

    println!("\n--- Result ---");
    println!("Gas used:        {}", output.gas_used);
    println!(
        "Pre-state root:  0x{}",
        hex::encode(commitment.pre_state_root.as_slice())
    );
    println!(
        "Post-state root: 0x{}",
        hex::encode(commitment.post_state_root.as_slice())
    );
    println!(
        "Commitment:      0x{}",
        hex::encode(commitment.commitment.as_slice())
    );

    println!("\n--- Use Case ---");
    println!("1. Run complex routing off-chain");
    println!("2. Generate ZK proof of optimal path");
    println!("3. Submit proof on-chain (~230k gas)");
    println!("4. Avoid paying gas for failed routes");
}
