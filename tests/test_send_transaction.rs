use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    message::Message,
    hash::Hash,
};

use solana_system_interface::instruction as system_instruction;

fn create_test_transaction() -> String {
    // Create keypairs
    let payer = Keypair::new();
    let recipient = Keypair::new();
    
    // Create transfer instruction
    let instruction = system_instruction::transfer(
        &payer.pubkey(),
        &recipient.pubkey(),
        1_000_000_000, // 1 SOL
    );
    
    // Create transaction
    let message = Message::new(&[instruction], Some(&payer.pubkey()));
    let mut tx = Transaction::new_unsigned(message);
    
    // Use a dummy blockhash for testing
    // tx.message.recent_blockhash = Hash::default();
    tx.message.recent_blockhash = CmpNeggWJ4JaWJeJ8YKN1Zypmk7uvQq3PECGUCAEMbky;
    
    // Sign transaction
    tx.sign(&[&payer], Hash::default());
    

    println!("Payer pubkey : {}", payer.pubkey());
    println!("Receipient pubkey : {}", recipient.pubkey());
    // Serialize and encode to base58
    let serialized = bincode::serialize(&tx).unwrap();
    bs58::encode(serialized).into_string()
}

#[tokio::test]
async fn test_send_transaction() {
    let tx_data = create_test_transaction();
    println!("Transaction data: {}", tx_data);
    
    // Now send this via your API
    // You'd use reqwest or similar to POST to your server
}