use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    message::Message,
    hash::Hash,
    pubkey::Pubkey,
};
use solana_system_interface::instruction as system_instruction;
use std::str::FromStr;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 4 {
        eprintln!("Usage: tx_generator <blockhash> <payer_pubkey> <recipient_pubkey>");
        eprintln!("\nExample:");
        eprintln!("  tx_generator 11111111111111111111111111111111 Payer... Recipient...");
        std::process::exit(1);
    }
    
    let blockhash_str = &args[1];
    let payer_str = &args[2];
    let recipient_str = &args[3];
    
    // Parse inputs 
    let blockhash = Hash::from_str(blockhash_str)
        .expect("Invalid blockhash");
    let payer_pubkey = Pubkey::from_str(payer_str)
        .expect("Invalid payer pubkey");
    let recipient_pubkey = Pubkey::from_str(recipient_str)
        .expect("Invalid recipient pubkey");
    
    // For testing, we need the payer's private key
    // Since we can't get it from just the pubkey, we'll create a new keypair
    // and tell the user to fund THIS address instead
    let payer_keypair = Keypair::new();
    
    println!("⚠️  NOTE: Generated new keypair for transaction signing");
    println!("📝 Payer address (use this in set_balance): {}", payer_keypair.pubkey());
    println!("📝 Recipient address: {}", recipient_pubkey);
    println!();
    
    // Create transfer instruction (1 SOL)
    let transfer_amount = 1_000_000_000; // 1 SOL
    let instruction = system_instruction::transfer(
        &payer_keypair.pubkey(),
        &recipient_pubkey,
        transfer_amount,
    );
    
    // Create transaction
    let message = Message::new(&[instruction], Some(&payer_keypair.pubkey()));
    let mut tx = Transaction::new_unsigned(message);
    tx.message.recent_blockhash = blockhash;
    
    // Sign transaction
    tx.sign(&[&payer_keypair], blockhash);
    
    // Serialize and encode
    let serialized = bincode::serialize(&tx).unwrap();
    let encoded = bs58::encode(serialized).into_string();
    
    println!("✅ Transaction created successfully!");
    println!("\n📦 Encoded transaction:");
    println!("{}", encoded);
    println!();
    println!("💡 Transfer details:");
    println!("   From: {}", payer_keypair.pubkey());
    println!("   To:   {}", recipient_pubkey);
    println!("   Amount: {} lamports (1 SOL)", transfer_amount);
}