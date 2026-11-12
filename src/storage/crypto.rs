use super::codec::MAX_PLAINTEXT;
use crate::error::AppError;
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{AeadInOut, KeyInit},
};
use std::time::Instant;
use zeroize::Zeroizing;

pub(super) const MAGIC: &[u8; 8] = b"AUTHLOC1";
pub(super) const PREFIX: usize = 68;
pub(super) const HEADER: usize = 144;
pub(super) const MAX_FILE: usize = HEADER + MAX_PLAINTEXT + 16;
pub(super) type Key = Zeroizing<[u8; 32]>;

/// Fixed policy envelope; never run a KDF from unvalidated disk parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KdfParameters {
    pub memory_kib: u32,
    pub iterations: u32,
    pub lanes: u32,
}
impl KdfParameters {
    pub fn validate(self) -> Result<(), AppError> {
        if !(65_536..=131_072).contains(&self.memory_kib)
            || !(3..=6).contains(&self.iterations)
            || !(1..=4).contains(&self.lanes)
        {
            return Err(AppError::InvalidVault);
        }
        Ok(())
    }
    /// Calibrate once on creation/rewrap, targeting ~500ms with a 64MiB floor.
    /// Slow machines retain the security floor rather than lowering it.
    pub fn calibrate() -> Result<Self, AppError> {
        let mut p = Self {
            memory_kib: 65_536,
            iterations: 3,
            lanes: 1,
        };
        let salt = random::<16>()?;
        let trial = Zeroizing::new(random::<32>()?);
        let start = Instant::now();
        let _key = derive(&trial[..], &salt, p)?;
        let elapsed = start.elapsed().as_millis().max(1);
        p.iterations = ((1500u128.div_ceil(elapsed)).clamp(3, 6)) as u32;
        Ok(p)
    }
}
pub(super) fn random<const N: usize>() -> Result<[u8; N], AppError> {
    let mut b = [0; N];
    getrandom::fill(&mut b).map_err(|_| AppError::StorageUnavailable)?;
    Ok(b)
}
pub(super) fn check_passphrase(p: &[u8]) -> Result<(), AppError> {
    if !(12..=1024).contains(&p.len()) {
        return Err(AppError::InvalidPassphrase);
    }
    Ok(())
}
fn derive(pass: &[u8], salt: &[u8], p: KdfParameters) -> Result<Key, AppError> {
    p.validate()?;
    check_passphrase(pass)?;
    let params = Params::new(p.memory_kib, p.iterations, p.lanes, Some(32))
        .map_err(|_| AppError::InvalidVault)?;
    // The crate's allocating convenience method does not clear its Blocks
    // allocation. Own the bounded scratch explicitly so every return clears it.
    let mut blocks = Zeroizing::new(Vec::new());
    blocks
        .try_reserve_exact(params.block_count())
        .map_err(|_| AppError::StorageUnavailable)?;
    blocks.resize(params.block_count(), argon2::Block::default());
    let mut key = Zeroizing::new([0u8; 32]);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into_with_memory(pass, salt, &mut key[..], &mut blocks[..])
        .map_err(|_| AppError::StorageUnavailable)?;
    Ok(key)
}
fn seal(key: &Key, nonce: &[u8], aad: &[u8], plain: &mut Vec<u8>) -> Result<(), AppError> {
    let cipher =
        XChaCha20Poly1305::new_from_slice(&key[..]).map_err(|_| AppError::StorageUnavailable)?;
    let nonce = <&XNonce>::try_from(nonce).map_err(|_| AppError::StorageUnavailable)?;
    cipher
        .encrypt_in_place(nonce, aad, plain)
        .map_err(|_| AppError::StorageUnavailable)
}
fn open(key: &Key, nonce: &[u8], aad: &[u8], ciphertext: &mut Vec<u8>) -> Result<(), AppError> {
    let cipher = XChaCha20Poly1305::new_from_slice(&key[..]).map_err(|_| AppError::InvalidVault)?;
    let nonce = <&XNonce>::try_from(nonce).map_err(|_| AppError::InvalidVault)?;
    cipher
        .decrypt_in_place(nonce, aad, ciphertext)
        .map_err(|_| AppError::InvalidVault)
}
pub(super) fn new_wrapping(pass: &[u8], dek: &Key, p: KdfParameters) -> Result<Vec<u8>, AppError> {
    p.validate()?;
    let mut h = Vec::with_capacity(PREFIX + 48);
    h.extend_from_slice(MAGIC);
    h.extend_from_slice(&1u32.to_le_bytes());
    h.extend_from_slice(&0x13u32.to_le_bytes());
    for n in [p.memory_kib, p.iterations, p.lanes] {
        h.extend_from_slice(&n.to_le_bytes());
    }
    h.extend_from_slice(&random::<16>()?);
    h.extend_from_slice(&random::<24>()?);
    let kek = derive(pass, &h[28..44], p)?;
    let mut wrapped = Zeroizing::new(Vec::with_capacity(48));
    wrapped.extend_from_slice(&dek[..]);
    seal(&kek, &h[44..68], &h, &mut wrapped)?;
    h.extend_from_slice(&wrapped);
    Ok(h)
}
pub(super) fn parse(bytes: &[u8]) -> Result<KdfParameters, AppError> {
    if !(HEADER + 16..=MAX_FILE).contains(&bytes.len()) {
        return Err(AppError::InvalidVault);
    }
    if &bytes[..8] != MAGIC {
        return Err(AppError::InvalidVault);
    }
    let word = |i| u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
    if word(8) != 1 || word(12) != 0x13 {
        return Err(AppError::UnsupportedVault);
    }
    let p = KdfParameters {
        memory_kib: word(16),
        iterations: word(20),
        lanes: word(24),
    };
    p.validate()?;
    if word(140) as usize != bytes.len() - HEADER {
        return Err(AppError::InvalidVault);
    }
    Ok(p)
}
pub(super) fn unlock(bytes: &[u8], pass: &[u8]) -> Result<(Key, Zeroizing<Vec<u8>>), AppError> {
    let p = parse(bytes)?;
    let kek = derive(pass, &bytes[28..44], p)?;
    let mut wrapped = Zeroizing::new(bytes[PREFIX..116].to_vec());
    open(&kek, &bytes[44..68], &bytes[..PREFIX], &mut wrapped)?;
    let mut dek = Zeroizing::new([0; 32]);
    if wrapped.len() != 32 {
        return Err(AppError::InvalidVault);
    }
    dek.copy_from_slice(&wrapped);
    let mut plain = Zeroizing::new(bytes[HEADER..].to_vec());
    open(&dek, &bytes[116..140], &bytes[..12], &mut plain)?;
    Ok((dek, plain))
}
pub(super) fn encrypt(
    wrapping: &[u8],
    dek: &Key,
    mut plain: Zeroizing<Vec<u8>>,
) -> Result<Vec<u8>, AppError> {
    if wrapping.len() != 116 || plain.len() > MAX_PLAINTEXT {
        return Err(AppError::InvalidVault);
    }
    let nonce = random::<24>()?;
    seal(dek, &nonce, &wrapping[..12], &mut plain)?;
    let mut bytes = wrapping.to_vec();
    bytes.extend_from_slice(&nonce);
    bytes.extend_from_slice(&(plain.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&plain);
    Ok(bytes)
}
