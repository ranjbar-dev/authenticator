//! Disposable end-to-end form/service/worker/encrypted-repository regressions.
use super::{import_review::ImportReview, state::AppState};
use crate::{
    domain::{
        clock::Clock,
        otp,
        records::{AccountId, FolderColor, Timestamp, Vault},
    },
    error::AppError,
    platform::lifecycle::Lifecycle,
    storage::VaultStore,
};
use std::time::{Duration, Instant};
use zeroize::Zeroizing;
type TestResult = Result<(), Box<dyn std::error::Error>>;
struct Fixed;
impl Clock for Fixed {
    fn unix_seconds(&self) -> Result<u64, AppError> {
        Ok(1_794_500_000)
    }
}
fn finish(state: &mut AppState) -> Result<(), Box<dyn std::error::Error>> {
    let deadline = Instant::now() + Duration::from_secs(45);
    let mut result = None;
    while state.busy() {
        if Instant::now() >= deadline {
            return Err("account worker timeout".into());
        }
        result = state.poll(Instant::now()).or(result);
        std::thread::sleep(Duration::from_millis(5));
    }
    result.ok_or("missing account result")??;
    Ok(())
}
fn set(value: &mut String, text: &str) {
    value.clear();
    value.push_str(text);
}
fn manual(index: usize) -> ImportReview {
    let mut form = ImportReview::manual();
    set(&mut form.editors[0].service, &format!("Disposable {index}"));
    set(&mut form.editors[0].account, "fixture");
    set(&mut form.editors[0].website, "https://example.invalid");
    set(
        &mut form.editors[0].secret,
        "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
    );
    form
}
fn combinations() -> Vec<(&'static str, &'static str, &'static str)> {
    ["SHA1", "SHA256", "SHA512"]
        .into_iter()
        .flat_map(|a| {
            ["6", "8"].into_iter().flat_map(move |d| {
                ["15", "30", "60", "120"]
                    .into_iter()
                    .map(move |p| (a, d, p))
            })
        })
        .collect()
}
fn settings(form: &mut ImportReview, combo: (&str, &str, &str)) {
    set(&mut form.editors[0].algorithm, combo.0);
    set(&mut form.editors[0].digits, combo.1);
    set(&mut form.editors[0].period, combo.2);
}
fn assert_settings(actual: otp::OtpSettings, combo: (&str, &str, &str)) -> Result<(), AppError> {
    assert_eq!(actual.algorithm, otp::Algorithm::parse(combo.0)?);
    assert_eq!(actual.digits, otp::Digits::parse(combo.1)?);
    assert_eq!(actual.period, otp::Period::parse(combo.2)?);
    Ok(())
}
fn pass() -> Zeroizing<String> {
    Zeroizing::new("disposable-account-test-only".into())
}
fn state_in(dir: &std::path::Path, create: bool) -> Result<AppState, AppError> {
    let mut store = VaultStore::open_directory(dir)?;
    if create {
        store.create(pass(), Vault::new(Timestamp(1)))?;
    }
    let mut state = AppState::new(Ok(store));
    state.repository_mut()?.unlock(pass())?;
    Ok(state)
}
#[test]
fn account_all24_create_and_edit_settings_survive_real_repository_reopen() -> TestResult {
    let dir = tempfile::tempdir()?;
    let lifecycle = Lifecycle::start(|| {})?;
    let mut state = state_in(dir.path(), true)?;
    let mut ids = Vec::<AccountId>::new();
    let mut codes = Vec::new();
    let combos = combinations();
    for (i, combo) in combos.iter().enumerate() {
        let mut form = manual(i);
        settings(&mut form, *combo);
        // The real form service and cancellable worker each validate these.
        let draft = form.drafts(state.vault()?)?.remove(0);
        codes.push(otp::current_code(draft.secret(), draft.settings(), &Fixed)?);
        if i == 0 {
            form.backup_enabled = true;
            set(
                &mut form.backup_text,
                "\u{feff} code-one\r\n\tcode-two\t\ncode-one\n",
            );
            assert!(matches!(
                form.save_intent(state.vault()?),
                Err(AppError::ReviewBackupDuplicates)
            ));
            form.acknowledge_backup_duplicates(true);
        }
        state.begin_save(form.save_intent(state.vault()?)?, lifecycle.watch(), || {})?;
        finish(&mut state)?;
        ids.push(state.take_imported().ok_or("missing account identity")?[0]);
    }
    assert_eq!(state.vault()?.accounts().len(), 24);
    let at = Timestamp(crate::domain::clock::SystemClock.unix_seconds()?);
    state.repository_mut()?.transaction(|v| {
        let folder = v.add_folder("Disposable folder", FolderColor::Blue, at)?;
        v.add_membership(folder, ids[0], at)?;
        Ok(())
    })?;
    drop(state);
    let mut state = state_in(dir.path(), false)?;
    for (i, account) in state.vault()?.accounts().iter().enumerate() {
        assert_eq!(account.id(), ids[i]);
        assert_settings(account.credential().settings(), combos[i])?;
        assert!(
            otp::current_code(
                account.credential().secret(),
                account.credential().settings(),
                &Fixed
            )?
            .expose_value()
                == codes[i].expose_value()
        );
    }
    let created = state.vault()?.accounts()[0].state().created_at;
    let set_id = state.vault()?.backup_sets()[0].id();
    let code_ids = state.vault()?.backup_sets()[0]
        .codes()
        .iter()
        .map(|c| c.id())
        .collect::<Vec<_>>();
    for (i, id) in ids.iter().enumerate() {
        let account = state
            .vault()?
            .accounts()
            .iter()
            .find(|a| a.id() == *id)
            .ok_or("account absent")?;
        let mut form = ImportReview::edit(account)?;
        assert!(form.editors[0].secret.is_empty());
        assert!(!form.editors[0].replace_secret);
        set(&mut form.editors[0].service, &format!("Edited {i}"));
        settings(&mut form, combos[(i + 1) % 24]);
        state.begin_save(form.save_intent(state.vault()?)?, lifecycle.watch(), || {})?;
        finish(&mut state)?;
    }
    drop(state);
    let state = state_in(dir.path(), false)?;
    for (i, account) in state.vault()?.accounts().iter().enumerate() {
        assert_eq!(account.id(), ids[i]);
        assert_eq!(account.credential().service(), format!("Edited {i}"));
        assert_settings(account.credential().settings(), combos[(i + 1) % 24])?;
        let code = otp::current_code(
            account.credential().secret(),
            account.credential().settings(),
            &Fixed,
        )?;
        assert!(code.expose_value() == codes[(i + 1) % 24].expose_value());
    }
    assert_eq!(state.vault()?.accounts()[0].state().created_at, created);
    assert_eq!(state.vault()?.memberships().len(), 1);
    let set = &state.vault()?.backup_sets()[0];
    assert_eq!(set.id(), set_id);
    assert_eq!(
        set.codes().iter().map(|c| c.id()).collect::<Vec<_>>(),
        code_ids
    );
    assert!(set.codes().iter().all(|c| c.used_at().is_none()));
    assert_eq!(set.codes().len(), 2);
    Ok(())
}

#[test]
fn account_validation_cancel_duplicate_stale_and_explicit_secret_replacement() -> TestResult {
    let mut vault = Vault::new(Timestamp(1));
    let first = manual(0);
    let id = first.save_intent(&vault)?.apply(&mut vault, Timestamp(2))?[0];
    let original = vault.accounts()[0].credential().clone();
    let folder = vault.add_folder("Associated fixture", FolderColor::Green, Timestamp(3))?;
    vault.add_membership(folder, id, Timestamp(3))?;
    let mut used = crate::domain::records::BackupCode::new("used-fixture", Timestamp(3))?;
    used.set_used(true, Timestamp(3))?;
    let used_id = used.id();
    let recovery = vault.add_backup_set(id, vec![used], Timestamp(3))?;
    let form = ImportReview::edit(&vault.accounts()[0])?;
    assert!(
        vault
            .duplicates(&form.drafts(&vault)?[0], form.mode.exclude())
            .is_empty()
    );
    drop(form); // Cancel/drop never reaches mutation.
    assert!(
        vault.accounts()[0]
            .credential()
            .secret()
            .same_seed(original.secret())
    );
    let mut form = ImportReview::edit(&vault.accounts()[0])?;
    form.editors[0].replace_secret = true;
    set(&mut form.editors[0].secret, "not Base32!");
    assert!(matches!(
        form.save_intent(&vault),
        Err(AppError::InvalidSecret)
    ));
    assert_eq!(&**form.editors[0].secret, "not Base32!");
    set(&mut form.editors[0].secret, "M4");
    form.save_intent(&vault)?.apply(&mut vault, Timestamp(3))?;
    assert!(
        !vault.accounts()[0]
            .credential()
            .secret()
            .same_seed(original.secret())
    );
    assert_eq!(vault.accounts()[0].id(), id);
    assert_eq!(vault.memberships()[0].account_id(), id);
    assert_eq!(vault.backup_sets()[0].id(), recovery);
    assert_eq!(vault.backup_sets()[0].codes()[0].id(), used_id);
    assert_eq!(
        vault.backup_sets()[0].codes()[0].used_at(),
        Some(Timestamp(3))
    );
    // Same-second edit is detected from the original ephemeral snapshot, even
    // when timestamps alone cannot distinguish the intervening change.
    let stale = ImportReview::edit(&vault.accounts()[0])?;
    let intent = stale.save_intent(&vault)?;
    let mut change = manual(99);
    set(&mut change.editors[0].service, "Concurrent change");
    vault.edit_account(id, change.approved()?.remove(0), Timestamp(3))?;
    assert_eq!(
        intent.apply(&mut vault, Timestamp(4)).err(),
        Some(AppError::StaleAccount)
    );
    assert_eq!(
        vault.accounts()[0].credential().service(),
        "Concurrent change"
    );
    let other = first.save_intent(&vault)?.apply(&mut vault, Timestamp(4))?[0];
    assert_ne!(id, other);
    assert_eq!(vault.accounts().len(), 2);
    let mut missing = ImportReview::edit(&vault.accounts()[0])?.save_intent(&vault)?;
    if let super::state::SaveIntent::Edit { id, .. } = &mut missing {
        *id = AccountId::new();
    }
    assert_eq!(
        missing.apply(&mut vault, Timestamp(5)).err(),
        Some(AppError::StaleAccount)
    );
    let trashed = ImportReview::edit(&vault.accounts()[0])?.save_intent(&vault)?;
    vault.accounts[0].state.deletion =
        crate::domain::records::DeletionState::Trashed { at: Timestamp(4) };
    assert!(matches!(
        ImportReview::edit(&vault.accounts()[0]),
        Err(AppError::StaleAccount)
    ));
    assert_eq!(
        trashed.apply(&mut vault, Timestamp(5)).err(),
        Some(AppError::StaleAccount)
    );
    for (field, invalid, error) in [
        ("service", "", AppError::MissingService),
        ("algorithm", "SHA3", AppError::InvalidAlgorithm),
        ("digits", "7", AppError::InvalidDigits),
        ("period", "9", AppError::InvalidPeriod),
    ] {
        let mut form = manual(0);
        let e = &mut form.editors[0];
        set(
            match field {
                "service" => &mut e.service,
                "algorithm" => &mut e.algorithm,
                "digits" => &mut e.digits,
                _ => &mut e.period,
            },
            invalid,
        );
        assert!(matches!(form.save_intent(&vault), Err(e) if e == error));
        assert!(!error.to_string().contains("GEZDG"));
    }
    Ok(())
}
