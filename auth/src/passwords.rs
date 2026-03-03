use eva::auto_impl;

use viendesu_core::types::user::PasswordHash;

/// Passwords generation utility.
#[auto_impl(&, &mut, Arc)]
pub trait Passwords: Send + Sync {
    fn verify(&self, hash: &str, cleartext: &str) -> bool;
    fn make(&self, cleartext: &str) -> PasswordHash;
}

/// Plaintext passwords, no hashing.
#[derive(Debug, Clone, Copy)]
pub struct Plaintext;

impl Passwords for Plaintext {
    fn verify(&self, hash: &str, cleartext: &str) -> bool {
        hash == cleartext
    }

    fn make(&self, cleartext: &str) -> PasswordHash {
        cleartext.parse().unwrap()
    }
}
