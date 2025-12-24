//! ERC20 Token Transfer Example
//!
//! Demonstrates calling an ERC20 contract's transfer function.
//! Note: Uses minimal bytecode for demonstration purposes.

use shadow_evm_core::prelude::*;

fn main() {
    println!("Shadow-EVM: ERC20 Transfer\n");

    let sender = Address::repeat_byte(0x01);
    let token = Address::repeat_byte(0xEE);
    let recipient = Address::repeat_byte(0x02);

    // Setup state
    let mut state = InMemoryDB::new();

    // Sender with gas funds
    state.insert_account(
        sender,
        AccountState::new_with_balance(U256::from(10u128.pow(18))),
    );

    // Token contract with storage
    let mut token_acc = AccountState::new_with_balance(U256::ZERO);
    token_acc.code = vec![0x60, 0x01, 0xf3]; // minimal bytecode
    token_acc.set_storage(U256::from(1), U256::from(1_000_000)); // balance slot
    state.insert_account(token, token_acc);

    // Build transfer calldata: transfer(address,uint256)
    let mut calldata = vec![0xa9, 0x05, 0x9c, 0xbb]; // function selector
    calldata.extend_from_slice(&[0u8; 12]);
    calldata.extend_from_slice(recipient.as_slice());
    calldata.extend_from_slice(&U256::from(1000u64).to_be_bytes::<32>());

    let input = ExecutionInput::new(
        BlockEnv::default(),
        TxInput::call(sender, token, calldata).with_gas_limit(100_000),
        state,
    );

    println!("Sender:    {:?}", sender);
    println!("Token:     {:?}", token);
    println!("Recipient: {:?}", recipient);
    println!("Amount:    1000 tokens");

    let (output, commitment) = ShadowExecutor::execute(input).expect("execution failed");

    println!("\n--- Result ---");
    println!(
        "Status:     {}",
        if output.is_success() {
            "Success"
        } else {
            "Reverted"
        }
    );
    println!("Gas used:   {}", output.gas_used);
    println!(
        "Commitment: 0x{}",
        hex::encode(commitment.commitment.as_slice())
    );

    println!("\nNote: Minimal bytecode returns immediately.");
    println!("Deploy real ERC20 bytecode for full functionality.");
}
