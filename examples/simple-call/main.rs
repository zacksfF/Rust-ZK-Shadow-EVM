//! Simple ETH Transfer Example
//!
//! Demonstrates basic Ethereum value transfer with ZK commitment generation.

use shadow_evm_core::prelude::*;

fn main() {
    println!("Shadow-EVM: Simple Transfer\n");

    // Define accounts
    let sender = Address::repeat_byte(0x01);
    let receiver = Address::repeat_byte(0x02);
    let amount = U256::from(10u128.pow(17)); // 0.1 ETH

    // Initialize state with sender balance
    let mut state = InMemoryDB::new();
    state.insert_account(
        sender,
        AccountState::new_with_balance(U256::from(10u128.pow(18))), // 1 ETH
    );

    // Build execution input
    let input = ExecutionInput::new(
        BlockEnv::default(),
        TxInput::transfer(sender, receiver, amount),
        state,
    );

    println!("Sender:   {:?}", sender);
    println!("Receiver: {:?}", receiver);
    println!("Amount:   0.1 ETH");
    println!(
        "\nPre-state root:  0x{}",
        hex::encode(input.pre_state_root().as_slice())
    );

    // Execute
    let (output, commitment) = ShadowExecutor::execute(input).expect("execution failed");

    // Results
    println!("\n--- Execution Result ---");
    println!(
        "Status:          {}",
        if output.is_success() {
            "Success"
        } else {
            "Failed"
        }
    );
    println!("Gas used:        {}", output.gas_used);
    println!(
        "Post-state root: 0x{}",
        hex::encode(output.post_state_root().as_slice())
    );

    // Show balances
    let sender_bal = output
        .post_state
        .accounts
        .get(&sender)
        .map(|a| a.balance)
        .unwrap_or(U256::ZERO);
    let receiver_bal = output
        .post_state
        .accounts
        .get(&receiver)
        .map(|a| a.balance)
        .unwrap_or(U256::ZERO);

    println!("\n--- Final Balances ---");
    println!("Sender:   {} wei", sender_bal);
    println!("Receiver: {} wei", receiver_bal);

    // ZK commitment (this would be proven in RISC Zero)
    println!("\n--- ZK Commitment ---");
    println!(
        "Commitment: 0x{}",
        hex::encode(commitment.commitment.as_slice())
    );
}
