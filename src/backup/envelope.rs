use super::*;
use crate::storage::crypto;

pub(super) const MAGIC: &[u8; 8] = b"AUTHPBK1";
pub(super) const HEADER: usize = 144;
pub(super) const MAX_FILE: usize = HEADER + codec::MAX_PLAINTEXT + 16;
const DOMAIN: &[u8] = b"Authenticator portable backup v1\0";

fn aad(header: &[u8]) -> Vec<u8> {
    let mut bytes = DOMAIN.to_vec();
    bytes.extend_from_slice(header);
    bytes
}
pub(super) fn encrypt(vault: &Vault, pass: &[u8], p: KdfParameters) -> Result<Vec<u8>> {
    encrypt_plain(
        codec::encode(vault).map_err(|_| BackupError::Invalid)?,
        pass,
        p,
    )
}
pub(super) fn encrypt_plain(
    mut plain: Zeroizing<Vec<u8>>,
    pass: &[u8],
    p: KdfParameters,
) -> Result<Vec<u8>> {
    if plain.len() > codec::MAX_PLAINTEXT {
        return Err(BackupError::Limit);
    }
    p.validate().map_err(|_| BackupError::Invalid)?;
    crypto::check_passphrase(pass).map_err(|_| BackupError::InvalidDecision)?;
    let mut header = Vec::with_capacity(HEADER);
    header.extend_from_slice(MAGIC);
    // envelope, KDF algorithm/version, memory, rounds, lanes
    for word in [1, 0x13, p.memory_kib, p.iterations, p.lanes] {
        header.extend_from_slice(&word.to_le_bytes());
    }
    header.extend_from_slice(&crypto::random::<16>().map_err(local_error)?);
    header.extend_from_slice(&crypto::random::<24>().map_err(local_error)?);
    header.extend_from_slice(&crypto::random::<24>().map_err(local_error)?);
    header.extend_from_slice(&((plain.len() + 16) as u32).to_le_bytes());
    let kek = crypto::derive(pass, &header[28..44], p).map_err(local_error)?;
    let dek = Zeroizing::new(crypto::random::<32>().map_err(local_error)?);
    let mut wrapped = Zeroizing::new(Vec::with_capacity(48));
    wrapped.extend_from_slice(&dek[..]);
    crypto::seal(&kek, &header[44..68], &aad(&header), &mut wrapped).map_err(local_error)?;
    header.extend_from_slice(&wrapped);
    crypto::seal(&dek, &header[68..92], &aad(&header), &mut plain).map_err(local_error)?;
    header.extend_from_slice(&plain);
    Ok(header)
}
pub(super) fn parse(bytes: &[u8]) -> Result<KdfParameters> {
    if !(HEADER + 16..=MAX_FILE).contains(&bytes.len()) || &bytes[..8] != MAGIC {
        return Err(BackupError::Invalid);
    }
    let word = |i: usize| u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
    if word(8) != 1 || word(12) != 0x13 {
        return Err(BackupError::Unsupported);
    }
    let p = KdfParameters {
        memory_kib: word(16),
        iterations: word(20),
        lanes: word(24),
    };
    p.validate().map_err(|_| BackupError::Invalid)?;
    if word(92) as usize != bytes.len() - HEADER {
        return Err(BackupError::Invalid);
    }
    Ok(p)
}
pub(super) fn decrypt(bytes: &[u8], pass: &[u8]) -> Result<Vault> {
    let p = parse(bytes)?;
    crypto::check_passphrase(pass).map_err(|_| BackupError::Invalid)?;
    let kek = crypto::derive(pass, &bytes[28..44], p).map_err(|_| BackupError::Invalid)?;
    let mut wrapped = Zeroizing::new(bytes[96..HEADER].to_vec());
    crypto::open(&kek, &bytes[44..68], &aad(&bytes[..96]), &mut wrapped)
        .map_err(|_| BackupError::Invalid)?;
    let mut dek = Zeroizing::new([0; 32]);
    if wrapped.len() != 32 {
        return Err(BackupError::Invalid);
    }
    dek.copy_from_slice(&wrapped);
    // Reserve tag space once: no plaintext reallocation on decrypt.
    let mut plain = Zeroizing::new(bytes[HEADER..].to_vec());
    crypto::open(&dek, &bytes[68..92], &aad(&bytes[..HEADER]), &mut plain)
        .map_err(|_| BackupError::Invalid)?;
    codec::decode(&plain).map_err(|e| {
        if e == AppError::UnsupportedVault {
            BackupError::Unsupported
        } else {
            BackupError::Invalid
        }
    })
}
