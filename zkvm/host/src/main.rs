//! Shadow-EVM Host CLI
//!
//! Command-line interface for generating and verifying ZK proofs
//! of EVM execution.

mod io;
mod prover;
mod verifier;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use shadow_evm_core::prelude::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "shadow-evm")]
#[command(about = "ZK Coprocessor for verifiable off-chain Ethereum execution")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a ZK proof for an EVM execution
    Prove {
        /// Path to the execution input JSON file
        #[arg(short, long)]
        input: PathBuf,

        /// Path to save the proof receipt
        #[arg(short, long)]
        output: PathBuf,

        /// Enable dev mode (faster but not cryptographically secure)
        #[arg(long)]
        dev: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Verify a ZK proof
    Verify {
        /// Path to the proof receipt file
        #[arg(short, long)]
        proof: PathBuf,

        /// Expected commitment (optional)
        #[arg(long)]
        commitment: Option<String>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Export proof for on-chain verification
    Export {
        /// Path to the proof receipt file
        #[arg(short, long)]
        proof: PathBuf,

        /// Path to save the exported proof
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Get the guest image ID
    ImageId,

    /// Execute without proving (for testing)
    Execute {
        /// Path to the execution input JSON file
        #[arg(short, long)]
        input: PathBuf,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Generate a sample input file
    Sample {
        /// Path to save the sample input
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prove {
            input,
            output,
            dev,
            verbose,
        } => {
            cmd_prove(input, output, dev, verbose).await?;
        }
        Commands::Verify {
            proof,
            commitment,
            verbose,
        } => {
            cmd_verify(proof, commitment, verbose).await?;
        }
        Commands::Export { proof, output } => {
            cmd_export(proof, output).await?;
        }
        Commands::ImageId => {
            cmd_image_id()?;
        }
        Commands::Execute { input, verbose } => {
            cmd_execute(input, verbose).await?;
        }
        Commands::Sample { output } => {
            cmd_sample(output)?;
        }
    }

    Ok(())
}

async fn cmd_prove(
    input_path: PathBuf,
    output_path: PathBuf,
    dev: bool,
    verbose: bool,
) -> Result<()> {
    println!("Shadow-EVM ZK Prover");
    println!("====================");

    // Load input
    let input = io::load_input_json(&input_path)
        .context(format!("Failed to load input from {:?}", input_path))?;

    if verbose {
        println!("Loaded input from {:?}", input_path);
        println!("  Caller: {:?}", input.caller());
        println!("  Target: {:?}", input.target());
        println!("  Pre-state root: 0x{}", hex::encode(input.pre_state_root().as_slice()));
    }

    // Generate proof
    let options = prover::ProveOptions {
        dev_mode: dev,
        verbose,
    };

    let result = prover::prove(input, &options)?;

    // Save receipt
    let receipt_bytes = bincode::serialize(&result.receipt)?;
    io::save_bytes(&receipt_bytes, &output_path)?;

    println!("\nProof generated successfully!");
    println!("  Output: {:?}", output_path);
    println!("  Proving time: {}ms", result.proving_time_ms);
    println!("  Cycles: {}", result.cycles);
    println!("\nCommitment:");
    println!("{}", io::format_commitment(&result.commitment));

    Ok(())
}

async fn cmd_verify(
    proof_path: PathBuf,
    expected_commitment: Option<String>,
    verbose: bool,
) -> Result<()> {
    println!("Shadow-EVM ZK Verifier");
    println!("======================");

    // Load receipt
    let receipt_bytes = io::load_bytes(&proof_path)?;
    let receipt: risc0_zkvm::Receipt = bincode::deserialize(&receipt_bytes)?;

    if verbose {
        println!("Loaded proof from {:?}", proof_path);
    }

    // Verify
    let result = verifier::verify(&receipt)?;

    if result.valid {
        println!("✓ Proof is VALID");
        println!("\nCommitment:");
        println!("{}", io::format_commitment(&result.commitment));

        // Check expected commitment if provided
        if let Some(expected) = expected_commitment {
            let expected_bytes = hex::decode(expected.trim_start_matches("0x"))?;
            let expected_hash = Hash::from_slice(&expected_bytes);
            
            if result.commitment.commitment == expected_hash {
                println!("\n✓ Commitment matches expected value");
            } else {
                println!("\n✗ Commitment does NOT match expected value");
                return Ok(());
            }
        }
    } else {
        println!("✗ Proof is INVALID");
        if let Some(error) = result.error {
            println!("  Error: {}", error);
        }
    }

    Ok(())
}

async fn cmd_export(proof_path: PathBuf, output_path: PathBuf) -> Result<()> {
    println!("Shadow-EVM Proof Export");
    println!("=======================");

    // Load receipt
    let receipt_bytes = io::load_bytes(&proof_path)?;
    let receipt: risc0_zkvm::Receipt = bincode::deserialize(&receipt_bytes)?;

    // Export for on-chain
    let onchain = verifier::export_for_onchain(&receipt)?;

    // Save as JSON
    let export = serde_json::json!({
        "seal": hex::encode(&onchain.seal),
        "imageId": hex::encode(&onchain.image_id),
        "journal": hex::encode(&onchain.journal),
    });

    let json = serde_json::to_string_pretty(&export)?;
    std::fs::write(&output_path, json)?;

    println!("Exported proof to {:?}", output_path);
    println!("  Image ID: 0x{}", hex::encode(&onchain.image_id));
    println!("  Seal size: {} bytes", onchain.seal.len());
    println!("  Journal size: {} bytes", onchain.journal.len());

    Ok(())
}

fn cmd_image_id() -> Result<()> {
    let image_id = prover::get_image_id();
    println!("Guest Image ID: 0x{}", hex::encode(&image_id));
    Ok(())
}

async fn cmd_execute(input_path: PathBuf, verbose: bool) -> Result<()> {
    println!("Shadow-EVM Execute (no proof)");
    println!("=============================");

    // Load input
    let input = io::load_input_json(&input_path)?;

    if verbose {
        println!("Loaded input from {:?}", input_path);
    }

    // Execute without proving
    let commitment = prover::execute_only(input)?;

    println!("Execution complete!");
    println!("\nCommitment:");
    println!("{}", io::format_commitment(&commitment));

    Ok(())
}

fn cmd_sample(output_path: PathBuf) -> Result<()> {
    println!("Generating sample input...");

    let sender = Address::repeat_byte(0x01);
    let receiver = Address::repeat_byte(0x02);

    let mut state = InMemoryDB::new();
    state.insert_account(
        sender,
        AccountState::new_with_balance(U256::from(10u128.pow(18))),
    );

    let input = ExecutionInput::new(
        BlockEnv::default(),
        TxInput::transfer(sender, receiver, U256::from(1_000_000_000_000_000u64)),
        state,
    );

    io::save_input_json(&input, &output_path)?;

    println!("Sample input saved to {:?}", output_path);
    println!("  Sender: {:?}", sender);
    println!("  Receiver: {:?}", receiver);
    println!("  Value: 0.001 ETH");

    Ok(())
}
