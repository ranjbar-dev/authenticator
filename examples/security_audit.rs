//! Disposable command-line audit probe, never a production entrypoint or vault.
use authenticator::{
    backup::{self, Destination, ImportOptions},
    domain::{
        clock::FixedClock,
        otp::current_code,
        records::{BackupCode, FolderColor, Timestamp, Vault},
    },
    qr::{self, PixelLayout, ReviewCandidate},
    storage::{OperationCancellation, VaultStore},
};
use qrcode::{Color, QrCode};
use std::{fs, path::Path, time::Duration};
use zeroize::Zeroizing;

const URI: &str = "otpauth://totp/AUDIT_SERVICE_SENTINEL:AUDIT_ACCOUNT_SENTINEL?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=AUDIT_SERVICE_SENTINEL&digits=8";
fn pass() -> Zeroizing<String> {
    Zeroizing::new("Public disposable audit passphrase".into())
}
fn scan_owned(directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            scan_owned(&entry.path())?;
        } else if entry.file_name() == "session.lock" {
            assert_eq!(entry.metadata()?.len(), 0);
        } else {
            let bytes = fs::read(entry.path())?;
            for sentinel in [
                "AUDIT_SERVICE_SENTINEL",
                "AUDIT_ACCOUNT_SENTINEL",
                "AUDIT_RECOVERY_SENTINEL",
                "AUDIT_FOLDER_SENTINEL",
                "Public disposable audit passphrase",
                "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
                "12345678901234567890",
                URI,
            ] {
                assert!(
                    !bytes
                        .windows(sentinel.len())
                        .any(|b| b == sentinel.as_bytes()),
                    "plaintext sentinel in owned audit artifacts"
                );
            }
        }
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::panic::set_hook(Box::new(|_| {}));
    let directory = tempfile::tempdir()?;
    let code = QrCode::new(URI.as_bytes())?;
    let scale = 6;
    let side = (code.width() + 8) * scale;
    let mut pixels = Zeroizing::new(vec![255u8; side * side]);
    for y in 0..code.width() {
        for x in 0..code.width() {
            if code[(x, y)] == Color::Dark {
                for dy in 0..scale {
                    for dx in 0..scale {
                        pixels[((y + 4) * scale + dy) * side + (x + 4) * scale + dx] = 0;
                    }
                }
            }
        }
    }
    let review = qr::decode_pixels(side as u32, side as u32, PixelLayout::Gray8, pixels)?;
    let draft = review
        .into_candidates()
        .into_iter()
        .find_map(|candidate| match candidate {
            ReviewCandidate::Supported(draft) => Some(draft),
            _ => None,
        })
        .ok_or("public audit QR was not decoded")?;
    let mut source = VaultStore::open_directory(directory.path().join("source"))?;
    source.create(pass(), Vault::new(Timestamp(10)))?;
    let account = source.transaction(|vault| {
        let account = vault.add_account(draft, Timestamp(11))?;
        let folder = vault.add_folder("AUDIT_FOLDER_SENTINEL", FolderColor::Blue, Timestamp(11))?;
        vault.add_membership(folder, account, Timestamp(11))?;
        let mut recovery = BackupCode::new("AUDIT_RECOVERY_SENTINEL", Timestamp(11))?;
        recovery.set_used(true, Timestamp(11))?;
        vault.add_backup_set(account, vec![recovery], Timestamp(11))?;
        Ok(account)
    })?;
    let credential = source.vault()?.accounts()[0].credential();
    assert_eq!(
        current_code(credential.secret(), credential.settings(), &FixedClock(59))?.expose_value(),
        "94287082"
    );
    let backup_path = directory.path().join("portable.authpbk");
    let cancel = OperationCancellation::default();
    backup::export(
        &source,
        Destination::create_new(&backup_path)?,
        pass(),
        &cancel,
    )?;
    scan_owned(directory.path())?;
    std::thread::sleep(Duration::from_millis(500));
    let mut restored = VaultStore::open_directory(directory.path().join("restored"))?;
    restored.create(pass(), Vault::new(Timestamp(10)))?;
    backup::preview(&restored, &backup_path, pass(), &cancel)?
        .plan(ImportOptions::default())?
        .apply(&mut restored, &cancel)?;
    assert_eq!(restored.vault()?.accounts()[0].id(), account);
    assert_eq!(
        restored.vault()?.backup_sets()[0].codes()[0].used_at(),
        Some(Timestamp(11))
    );
    restored.lock();
    restored.unlock(pass())?;
    let credential = restored.vault()?.accounts()[0].credential();
    assert_eq!(
        current_code(credential.secret(), credential.settings(), &FixedClock(59))?.expose_value(),
        "94287082"
    );
    scan_owned(directory.path())?;
    std::thread::sleep(Duration::from_millis(500));
    println!("Disposable core audit passed; no native UI or disconnected-machine claim.");
    Ok(())
}
