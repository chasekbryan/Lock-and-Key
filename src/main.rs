//
// LOCK - v.0.1
//
// 1. cargo build --release
// 2. ./target/release/lock path/to/yourfile.txt
//      ###Enter passphrase: ******
//      ###Encrypted to: yourfile.lock
// 3. Decrypt (future implementation)
//
use std::{
    error::Error,
    fs,
    io::{Read, Write},
    path::PathBuf,
};

use clap::Parser;
use rpassword::prompt_password;
use rand::{rngs::OsRng, RngCore};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString},
};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, Key, XNonce,
};

/// Encrypt a file with Argon2id + XChaCha20-Poly1305 AEAD.
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// File to encrypt
    input: PathBuf,
}

fn main() {
    if let Err(e) = real_main() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn real_main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Read plaintext
    let mut plaintext = Vec::new();
    fs::File::open(&args.input)?.read_to_end(&mut plaintext)?;

    // Prompt (no-echo) for passphrase
    let password = prompt_password("Enter passphrase: ")
        .expect("Failed to read passphrase");

    // Derive a random 16-byte salt and encode to SaltString
    let mut salt_bytes = [0u8; 16];
    OsRng.fill_bytes(&mut salt_bytes);
    let salt = SaltString::b64_encode(&salt_bytes)
        .expect("Salt encoding failed");

    // Argon2id key derivation
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Argon2 hashing failed");
    // Extract the raw hash and keep it alive
    let raw_hash = password_hash.hash.unwrap();
    let key_bytes = raw_hash.as_bytes();
    let key = Key::from_slice(key_bytes);

    // XChaCha20-Poly1305 encryption
    let cipher = XChaCha20Poly1305::new(key);
    let mut nonce_bytes = [0u8; 24];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .expect("Encryption failure");

    // Write salt || nonce || ciphertext to `<input>.lock`
    let mut out_path = args.input.clone();
    out_path.set_extension("lock");
    let mut out_file = fs::File::create(&out_path)?;
    out_file.write_all(&salt_bytes)?;
    out_file.write_all(&nonce_bytes)?;
    out_file.write_all(&ciphertext)?;

    println!("Encrypted to: {}", out_path.display());
    Ok(())
}
