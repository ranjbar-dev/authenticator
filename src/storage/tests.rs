use super::*;
use crate::domain::{
    otp::{OtpSettings, Secret},
    records::*,
};
use std::{io::Write, process::Command};

type TestResult = Result<(), Box<dyn std::error::Error>>;
#[test]
fn account_with_recovery_and_edits_are_atomic_on_fault_cancel_and_validation() -> TestResult {
    use crate::ui::{import_review::ImportReview, state::SaveIntent};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    let (dir, mut store) = fixture()?;
    let original = fs::read(dir.path().join("vault.bin"))?;
    let accounts = store.vault()?.accounts().len();
    let sets = store.vault()?.backup_sets().len();
    let draft = store.vault()?.accounts()[0].credential().clone();
    let intent = || SaveIntent::Create {
        draft: draft.clone(),
        codes: vec![
            Zeroizing::new("new-code-one".into()),
            Zeroizing::new("new-code-two".into()),
        ],
    };
    for fault in [
        files::WriteFault::AfterPartial,
        files::WriteFault::BeforeReplace,
    ] {
        store.fault = fault;
        let result = store.transaction_cancellable(&OperationCancellation::default(), |v| {
            intent().apply(v, Timestamp(10))
        });
        assert_eq!(result.err(), Some(AppError::StorageUnavailable));
        assert_eq!(store.vault()?.accounts().len(), accounts);
        assert_eq!(store.vault()?.backup_sets().len(), sets);
        assert_eq!(fs::read(dir.path().join("vault.bin"))?, original);
        assert_eq!(fs::read_dir(dir.path())?.count(), 2);
    }
    store.fault = files::WriteFault::None;
    let allowed = Arc::new(AtomicBool::new(true));
    let guard = allowed.clone();
    let cancel = OperationCancellation::guarded(move || guard.load(Ordering::SeqCst));
    let result = store.transaction_cancellable(&cancel, |v| {
        let ids = intent().apply(v, Timestamp(10))?;
        allowed.store(false, Ordering::SeqCst);
        Ok(ids)
    });
    assert_eq!(result.err(), Some(AppError::VaultLocked));
    assert_eq!(store.vault()?.accounts().len(), accounts);
    assert_eq!(fs::read(dir.path().join("vault.bin"))?, original);
    // Duplicate values bypassing normalizer fail after candidate account add,
    // proving the enclosing transaction never commits a partial account.
    let result = store.transaction(|v| {
        SaveIntent::Create {
            draft: draft.clone(),
            codes: vec![Zeroizing::new("same".into()), Zeroizing::new("same".into())],
        }
        .apply(v, Timestamp(10))
    });
    assert_eq!(result.err(), Some(AppError::InvalidRecord));
    assert_eq!(store.vault()?.accounts().len(), accounts);
    assert_eq!(fs::read(dir.path().join("vault.bin"))?, original);
    let id = store.transaction(|v| intent().apply(v, Timestamp(10)))?[0];
    assert_eq!(store.vault()?.accounts().len(), accounts + 1);
    assert_eq!(store.vault()?.backup_sets().len(), sets + 1);
    let committed = fs::read(dir.path().join("vault.bin"))?;
    let account = store
        .vault()?
        .accounts()
        .iter()
        .find(|a| a.id() == id)
        .ok_or("new account absent")?;
    let mut form = ImportReview::edit(account)?;
    form.editors[0].service.push_str(" changed");
    store.fault = files::WriteFault::BeforeReplace;
    let edit = form.save_intent(store.vault()?)?;
    assert_eq!(
        store.transaction(|v| edit.apply(v, Timestamp(11))).err(),
        Some(AppError::StorageUnavailable)
    );
    assert_eq!(fs::read(dir.path().join("vault.bin"))?, committed);
    store.fault = files::WriteFault::None;
    let edit = form.save_intent(store.vault()?)?;
    store.transaction(|v| edit.apply(v, Timestamp(11)))?;
    store.lock();
    store.unlock(pass())?;
    assert_eq!(store.vault()?.accounts().len(), accounts + 1);
    assert_eq!(store.vault()?.backup_sets().len(), sets + 1);
    let codes = store
        .vault()?
        .backup_sets()
        .iter()
        .find(|s| s.account_id() == id)
        .ok_or("recovery set absent")?
        .codes();
    assert_eq!(codes.len(), 2);
    assert!(codes.iter().all(|c| c.used_at().is_none()));
    scan(dir.path())?;
    Ok(())
}
#[test]
fn import_batch_fault_cancel_at_commit_and_success_are_atomic() -> TestResult {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    let (dir, mut store) = fixture()?;
    let original = fs::read(dir.path().join("vault.bin"))?;
    let count = store.vault()?.accounts().len();
    let draft = store.vault()?.accounts()[0].credential().clone();
    for fault in [
        files::WriteFault::AfterPartial,
        files::WriteFault::BeforeReplace,
    ] {
        store.fault = fault;
        let result = store.transaction_cancellable(&OperationCancellation::default(), |v| {
            v.add_account(draft.clone(), Timestamp(10))?;
            v.add_account(draft.clone(), Timestamp(10))
        });
        assert_eq!(result.err(), Some(AppError::StorageUnavailable));
        assert_eq!(store.vault()?.accounts().len(), count);
        assert_eq!(fs::read(dir.path().join("vault.bin"))?, original);
        assert_eq!(fs::read_dir(dir.path())?.count(), 2);
    }
    store.fault = files::WriteFault::None;
    let allowed = Arc::new(AtomicBool::new(true));
    let guard = allowed.clone();
    let cancel = OperationCancellation::guarded(move || guard.load(Ordering::SeqCst));
    let result = store.transaction_cancellable(&cancel, |v| {
        v.add_account(draft.clone(), Timestamp(10))?;
        let id = v.add_account(draft.clone(), Timestamp(10))?;
        allowed.store(false, Ordering::SeqCst); // invalidated after edits, before rename
        Ok(id)
    });
    assert_eq!(result.err(), Some(AppError::VaultLocked));
    assert_eq!(store.vault()?.accounts().len(), count);
    assert_eq!(fs::read(dir.path().join("vault.bin"))?, original);
    assert_eq!(fs::read_dir(dir.path())?.count(), 2);
    let ids = store.transaction_cancellable(&OperationCancellation::default(), |v| {
        Ok([
            v.add_account(draft.clone(), Timestamp(10))?,
            v.add_account(draft, Timestamp(10))?,
        ])
    })?;
    assert_ne!(ids[0], ids[1]);
    assert_eq!(store.vault()?.accounts().len(), count + 2);
    store.lock();
    store.unlock(pass())?;
    assert!(ids.iter().all(|id| {
        store
            .vault()
            .is_ok_and(|v| v.accounts().iter().any(|a| a.id() == *id))
    }));
    scan(dir.path())?;
    Ok(())
}
fn pass() -> Zeroizing<String> {
    Zeroizing::new("Disposable vault passphrase only".into())
}
fn params() -> KdfParameters {
    KdfParameters {
        memory_kib: 65_536,
        iterations: 3,
        lanes: 1,
    }
}
fn sample() -> Result<Vault, AppError> {
    let mut v = Vault::new(Timestamp(1));
    let a = v.add_account(
        AccountDraft::new(
            "SENTINEL-SERVICE-ONLY",
            Some("https://example.invalid/disposable"),
            Some("SENTINEL-USER-ONLY"),
            Secret::from_base32("JBSWY3DPEHPK3PXP")?,
            OtpSettings::default(),
        )?,
        Timestamp(2),
    )?;
    let folder = v.add_folder("SENTINEL-FOLDER-ONLY", FolderColor::Purple, Timestamp(3))?;
    v.add_membership(folder, a, Timestamp(4))?;
    v.add_backup_set(
        a,
        vec![BackupCode::new("SENTINEL-RECOVERY-ONLY", Timestamp(5))?],
        Timestamp(5),
    )?;
    v.accounts[0].state.deletion = DeletionState::Trashed { at: Timestamp(2) };
    v.backup_sets[0].codes[0].used_at = Some(Timestamp(5));
    v.preferences.sort = SortOrder::AccountDescending;
    v.preferences.view = ViewMode::Grid;
    v.preferences.auto_start = true;
    v.preferences.lock_timeout_seconds = 421;
    v.preferences.clipboard_timeout_seconds = 19;
    v.validate()?;
    Ok(v)
}
fn fixture() -> Result<(tempfile::TempDir, VaultStore), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let mut store = VaultStore::open_directory(dir.path())?;
    store.create_with_parameters(pass(), sample()?, params())?;
    Ok((dir, store))
}
fn scan(dir: &Path) -> TestResult {
    for file in fs::read_dir(dir)? {
        let file = file?;
        // Windows' mandatory exclusive byte-range lock blocks reading even an
        // empty lock file. It contains no serialized data by construction.
        if file.file_name() == "session.lock" {
            assert_eq!(file.metadata()?.len(), 0);
            continue;
        }
        let bytes = fs::read(file.path())?;
        for sentinel in [
            "SENTINEL",
            "JBSWY3DPEHPK3PXP",
            "Hello!",
            "Disposable vault passphrase only",
            "example.invalid",
        ] {
            assert!(
                !bytes
                    .windows(sentinel.len())
                    .any(|w| w == sentinel.as_bytes()),
                "plaintext marker persisted"
            );
        }
    }
    Ok(())
}
#[test]
fn restart_preserves_entire_aggregate_without_plaintext_artifacts() -> TestResult {
    let (dir, mut store) = fixture()?;
    let expected = codec::encode(store.vault()?)?;
    store.transaction(|v| {
        v.state.updated_at = Timestamp(6);
        v.preferences.state.updated_at = Timestamp(6);
        Ok(())
    })?;
    let expected_after = codec::encode(store.vault()?)?;
    assert_ne!(*expected, *expected_after);
    drop(store);
    let mut restarted = VaultStore::open_directory(dir.path())?;
    assert!(!restarted.is_unlocked());
    assert_eq!(restarted.vault().err(), Some(AppError::VaultLocked));
    restarted.unlock(pass())?;
    assert_eq!(*codec::encode(restarted.vault()?)?, *expected_after);
    assert_eq!(fs::read_dir(dir.path())?.count(), 2);
    scan(dir.path())?;
    Ok(())
}
#[test]
fn wrong_passphrase_tamper_truncate_and_oversize_never_mutate() -> TestResult {
    let (dir, mut store) = fixture()?;
    let original = fs::read(dir.path().join("vault.bin"))?;
    store.lock();
    assert_eq!(
        store
            .unlock(Zeroizing::new("Incorrect disposable passphrase".into()))
            .err(),
        Some(AppError::InvalidVault)
    );
    assert_eq!(fs::read(dir.path().join("vault.bin"))?, original);
    // Structural rejection occurs before expensive KDF. Authenticate meaningful
    // header, wrapped key, payload nonce and payload flips with a real KDF.
    for offset in [0, 8, 12, 20, 24, 28, 44, 68, 116, 140, original.len() - 1] {
        let mut b = original.clone();
        b[offset] ^= 0x40;
        fs::write(dir.path().join("vault.bin"), &b)?;
        assert!(store.unlock(pass()).is_err());
        assert!(!store.is_unlocked());
        assert_eq!(fs::read(dir.path().join("vault.bin"))?, b);
    }
    for n in [0, 7, 67, 143, original.len() - 1] {
        let b = &original[..n];
        fs::write(dir.path().join("vault.bin"), b)?;
        assert!(store.unlock(pass()).is_err());
        assert_eq!(fs::read(dir.path().join("vault.bin"))?, b);
    }
    let mut b = original.clone();
    b[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
    fs::write(dir.path().join("vault.bin"), &b)?;
    assert!(store.unlock(pass()).is_err());
    let f = std::fs::OpenOptions::new()
        .write(true)
        .open(dir.path().join("vault.bin"))?;
    f.set_len((crypto::MAX_FILE + 1) as u64)?;
    drop(f);
    assert!(store.unlock(pass()).is_err());
    assert_eq!(
        fs::metadata(dir.path().join("vault.bin"))?.len(),
        (crypto::MAX_FILE + 1) as u64
    );
    fs::write(dir.path().join("vault.bin"), &original)?;
    store.unlock(pass())?;
    Ok(())
}
#[test]
fn malicious_lengths_enums_versions_and_noncanonical_dtos_are_bounded() -> TestResult {
    let v = sample()?;
    let plain = codec::encode(&v)?;
    for version in [0, 2, u32::MAX] {
        let mut b = plain.clone();
        b[..4].copy_from_slice(&version.to_le_bytes());
        assert_eq!(codec::decode(&b).err(), Some(AppError::UnsupportedVault));
    }
    // Exhaustively truncate a small complete aggregate at every byte boundary.
    for n in 0..plain.len() {
        assert!(codec::decode(&plain[..n]).is_err());
    }
    let mut b = plain.clone();
    b.extend_from_slice(&[0]);
    assert!(codec::decode(&b).is_err());
    // Account count follows schema + two identities/states + preferences (81+11).
    let count_offset = 4 + 33 + 33 + 11;
    let mut b = plain.clone();
    b[count_offset..count_offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(codec::decode(&b).is_err());
    for p in [
        KdfParameters {
            memory_kib: 0,
            ..params()
        },
        KdfParameters {
            memory_kib: u32::MAX,
            ..params()
        },
        KdfParameters {
            iterations: 7,
            ..params()
        },
        KdfParameters {
            lanes: 0,
            ..params()
        },
        KdfParameters {
            lanes: 5,
            ..params()
        },
    ] {
        assert!(p.validate().is_err());
    }
    Ok(())
}
#[test]
fn authenticated_future_schema_and_failed_migration_preserve_original() -> TestResult {
    let (dir, mut store) = fixture()?;
    let original = fs::read(dir.path().join("vault.bin"))?;
    let before = codec::encode(store.vault()?)?;
    assert!(
        store
            .transaction(|v| {
                v.schema_version = 2;
                Ok(())
            })
            .is_err()
    );
    assert_eq!(fs::read(dir.path().join("vault.bin"))?, original);
    assert_eq!(*codec::encode(store.vault()?)?, *before);
    let s = store.session.as_ref().ok_or(AppError::VaultLocked)?;
    let mut unsupported = before.clone();
    unsupported[..4].copy_from_slice(&2u32.to_le_bytes());
    let future = crypto::encrypt(&s.wrapping, &s.dek, unsupported)?;
    fs::write(dir.path().join("vault.bin"), &future)?;
    store.lock();
    assert_eq!(store.unlock(pass()).err(), Some(AppError::UnsupportedVault));
    assert_eq!(fs::read(dir.path().join("vault.bin"))?, future);
    assert!(
        store
            .create_with_parameters(pass(), sample()?, params())
            .is_err()
    );
    assert_eq!(fs::read(dir.path().join("vault.bin"))?, future);
    Ok(())
}
#[test]
fn failed_cancelled_invalid_and_panicking_transactions_are_rollback_safe() -> TestResult {
    let (dir, mut store) = fixture()?;
    let original = fs::read(dir.path().join("vault.bin"))?;
    let before = codec::encode(store.vault()?)?;
    for fault in [
        files::WriteFault::AfterPartial,
        files::WriteFault::BeforeReplace,
    ] {
        store.fault = fault;
        assert!(
            store
                .transaction(|v| v.add_folder("Never committed", FolderColor::Red, Timestamp(9)))
                .is_err()
        );
        assert_eq!(fs::read(dir.path().join("vault.bin"))?, original);
        assert_eq!(*codec::encode(store.vault()?)?, *before);
        assert_eq!(fs::read_dir(dir.path())?.count(), 2);
    }
    store.fault = files::WriteFault::None;
    assert!(
        store
            .transaction::<()>(|v| {
                v.accounts.clear();
                Err(AppError::InvalidRecord)
            })
            .is_err()
    );
    assert!(
        store
            .transaction(|v| {
                v.accounts.clear();
                Ok(())
            })
            .is_err()
    );
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = store.transaction::<()>(|v| {
            v.accounts.clear();
            panic!("disposable cancellation")
        });
    }));
    assert!(result.is_err());
    assert_eq!(fs::read(dir.path().join("vault.bin"))?, original);
    assert_eq!(*codec::encode(store.vault()?)?, *before);
    store.transaction(|v| v.add_folder("Committed", FolderColor::Green, Timestamp(9)))?;
    scan(dir.path())?;
    Ok(())
}
#[test]
fn rewrap_changes_only_wrapping_and_failure_preserves_old_unlock() -> TestResult {
    let (dir, mut store) = fixture()?;
    let original = fs::read(dir.path().join("vault.bin"))?;
    let next = || Zeroizing::new("Replacement disposable passphrase".into());
    store.fault = files::WriteFault::BeforeReplace;
    assert!(store.rewrap_with_parameters(next(), params()).is_err());
    assert_eq!(fs::read(dir.path().join("vault.bin"))?, original);
    store.lock();
    store.unlock(pass())?;
    store.fault = files::WriteFault::None;
    store.rewrap_with_parameters(next(), params())?;
    let changed = fs::read(dir.path().join("vault.bin"))?;
    assert_ne!(&changed[..116], &original[..116]);
    assert_eq!(&changed[116..], &original[116..]);
    store.lock();
    assert!(store.session.is_none());
    assert!(store.committed.is_none());
    assert!(store.unlock(pass()).is_err());
    store.unlock(next())?;
    assert_eq!(store.vault()?.accounts().len(), 1);
    assert!(!format!("{store:?}").contains("SENTINEL"));
    Ok(())
}
#[test]
fn concurrent_instances_are_rejected_in_process_and_across_processes() -> TestResult {
    let (dir, store) = fixture()?;
    assert_eq!(
        VaultStore::open_directory(dir.path()).err(),
        Some(AppError::VaultBusy)
    );
    let status = Command::new(std::env::current_exe()?)
        .args([
            "--ignored",
            "--exact",
            "storage::tests::probe_instance_lock",
        ])
        .env("AUTH_TEST_DIRECTORY", dir.path())
        .status()?;
    assert!(status.success());
    drop(store);
    let _next = VaultStore::open_directory(dir.path())?;
    Ok(())
}
#[test]
#[ignore = "subprocess helper only"]
fn probe_instance_lock() -> TestResult {
    let directory =
        std::env::var_os("AUTH_TEST_DIRECTORY").ok_or("missing disposable directory")?;
    assert_eq!(
        VaultStore::open_directory(PathBuf::from(directory)).err(),
        Some(AppError::VaultBusy)
    );
    Ok(())
}
#[test]
fn terminated_writer_keeps_last_commit_and_leaves_only_ciphertext() -> TestResult {
    let (dir, store) = fixture()?;
    let original = fs::read(dir.path().join("vault.bin"))?;
    drop(store);
    let status = Command::new(std::env::current_exe()?)
        .args([
            "--ignored",
            "--exact",
            "storage::tests::probe_interrupted_writer",
        ])
        .env("AUTH_TEST_DIRECTORY", dir.path())
        .status()?;
    assert_eq!(status.code(), Some(73));
    assert_eq!(fs::read(dir.path().join("vault.bin"))?, original);
    assert_eq!(fs::read_dir(dir.path())?.count(), 3);
    scan(dir.path())?;
    let mut store = VaultStore::open_directory(dir.path())?;
    store.unlock(pass())?;
    assert_eq!(store.vault()?.folders().len(), 1);
    Ok(())
}
#[test]
#[ignore = "subprocess helper only"]
fn probe_interrupted_writer() -> TestResult {
    let directory =
        std::env::var_os("AUTH_TEST_DIRECTORY").ok_or("missing disposable directory")?;
    let mut store = VaultStore::open_directory(PathBuf::from(directory))?;
    store.unlock(pass())?;
    store.fault = files::WriteFault::TerminateBeforeReplace;
    store
        .transaction(|v| v.add_folder("SENTINEL-PENDING-ONLY", FolderColor::Blue, Timestamp(9)))?;
    Err("expected termination".into())
}
#[test]
fn external_file_changes_and_existing_corruption_are_never_overwritten() -> TestResult {
    let (dir, mut store) = fixture()?;
    let before = codec::encode(store.vault()?)?;
    fs::OpenOptions::new()
        .append(true)
        .open(dir.path().join("vault.bin"))?
        .write_all(b"corruption")?;
    let corrupt = fs::read(dir.path().join("vault.bin"))?;
    assert!(
        store
            .transaction(|v| v.add_folder("No overwrite", FolderColor::Blue, Timestamp(9)))
            .is_err()
    );
    assert_eq!(*codec::encode(store.vault()?)?, *before);
    assert_eq!(fs::read(dir.path().join("vault.bin"))?, corrupt);
    assert!(
        store
            .create_with_parameters(pass(), sample()?, params())
            .is_err()
    );
    Ok(())
}
#[test]
fn calibration_and_randomness_use_bounded_production_policy() -> TestResult {
    let start = std::time::Instant::now();
    let p = KdfParameters::calibrate()?;
    p.validate()?;
    println!(
        "calibration memory_kib={} iterations={} lanes={} probe_elapsed_ms={}",
        p.memory_kib,
        p.iterations,
        p.lanes,
        start.elapsed().as_millis()
    );
    assert_eq!(p.memory_kib, 65_536);
    assert_ne!(crypto::random::<32>()?, crypto::random::<32>()?);
    for length in [0, 11, 1025] {
        assert!(crypto::check_passphrase(&vec![b'x'; length]).is_err());
    }
    Ok(())
}

#[cfg(windows)]
#[test]
fn windows_dacl_is_protected_and_only_current_user_has_access() -> TestResult {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        Security::*,
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    let (dir, store) = fixture()?;
    let paths = [
        dir.path().to_owned(),
        dir.path().join("vault.bin"),
        dir.path().join("session.lock"),
    ];
    // SAFETY: Fixed aligned buffers, validated successful native calls and
    // lifetimes. All pointers remain inside live native descriptor/token buffers.
    unsafe {
        let mut token = std::ptr::null_mut();
        assert_ne!(
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token),
            0
        );
        let mut user = [0usize; 512];
        let mut needed = 0;
        let ok = GetTokenInformation(
            token,
            TokenUser,
            user.as_mut_ptr().cast(),
            std::mem::size_of_val(&user) as u32,
            &mut needed,
        );
        CloseHandle(token);
        assert_ne!(ok, 0);
        let current = &*user.as_ptr().cast::<TOKEN_USER>();
        for path in paths {
            let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
            let mut sd = [0usize; 512];
            let sd_ptr = sd.as_mut_ptr().cast();
            assert_ne!(
                GetFileSecurityW(
                    wide.as_ptr(),
                    DACL_SECURITY_INFORMATION,
                    sd_ptr,
                    std::mem::size_of_val(&sd) as u32,
                    &mut needed
                ),
                0
            );
            let mut control = 0;
            let mut revision = 0;
            assert_ne!(
                GetSecurityDescriptorControl(sd_ptr, &mut control, &mut revision),
                0
            );
            assert_ne!(control & SE_DACL_PROTECTED, 0);
            let mut present = 0;
            let mut defaulted = 0;
            let mut acl = std::ptr::null_mut();
            assert_ne!(
                GetSecurityDescriptorDacl(sd_ptr, &mut present, &mut acl, &mut defaulted),
                0
            );
            assert_ne!(present, 0);
            assert!(!acl.is_null());
            assert_eq!((*acl).AceCount, 1);
            let mut ace = std::ptr::null_mut();
            assert_ne!(GetAce(acl, 0, &mut ace), 0);
            let allowed = &*ace.cast::<ACCESS_ALLOWED_ACE>();
            assert_eq!(allowed.Header.AceType, 0); // ACCESS_ALLOWED_ACE_TYPE
            assert_ne!(
                EqualSid(
                    (&raw const allowed.SidStart).cast_mut().cast(),
                    current.User.Sid
                ),
                0
            );
            assert_eq!(allowed.Mask & 0x1f01ff, 0x1f01ff);
        }
    }
    drop(store);
    Ok(())
}

#[cfg(windows)]
#[test]
fn native_windows_replace_failure_preserves_memory_and_file() -> TestResult {
    use std::os::windows::fs::OpenOptionsExt;
    let (dir, mut store) = fixture()?;
    let before = codec::encode(store.vault()?)?;
    let original = fs::read(dir.path().join("vault.bin"))?;
    // Real sharing violation: another reader denies DELETE sharing required by rename.
    let blocker = fs::OpenOptions::new()
        .read(true)
        .share_mode(3)
        .open(dir.path().join("vault.bin"))?;
    assert!(
        store
            .transaction(|v| v.add_folder("Not committed", FolderColor::Red, Timestamp(9)))
            .is_err()
    );
    assert_eq!(*codec::encode(store.vault()?)?, *before);
    assert_eq!(fs::read(dir.path().join("vault.bin"))?, original);
    assert_eq!(fs::read_dir(dir.path())?.count(), 2);
    drop(blocker);
    store.transaction(|v| {
        v.add_folder(
            "Committed after sharing release",
            FolderColor::Red,
            Timestamp(9),
        )
    })?;
    Ok(())
}
