use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand_core::OsRng;
use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let mut csprng = OsRng;
    let keypair: Keypair = Keypair::generate(&mut csprng);

    let private_key_bytes = keypair.to_bytes();
    let public_key_bytes = keypair.public.to_bytes();

    fs::write("private_key.bin", private_key_bytes)?;
    fs::write("public_key.bin", public_key_bytes)?;

    println!("Ключи сгенерированы:");
    println!("  Закрытый ключ сохранен в private_key.bin");
    println!("  Открытый ключ сохранен в public_key.bin");
    println!(
        "  Публичный ключ (для встраивания в программу): {:?}",
        hex::encode(public_key_bytes)
    );

    Ok(())
}
