use super::*;
use crate::domain::otp::{OtpSettings, Secret};
use std::fs;
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

fn store(
    v: Vault,
) -> std::result::Result<(tempfile::TempDir, VaultStore), Box<dyn std::error::Error>> {
    store_with_pass(v, "independent local passphrase")
}
fn store_with_pass(
    v: Vault,
    phrase: &str,
) -> std::result::Result<(tempfile::TempDir, VaultStore), Box<dyn std::error::Error>> {
    let d = tempfile::tempdir()?;
    let mut s = VaultStore::open_directory(d.path())?;
    s.create_with_parameters(Zeroizing::new(phrase.into()), v, parameters())?;
    Ok((d, s))
}
fn make_backup(s: &VaultStore, d: &Path) -> TestResult {
    export_with_parameters(
        s,
        Destination::create_new(d)?,
        pass(),
        parameters(),
        &OperationCancellation::default(),
    )?;
    Ok(())
}

fn parameters() -> KdfParameters {
    KdfParameters {
        memory_kib: 65_536,
        iterations: 3,
        lanes: 1,
    }
}
fn pass() -> Zeroizing<String> {
    Zeroizing::new("disposable backup passphrase".into())
}
fn sample() -> std::result::Result<Vault, AppError> {
    let mut v = Vault::new(Timestamp(1));
    let id = v.add_account(
        AccountDraft::new(
            "Public vector",
            Some("https://example.test"),
            Some("fixture"),
            Secret::from_base32("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")?,
            OtpSettings::default(),
        )?,
        Timestamp(2),
    )?;
    let f = v.add_folder("Disposable", FolderColor::Purple, Timestamp(3))?;
    v.add_membership(f, id, Timestamp(4))?;
    let mut code = BackupCode::new("public-fixture-code", Timestamp(5))?;
    code.set_used(true, Timestamp(6))?;
    v.add_backup_set(id, vec![code], Timestamp(7))?;
    v.trash_account(id, v.accounts[0].state, Timestamp(8))?;
    Ok(v)
}
#[test]
fn portable_envelope_roundtrip_and_randomization()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let v = sample()?;
    let a = envelope::encrypt(&v, pass().as_bytes(), parameters())?;
    let b = envelope::encrypt(&v, pass().as_bytes(), parameters())?;
    assert_ne!(a, b);
    assert!(*codec::encode(&v)? == *codec::encode(&envelope::decrypt(&a, pass().as_bytes())?)?);
    assert_eq!(
        envelope::decrypt(&a, b"incorrect disposable passphrase").err(),
        Some(BackupError::Invalid)
    );
    for i in [
        0,
        8,
        12,
        16,
        20,
        24,
        28,
        43,
        44,
        67,
        68,
        91,
        92,
        95,
        96,
        143,
        144,
        a.len() - 1,
    ] {
        let mut broken = a.clone();
        broken[i] ^= 1;
        assert!(envelope::decrypt(&broken, pass().as_bytes()).is_err());
    }
    for n in [0, 8, 143, 144, a.len() - 1] {
        assert!(envelope::decrypt(&a[..n], pass().as_bytes()).is_err());
    }
    let mut extra = a.clone();
    extra.push(0);
    assert!(envelope::decrypt(&extra, pass().as_bytes()).is_err());
    Ok(())
}
#[test]
fn merge_conservative_and_keep_both_preserve_graph()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let v = sample()?;
    let conflicts = merge::conflicts(&v, &v)?;
    let (kept, s) = merge::build(&v, &v, &conflicts, ImportOptions::default())?;
    assert_eq!(s.skipped_account_bundles, 1);
    assert_eq!(kept.accounts.len(), 1);
    assert_eq!(kept.backup_sets.len(), 1);
    assert_eq!(kept.folders.len(), 1);
    assert_eq!(s.remapped_ids, 0);
    assert!(!s.folder_copies[0].copied);
    assert!(*codec::encode(&kept)? == *codec::encode(&v)?);
    let repeated_conflicts = merge::conflicts(&kept, &v)?;
    let (repeated, _) = merge::build(&kept, &v, &repeated_conflicts, Default::default())?;
    assert!(*codec::encode(&repeated)? == *codec::encode(&v)?);
    let (both, s) = merge::build(
        &v,
        &v,
        &conflicts,
        ImportOptions {
            account_choices: vec![(0, MergeChoice::KeepBoth)],
            folder_choices: vec![(0, MergeChoice::KeepBoth)],
            import_display_preferences: true,
            ..Default::default()
        },
    )?;
    both.validate()?;
    merge::unique_ids(&both)?;
    assert_eq!(s.after.accounts, 2);
    assert_eq!(s.after.used_codes, 2);
    assert_eq!(both.memberships.len(), 2);
    assert_eq!(both.backup_sets[1].account_id, both.accounts[1].id);
    assert_eq!(both.memberships[1].account_id, both.accounts[1].id);
    assert_eq!(both.memberships[1].folder_id, both.folders[1].id);
    assert_eq!(both.accounts[0].state, both.accounts[1].state);
    Ok(())
}

#[test]
fn independent_local_keys_all_entities_parameters_and_policies() -> TestResult {
    use crate::domain::otp::{Algorithm, Digits, Period};
    let mut v = sample()?;
    for algorithm in [Algorithm::Sha1, Algorithm::Sha256, Algorithm::Sha512] {
        for digits in [Digits::Six, Digits::Eight] {
            for period in [
                Period::Seconds15,
                Period::Seconds30,
                Period::Seconds60,
                Period::Seconds120,
            ] {
                let a = AccountDraft::new(
                    "Public parameters",
                    None,
                    None,
                    Secret::from_base32("MY")?,
                    OtpSettings {
                        algorithm,
                        digits,
                        period,
                    },
                )?;
                v.add_account(a, Timestamp(9))?;
            }
        }
    }
    v.preferences.auto_start = true;
    let trashed_folder = v.add_folder("Trashed public folder", FolderColor::Green, Timestamp(9))?;
    v.add_membership(trashed_folder, v.accounts[0].id, Timestamp(9))?;
    v.folders[1].state.deletion = DeletionState::Trashed { at: Timestamp(9) };
    v.memberships[1].state.deletion = DeletionState::Trashed { at: Timestamp(9) };
    v.backup_sets[0].state.updated_at = Timestamp(9);
    v.backup_sets[0].state.deletion = DeletionState::Trashed { at: Timestamp(9) };
    v.backup_sets[0].codes[0].state.updated_at = Timestamp(9);
    v.backup_sets[0].codes[0].state.deletion = DeletionState::Trashed { at: Timestamp(9) };
    v.preferences.lock_timeout_seconds = 900;
    v.preferences.clipboard_timeout_seconds = 60;
    v.preferences.sort = SortOrder::AccountDescending;
    v.preferences.view = ViewMode::Grid;
    v.preferences.query = Zeroizing::new("public".into());
    v.preferences.folder = FolderFilter::Folder(v.folders[0].id);
    let (source_dir, source) = store_with_pass(v.clone(), "source only local passphrase")?;
    let portable = tempfile::tempdir()?;
    let path = portable.path().join("portable.backup");
    make_backup(&source, &path)?;
    let (dest_dir, mut dest) = store(Vault::new(Timestamp(100)))?;
    let local_id = dest.vault()?.id;
    let pref_id = dest.vault()?.preferences.id;
    let preview = preview(&dest, &path, pass(), &OperationCancellation::default())?;
    let plan = preview.plan(ImportOptions {
        mode: ImportMode::Replace,
        import_display_preferences: true,
        import_security_preferences: true,
        ..Default::default()
    })?;
    assert_eq!(plan.summary().incoming.accounts, 25);
    plan.apply(&mut dest, &OperationCancellation::default())?;
    let actual = dest.vault()?;
    assert_eq!(actual.id, local_id);
    assert_eq!(actual.preferences.id, pref_id);
    assert!(!actual.preferences.auto_start);
    let mut expected = v;
    expected.id = actual.id;
    expected.state = actual.state;
    expected.preferences.id = pref_id;
    expected.preferences.state = actual.preferences.state;
    expected.preferences.auto_start = false;
    assert!(*codec::encode(actual)? == *codec::encode(&expected)?);
    assert_ne!(
        fs::read(source_dir.path().join("vault.bin"))?,
        fs::read(dest_dir.path().join("vault.bin"))?
    );
    dest.lock();
    dest.unlock(Zeroizing::new("independent local passphrase".into()))?;
    assert!(*codec::encode(dest.vault()?)? == *codec::encode(&expected)?);
    assert!(source_dir.path().join("session.lock").exists());
    assert!(!portable.path().join("session.lock").exists());
    Ok(())
}

#[test]
fn stale_source_revision_cancel_and_failed_import_never_mutate() -> TestResult {
    let (_sd, source) = store(sample()?)?;
    let portable = tempfile::tempdir()?;
    let path = portable.path().join("portable.backup");
    make_backup(&source, &path)?;
    let (dd, mut dest) = store(Vault::new(Timestamp(1)))?;
    let original = fs::read(dd.path().join("vault.bin"))?;
    assert_eq!(
        preview(
            &dest,
            &path,
            Zeroizing::new("incorrect disposable passphrase".into()),
            &OperationCancellation::default()
        )
        .err(),
        Some(BackupError::Invalid)
    );
    assert_eq!(fs::read(dd.path().join("vault.bin"))?, original);
    let plan = preview(&dest, &path, pass(), &OperationCancellation::default())?
        .plan(Default::default())?;
    let mut changed = fs::read(&path)?;
    changed[144] ^= 1;
    fs::write(&path, &changed)?;
    assert_eq!(
        plan.apply(&mut dest, &OperationCancellation::default())
            .err(),
        Some(BackupError::Changed)
    );
    assert_eq!(fs::read(dd.path().join("vault.bin"))?, original);
    fs::remove_file(&path)?;
    make_backup(&source, &path)?;
    let plan = preview(&dest, &path, pass(), &OperationCancellation::default())?
        .plan(Default::default())?;
    dest.transaction(|v| v.set_clipboard_timeout(31, Timestamp(2)))?;
    assert_eq!(
        plan.apply(&mut dest, &OperationCancellation::default())
            .err(),
        Some(BackupError::Changed)
    );
    let baseline = fs::read(dd.path().join("vault.bin"))?;
    let plan = preview(&dest, &path, pass(), &OperationCancellation::default())?
        .plan(Default::default())?;
    let c = OperationCancellation::default();
    c.cancel();
    assert_eq!(
        plan.apply(&mut dest, &c).err(),
        Some(BackupError::Cancelled)
    );
    let plan = preview(&dest, &path, pass(), &OperationCancellation::default())?
        .plan(Default::default())?;
    dest.fail_next_backup_write_for_test(true);
    assert_eq!(
        plan.apply(&mut dest, &OperationCancellation::default())
            .err(),
        Some(BackupError::Unavailable)
    );
    assert_eq!(fs::read(dd.path().join("vault.bin"))?, baseline);
    assert!(dest.vault()?.accounts.is_empty());
    dest.fail_next_backup_write_for_test(false);
    Ok(())
}

#[test]
fn destination_consent_faults_and_cancellation_preserve_prior_ciphertext() -> TestResult {
    let d = tempfile::tempdir()?;
    let path = d.path().join("portable.backup");
    let bytes = envelope::encrypt(&sample()?, pass().as_bytes(), parameters())?;
    fs::write(&path, &bytes)?;
    assert!(Destination::create_new(&path).is_err());
    let consent = Destination::overwrite_reviewed(&path)?;
    let mut different = bytes.clone();
    different.push(0);
    fs::write(&path, &different)?;
    assert_eq!(
        files::write(
            consent,
            &bytes,
            &OperationCancellation::default(),
            files::Fault::None
        ),
        Err(BackupError::Changed)
    );
    assert_eq!(fs::read(&path)?, different);
    fs::write(&path, &bytes)?;
    for fault in [files::Fault::Partial, files::Fault::BeforeRename] {
        assert!(
            files::write(
                Destination::overwrite_reviewed(&path)?,
                &different,
                &OperationCancellation::default(),
                fault
            )
            .is_err()
        );
        assert_eq!(fs::read(&path)?, bytes);
    }
    let consent = Destination::overwrite_reviewed(&path)?;
    let cancel = OperationCancellation::default();
    cancel.cancel();
    assert_eq!(
        files::write(consent, &different, &cancel, files::Fault::None),
        Err(BackupError::Cancelled)
    );
    assert_eq!(fs::read_dir(d.path())?.count(), 1);
    let collision = d.path().join("pending-deterministic.bin");
    fs::write(&collision, b"unowned disposable file")?;
    assert!(
        files::write_at(
            Destination::overwrite_reviewed(&path)?,
            &different,
            &OperationCancellation::default(),
            files::Fault::None,
            collision.clone()
        )
        .is_err()
    );
    assert_eq!(fs::read(&collision)?, b"unowned disposable file");
    assert_eq!(fs::read(&path)?, bytes);
    Ok(())
}

#[test]
fn admitted_export_and_import_finish_whole_after_cancel() -> TestResult {
    use crate::storage::test_gate::CommitGate;
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };
    let (_sd, source) = store(sample()?)?;
    let d = tempfile::tempdir()?;
    let path = d.path().join("portable.backup");
    make_backup(&source, &path)?;
    let (dd, mut dest) = store(Vault::new(Timestamp(1)))?;
    let plan = preview(&dest, &path, pass(), &OperationCancellation::default())?
        .plan(Default::default())?;
    let gate = Arc::new(CommitGate::default());
    let cancel = OperationCancellation::default().with_test_gate(Some(gate.clone()));
    std::thread::scope(|scope| -> TestResult {
        let _release = gate.release_on_drop();
        let worker = scope.spawn(|| plan.apply(&mut dest, &cancel));
        assert!(gate.wait_entered(Duration::from_secs(10)));
        let now = Instant::now();
        cancel.cancel();
        assert!(now.elapsed() < Duration::from_millis(100));
        gate.release();
        assert!(worker.join().map_err(|_| "worker failed")?.is_ok());
        Ok(())
    })?;
    dest.lock();
    dest.unlock(Zeroizing::new("independent local passphrase".into()))?;
    assert_eq!(dest.vault()?.accounts.len(), 1);
    let original = fs::read(dd.path().join("vault.bin"))?;
    assert!(!original.is_empty());
    let replacement =
        envelope::encrypt(&Vault::new(Timestamp(1)), pass().as_bytes(), parameters())?;
    let gate = Arc::new(CommitGate::default());
    let cancel = OperationCancellation::default().with_test_gate(Some(gate.clone()));
    let destination = Destination::overwrite_reviewed(&path)?;
    std::thread::scope(|scope| -> TestResult {
        let _release = gate.release_on_drop();
        let worker =
            scope.spawn(|| files::write(destination, &replacement, &cancel, files::Fault::None));
        assert!(gate.wait_entered(Duration::from_secs(10)));
        cancel.cancel();
        gate.release();
        assert!(worker.join().map_err(|_| "worker failed")?.is_ok());
        Ok(())
    })?;
    assert_eq!(fs::read(&path)?, replacement);
    Ok(())
}

#[test]
fn authenticated_schema_counts_trailing_and_global_ids_fail_closed() -> TestResult {
    let v = sample()?;
    let plain = codec::encode(&v)?;
    for data in [Zeroizing::new(vec![255, 255, 255, 255]), {
        let mut p = plain.clone();
        p.push(0);
        p
    }] {
        let b = envelope::encrypt_plain(data, pass().as_bytes(), parameters())?;
        assert!(envelope::decrypt(&b, pass().as_bytes()).is_err());
    }
    let mut bad = v.clone();
    bad.folders[0].id = FolderId::from_uuid(bad.accounts[0].id.as_uuid())?;
    assert_eq!(merge::unique_ids(&bad), Err(BackupError::Invalid));
    let mut oversized = plain.clone();
    // Counts checked before allocation by the shared codec: schema/identities,
    // fixed default preferences (empty query + All) occupy 86 bytes.
    let offset = 4 + 33 + 33 + 3 + 4 + 4 + 4 + 1;
    oversized[offset..offset + 4].copy_from_slice(&10_001u32.to_le_bytes());
    assert!(codec::decode(&oversized).is_err());
    let mut invalid = v.clone();
    invalid.memberships[0].account_id = AccountId::new();
    assert!(codec::encode(&invalid).is_err());
    Ok(())
}

#[test]
fn replace_merge_cross_type_duplicates_folder_filter_and_logical_time() -> TestResult {
    let mut local = sample()?;
    local.preferences.folder = FolderFilter::Folder(local.folders[0].id);
    let mut incoming = sample()?;
    incoming.accounts[0].id = AccountId::from_uuid(local.preferences.id.as_uuid())?;
    incoming.memberships[0].account_id = incoming.accounts[0].id;
    incoming.backup_sets[0].account_id = incoming.accounts[0].id;
    incoming.state.updated_at = Timestamp(9_999);
    let conflicts = merge::conflicts(&local, &incoming)?;
    assert!(conflicts.iter().any(|c| c.global_id_collisions > 0));
    let (both, summary) = merge::build(
        &local,
        &incoming,
        &conflicts,
        ImportOptions {
            account_choices: vec![(0, MergeChoice::KeepBoth)],
            ..Default::default()
        },
    )?;
    assert_eq!(summary.after.used_codes, 2);
    assert!(summary.remapped_ids > 0);
    both.validate()?;
    assert_eq!(both.accounts[0].id, local.accounts[0].id);
    assert_eq!(both.preferences.folder, local.preferences.folder);
    let (mut replaced, s) = merge::build(
        &local,
        &incoming,
        &conflicts,
        ImportOptions {
            mode: ImportMode::Replace,
            ..Default::default()
        },
    )?;
    assert_eq!(s.after.accounts, 1);
    assert_eq!(s.removed_local_records.accounts, 1);
    assert!(s.selected_folder_reset);
    assert_eq!(replaced.preferences.folder, FolderFilter::All);
    assert_eq!(replaced.state.updated_at, Timestamp(9_999));
    assert_eq!(replaced.accounts[0].state, incoming.accounts[0].state);
    merge::unique_ids(&replaced)?;
    assert_eq!(s.folder_copies.len(), 1);
    assert!(s.folder_copies[0].local_folder_name_collision);
    let existing = replaced.accounts[0].id;
    let credential = replaced.accounts[0].credential.clone();
    replaced.edit_account(existing, credential, Timestamp(1))?;
    let added = replaced.add_account(local.accounts[0].credential.clone(), Timestamp(1))?;
    let folder = replaced.add_folder("Logical time", FolderColor::Blue, Timestamp(1))?;
    replaced.add_membership(folder, added, Timestamp(1))?;
    replaced.add_backup_set(
        added,
        vec![BackupCode::new("logical-fixture", Timestamp(1))?],
        Timestamp(1),
    )?;
    replaced.set_lock_timeout(300, Timestamp(1))?;
    replaced.validate()?;
    assert_eq!(
        replaced.accounts[0].state.created_at,
        incoming.accounts[0].state.created_at
    );
    assert_eq!(replaced.accounts[0].state.updated_at, Timestamp(9_999));
    let mut duplicate = incoming.clone();
    let mut second = incoming.accounts[0].clone();
    second.id = AccountId::new();
    duplicate.accounts.push(second);
    let c = merge::conflicts(&Vault::new(Timestamp(1)), &duplicate)?;
    assert!(c.iter().any(|c| c.other_incoming_account == Some(0)));
    assert!(
        merge::build(
            &local,
            &incoming,
            &conflicts,
            ImportOptions {
                account_choices: vec![(99, MergeChoice::KeepBoth)],
                ..Default::default()
            }
        )
        .is_err()
    );
    Ok(())
}

#[test]
#[ignore = "subprocess helper invoked by interruption parent"]
fn portable_interruption_helper() -> TestResult {
    let path = std::env::var_os("AUTHENTICATOR_DISPOSABLE_BACKUP").ok_or("missing test path")?;
    let bytes = envelope::encrypt(&sample()?, pass().as_bytes(), parameters())?;
    files::write(
        Destination::overwrite_reviewed(Path::new(&path))?,
        &bytes,
        &OperationCancellation::default(),
        files::Fault::Terminate,
    )?;
    Err("interruption did not terminate".into())
}

#[test]
fn interrupted_process_leaves_only_ciphertext_and_preserves_backup() -> TestResult {
    let d = tempfile::tempdir()?;
    let path = d.path().join("portable.backup");
    let old = envelope::encrypt(&Vault::new(Timestamp(1)), pass().as_bytes(), parameters())?;
    fs::write(&path, &old)?;
    let output = std::process::Command::new(std::env::current_exe()?)
        .args([
            "--exact",
            "backup::tests::portable_interruption_helper",
            "--ignored",
            "--nocapture",
        ])
        .env("AUTHENTICATOR_DISPOSABLE_BACKUP", &path)
        .output()?;
    assert_eq!(output.status.code(), Some(74));
    assert_eq!(fs::read(&path)?, old);
    let files: Vec<_> = fs::read_dir(d.path())?.collect::<std::result::Result<_, _>>()?;
    assert_eq!(files.len(), 2);
    for file in files {
        let b = fs::read(file.path())?;
        assert!(b.starts_with(envelope::MAGIC));
        assert!(
            !b.windows(b"public-fixture-code".len())
                .any(|w| w == b"public-fixture-code")
        );
        envelope::decrypt(&b, pass().as_bytes())?;
    }
    Ok(())
}

#[test]
fn unsafe_endpoints_oversized_files_and_processing_limits_are_rejected() -> TestResult {
    let d = tempfile::tempdir()?;
    assert!(Destination::create_new(d.path()).is_err());
    assert!(Destination::create_new(Path::new("relative.backup")).is_err());
    let path = d.path().join("large.backup");
    let f = fs::File::create(&path)?;
    f.set_len((envelope::MAX_FILE + 1) as u64)?;
    assert_eq!(files::source(&path).err(), Some(BackupError::Limit));
    #[cfg(windows)]
    for name in ["NUL.backup", "COM1.backup", "file.backup:stream", "name."] {
        assert!(Destination::create_new(&d.path().join(name)).is_err());
    }
    let mut a = sample()?;
    let mut b = sample()?;
    a.accounts = vec![a.accounts[0].clone(); Vault::MAX_RECORDS];
    b.accounts = vec![b.accounts[0].clone(); Vault::MAX_RECORDS];
    for account in a
        .accounts
        .iter_mut()
        .skip(1)
        .chain(b.accounts.iter_mut().skip(1))
    {
        account.id = AccountId::new();
    }
    a.validate()?;
    b.validate()?;
    let dense = merge::conflicts(&a, &b)?;
    assert_eq!(dense.len(), Vault::MAX_RECORDS + 1);
    assert_eq!(dense[0].local_match_count, Vault::MAX_RECORDS);
    assert_eq!(dense[0].incoming_match_count, 0);
    assert_eq!(
        dense[Vault::MAX_RECORDS - 1].incoming_match_count,
        Vault::MAX_RECORDS - 1
    );
    let (_, dense_summary) = merge::build(
        &a,
        &b,
        &dense,
        ImportOptions {
            mode: ImportMode::Replace,
            ..Default::default()
        },
    )?;
    assert_eq!(dense_summary.after.accounts, Vault::MAX_RECORDS);
    let mut local = sample()?;
    let incoming = sample()?;
    let template = local.accounts[0].clone();
    while local.accounts.len() < Vault::MAX_RECORDS {
        let mut x = template.clone();
        x.id = AccountId::new();
        local.accounts.push(x);
    }
    local.validate()?;
    // No conflict scan is needed to prove the final aggregate cap is enforced.
    assert!(merge::build(&local, &incoming, &[], ImportOptions::default()).is_err());
    Ok(())
}

#[test]
fn maximum_unique_snapshot_uses_bounded_index_and_roundtrips_codec() -> TestResult {
    let mut incoming = Vault::new(Timestamp(1));
    for i in 0..Vault::MAX_RECORDS {
        let credential = AccountDraft::new(
            &format!("public-{i}"),
            None,
            None,
            Secret::from_bytes(Zeroizing::new((i as u64).to_le_bytes().to_vec()))?,
            OtpSettings::default(),
        )?;
        incoming.add_account(credential, Timestamp(2))?;
    }
    let local = Vault::new(Timestamp(1));
    let conflicts = merge::conflicts(&local, &incoming)?;
    assert!(conflicts.is_empty());
    let (candidate, s) = merge::build(&local, &incoming, &conflicts, Default::default())?;
    assert_eq!(s.after.accounts, Vault::MAX_RECORDS);
    let encoded = codec::encode(&candidate)?;
    let decoded = codec::decode(&encoded)?;
    assert_eq!(decoded.accounts.len(), Vault::MAX_RECORDS);
    Ok(())
}

#[test]
fn repeated_cross_type_folder_collision_reuses_exact_content_unless_copy_is_explicit() -> TestResult
{
    let local = sample()?;
    let mut incoming = sample()?;
    let colliding = FolderId::from_uuid(local.accounts[0].id.as_uuid())?;
    incoming.folders[0].id = colliding;
    incoming.folders[0].name = "Collision import folder".into();
    incoming.memberships[0].folder_id = colliding;
    incoming.preferences.folder = FolderFilter::Folder(colliding);
    local.validate()?;
    incoming.validate()?;
    merge::unique_ids(&incoming)?;
    let conflicts = merge::conflicts(&local, &incoming)?;
    let (first, first_summary) =
        merge::build(&local, &incoming, &conflicts, ImportOptions::default())?;
    assert_eq!(first.folders.len(), 2);
    assert_eq!(first_summary.skipped_account_bundles, 1);
    assert!(first_summary.folder_copies[0].copied);
    assert!(first_summary.folder_copies[0].local_global_id_collision);
    let imported_id = first_summary.folder_copies[0].resulting_id;
    assert_ne!(imported_id, colliding);

    let conflicts = merge::conflicts(&first, &incoming)?;
    let (second, second_summary) =
        merge::build(&first, &incoming, &conflicts, ImportOptions::default())?;
    assert_eq!(*codec::encode(&first)?, *codec::encode(&second)?);
    assert_eq!(second.folders.len(), 2);
    assert_eq!(second_summary.folder_copies[0].resulting_id, imported_id);
    assert!(second_summary.folder_copies[0].matching_content_reused);
    assert!(!second_summary.folder_copies[0].copied);

    // An explicitly added account's memberships follow the reused folder too.
    let (linked, _) = merge::build(
        &first,
        &incoming,
        &conflicts,
        ImportOptions {
            account_choices: vec![(0, MergeChoice::KeepBoth)],
            ..Default::default()
        },
    )?;
    assert_eq!(linked.folders.len(), 2);
    assert_eq!(
        linked.memberships.last().ok_or("membership")?.folder_id,
        imported_id
    );
    assert_eq!(linked.backup_sets.len(), first.backup_sets.len() + 1);
    assert_eq!(
        linked.backup_sets.last().ok_or("recovery")?.codes[0].used_at,
        Some(Timestamp(6))
    );

    // KeepBoth remains deliberate, including the first cross-type collision.
    let (_, explicit_first) = merge::build(
        &local,
        &incoming,
        &merge::conflicts(&local, &incoming)?,
        ImportOptions {
            folder_choices: vec![(0, MergeChoice::KeepBoth)],
            ..Default::default()
        },
    )?;
    assert!(explicit_first.folder_copies[0].copied);
    assert!(matches!(
        merge::build(
            &local,
            &incoming,
            &merge::conflicts(&local, &incoming)?,
            ImportOptions {
                folder_choices: vec![(0, MergeChoice::KeepLocal)],
                ..Default::default()
            }
        ),
        Err(BackupError::InvalidDecision)
    ));
    let (copied, copied_summary) = merge::build(
        &first,
        &incoming,
        &conflicts,
        ImportOptions {
            folder_choices: vec![(0, MergeChoice::KeepBoth)],
            ..Default::default()
        },
    )?;
    assert_eq!(copied.folders.len(), 3);
    assert!(copied_summary.folder_copies[0].copied);
    assert!(!copied_summary.folder_copies[0].matching_content_reused);
    let expected = imported_id.min(copied_summary.folder_copies[0].resulting_id);
    let (repeated_copies, summary) = merge::build(
        &copied,
        &incoming,
        &merge::conflicts(&copied, &incoming)?,
        ImportOptions::default(),
    )?;
    assert_eq!(*codec::encode(&copied)?, *codec::encode(&repeated_copies)?);
    assert_eq!(summary.folder_copies[0].resulting_id, expected);

    // Equal names alone do not trigger this automatic collision recovery.
    incoming.folders[0].color = FolderColor::Blue;
    let (different, summary) = merge::build(
        &first,
        &incoming,
        &merge::conflicts(&first, &incoming)?,
        ImportOptions::default(),
    )?;
    assert_eq!(different.folders.len(), 3);
    assert!(summary.folder_copies[0].copied);
    Ok(())
}

#[test]
fn folder_reuse_copy_name_choice_and_trash_resolutions_are_explicit() -> TestResult {
    let mut incoming = sample()?;
    incoming.preferences.folder = FolderFilter::Folder(incoming.folders[0].id);
    let mut local = incoming.clone();
    local.preferences.folder = FolderFilter::All;
    local.folders[0].state.updated_at = Timestamp(8);
    local.folders[0].state.deletion = DeletionState::Trashed { at: Timestamp(8) };
    let conflicts = merge::conflicts(&local, &incoming)?;
    let (reused, s) = merge::build(
        &local,
        &incoming,
        &conflicts,
        ImportOptions {
            import_display_preferences: true,
            ..Default::default()
        },
    )?;
    assert_eq!(reused.folders.len(), 1);
    assert!(!s.folder_copies[0].copied);
    assert!(!s.folder_copies[0].in_trash);
    assert!(s.folder_copies[0].resulting_in_trash);
    assert!(s.selected_folder_reset);
    let (copied, s) = merge::build(
        &local,
        &incoming,
        &conflicts,
        ImportOptions {
            folder_choices: vec![(0, MergeChoice::KeepBoth)],
            import_display_preferences: true,
            ..Default::default()
        },
    )?;
    assert_eq!(copied.folders.len(), 2);
    assert!(s.folder_copies[0].copied);
    assert!(!s.selected_folder_reset);
    assert_eq!(
        copied.preferences.folder,
        FolderFilter::Folder(copied.folders[1].id)
    );
    let local = sample()?;
    let mut incoming = sample()?;
    incoming.accounts[0].credential = AccountDraft::new(
        "Other public identity",
        None,
        None,
        Secret::from_base32("M4")?,
        OtpSettings::default(),
    )?;
    let conflicts = merge::conflicts(&local, &incoming)?;
    let (named, s) = merge::build(
        &local,
        &incoming,
        &conflicts,
        ImportOptions {
            folder_choices: vec![(0, MergeChoice::KeepLocal)],
            ..Default::default()
        },
    )?;
    assert_eq!(named.folders.len(), 1);
    assert_eq!(named.accounts.len(), 2);
    assert!(!s.folder_copies[0].copied);
    assert_eq!(named.memberships[1].folder_id, local.folders[0].id);
    named.validate()?;
    Ok(())
}
