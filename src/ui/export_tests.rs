//! Disposable native form/worker/engine security and data-loss regressions.
use super::*;
use crate::{
    backup::{BackupError, Destination, ImportOptions},
    domain::{
        otp::{OtpSettings, Secret},
        records::{AccountDraft, BackupCode, FolderColor, Timestamp, Vault},
    },
    storage::{OperationCancellation, test_gate::CommitGate},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn pass() -> Zeroizing<String> {
    Zeroizing::new("disposable-export-local-only".into())
}
fn backup_pass() -> Zeroizing<String> {
    Zeroizing::new("disposable-export-backup-only".into())
}
fn fixture() -> Result<Vault, crate::error::AppError> {
    let mut v = Vault::new(Timestamp(1));
    let id = v.add_account(
        AccountDraft::new(
            "Public export sentinel",
            Some("https://example.invalid"),
            Some("public-fixture"),
            Secret::from_base32("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")?,
            OtpSettings::default(),
        )?,
        Timestamp(2),
    )?;
    let folder = v.add_folder("Public export folder", FolderColor::Blue, Timestamp(3))?;
    v.add_membership(folder, id, Timestamp(4))?;
    let mut code = BackupCode::new("public-recovery-sentinel", Timestamp(5))?;
    code.set_used(true, Timestamp(6))?;
    v.add_backup_set(id, vec![code], Timestamp(7))?;
    v.trash_account(id, v.accounts()[0].state(), Timestamp(8))?;
    let mut preferences = v.preferences().codes();
    preferences.query = Zeroizing::new("public preference sentinel".into());
    v.set_codes_preferences(preferences, Timestamp(9))?;
    Ok(v)
}
fn app(directory: &std::path::Path) -> Result<AuthenticatorApp, Box<dyn std::error::Error>> {
    let mut store = VaultStore::open_directory(directory)?;
    store.create(pass(), fixture()?)?;
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut app = AuthenticatorApp::with_context(context, Ok(store), 0);
    app.state.repository_mut()?.unlock(pass())?;
    app.state.navigate(Route::Settings);
    app.export_hub = true;
    app.export_form = Some(ExportForm::new(egui::Id::unique("export-opener")));
    Ok(app)
}
fn finish(
    app: &mut AuthenticatorApp,
) -> Result<Option<Result<(), BackupError>>, Box<dyn std::error::Error>> {
    let deadline = Instant::now() + Duration::from_secs(45);
    while app.state.busy() {
        if Instant::now() >= deadline {
            return Err("export worker timeout".into());
        }
        app.state.poll(Instant::now());
        std::thread::sleep(Duration::from_millis(5));
    }
    Ok(app.state.take_exported())
}
fn ready(path: &std::path::Path) -> Result<ExportForm, BackupError> {
    let mut form = ExportForm::new(egui::Id::unique("export-opener"));
    form.path = Some(path.to_owned());
    form.destination = Some(Destination::review(path)?);
    form.passphrase = backup_pass();
    form.confirmation = backup_pass();
    Ok(form)
}
fn render(app: &mut AuthenticatorApp, width: f32) -> egui::FullOutput {
    render_events(app, width, Vec::new())
}
fn render_until(
    app: &mut AuthenticatorApp,
    done: impl Fn(&AuthenticatorApp) -> bool,
) -> TestResult {
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        render(app, 600.);
        if done(app) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("queued export timeout".into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn render_events(
    app: &mut AuthenticatorApp,
    width: f32,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    let ctx = app.context.clone();
    let mut out = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 1500.),
            )),
            events,
            ..Default::default()
        },
        |root| app.show(root),
    );
    out.textures_delta.clear();
    out
}
fn button_point(
    out: &egui::FullOutput,
    label: &str,
) -> Result<egui::Pos2, Box<dyn std::error::Error>> {
    let bounds = out
        .platform_output
        .accesskit_update
        .as_ref()
        .ok_or("missing AX tree")?
        .nodes
        .iter()
        .find(|(_, n)| n.role() == egui::Role::Button && n.label() == Some(label))
        .and_then(|(_, n)| n.bounds())
        .ok_or("missing button bounds")?;
    Ok(egui::pos2(
        ((bounds.x0 + bounds.x1) / 2.) as f32,
        ((bounds.y0 + bounds.y1) / 2.) as f32,
    ))
}
fn click(
    app: &mut AuthenticatorApp,
    label: &str,
) -> Result<egui::FullOutput, Box<dyn std::error::Error>> {
    let before = render(app, 600.);
    let pos = button_point(&before, label)?;
    render_events(
        app,
        600.,
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    render_events(
        app,
        600.,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    // Egui's first modal pass measures its Area; inspect the visible next pass.
    Ok(render(app, 600.))
}
fn key(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}
fn ax(out: &egui::FullOutput) -> Vec<String> {
    out.platform_output
        .accesskit_update
        .as_ref()
        .map(|tree| {
            tree.nodes
                .iter()
                .flat_map(|(_, n)| {
                    [n.label(), n.value()]
                        .into_iter()
                        .flatten()
                        .map(str::to_owned)
                })
                .collect()
        })
        .unwrap_or_default()
}
#[test]
fn export_flushes_latest_display_preferences_before_the_backup_snapshot() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut app = app(directory.path())?;
    render(&mut app, 600.);
    let query = "current pending public filter";
    let preferences = app.codes_preferences.as_mut().ok_or("preferences")?;
    preferences.query = Zeroizing::new(query.into());
    preferences.sort = crate::domain::records::SortOrder::AccountDescending;
    preferences.view = crate::domain::records::ViewMode::Grid;
    preferences.folder = crate::domain::records::FolderFilter::Unfiled;
    app.state.query = query.into();
    app.preferences_changed();
    let before = std::fs::read(directory.path().join("vault.bin"))?;
    let destination = directory.path().join("current-preferences.authpbk");
    let mut form = ready(&destination)?;
    app.start_export(&mut form);
    assert!(form.saving_preferences && !form.exporting);
    assert!(!destination.exists());
    app.export_form = Some(form);
    render_until(&mut app, |app| {
        app.export_form.as_ref().is_some_and(|form| form.success)
    })?;
    let preview = crate::backup::preview(
        app.state.repository_mut()?,
        &destination,
        backup_pass(),
        &OperationCancellation::default(),
    )?;
    let saved = preview.incoming_vault();
    assert!(saved.preferences().codes().query.as_str() == query);
    assert_eq!(
        saved.preferences().sort(),
        crate::domain::records::SortOrder::AccountDescending
    );
    assert_eq!(
        saved.preferences().view(),
        crate::domain::records::ViewMode::Grid
    );
    assert_eq!(
        saved.preferences().codes().folder,
        crate::domain::records::FolderFilter::Unfiled
    );
    assert!(
        *crate::storage::codec::encode(saved)?
            == *crate::storage::codec::encode(app.state.vault()?)?
    );
    assert_ne!(before, std::fs::read(directory.path().join("vault.bin"))?);
    assert!(app.preferences_dirty.is_none() && !app.preferences_error);
    let form = app.export_form.as_ref().ok_or("form")?;
    assert!(form.passphrase.is_empty() && form.confirmation.is_empty());
    Ok(())
}

#[test]
fn export_preference_save_failure_blocks_file_write_and_retries_current_intent() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut app = app(directory.path())?;
    render(&mut app, 600.);
    let query = "pending preference retry";
    app.codes_preferences.as_mut().ok_or("preferences")?.query = Zeroizing::new(query.into());
    app.state.query = query.into();
    app.preferences_changed();
    let before = std::fs::read(directory.path().join("vault.bin"))?;
    app.state
        .repository_mut()?
        .fail_next_backup_write_for_test(true);
    let destination = directory.path().join("retry-preferences.authpbk");
    let mut form = ready(&destination)?;
    app.start_export(&mut form);
    app.export_form = Some(form);
    render_until(&mut app, |app| {
        !app.state.busy()
            && app
                .export_form
                .as_ref()
                .is_some_and(|form| !form.saving_preferences && form.error.is_some())
    })?;
    assert!(!destination.exists());
    assert_eq!(before, std::fs::read(directory.path().join("vault.bin"))?);
    assert!(app.preferences_dirty.is_some() && app.preferences_error);
    let form = app.export_form.as_ref().ok_or("form")?;
    assert!(
        form.error
            .is_some_and(|error| error.contains("display changes could not be saved"))
    );
    assert!(form.passphrase.is_empty() && form.confirmation.is_empty());
    assert!(
        app.codes_preferences
            .as_ref()
            .ok_or("preferences")?
            .query
            .as_str()
            == query
    );
    app.state
        .repository_mut()?
        .fail_next_backup_write_for_test(false);
    let mut form = app.export_form.take().ok_or("form")?;
    form.passphrase = backup_pass();
    form.confirmation = backup_pass();
    app.start_export(&mut form);
    app.export_form = Some(form);
    render_until(&mut app, |app| {
        app.export_form.as_ref().is_some_and(|form| form.success)
    })?;
    let preview = crate::backup::preview(
        app.state.repository_mut()?,
        &destination,
        backup_pass(),
        &OperationCancellation::default(),
    )?;
    assert!(
        preview
            .incoming_vault()
            .preferences()
            .codes()
            .query
            .as_str()
            == query
    );
    assert!(app.preferences_dirty.is_none() && !app.preferences_error);
    Ok(())
}

#[test]
fn export_unexpected_repository_lock_drops_queued_form_secrets_and_path() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut app = app(directory.path())?;
    let path = directory.path().join("queued-private-path.authpbk");
    app.export_form = Some(ready(&path)?);
    let before = render(&mut app, 600.);
    assert!(
        ax(&before)
            .iter()
            .any(|value| value.contains("queued-private-path"))
    );
    app.export_form.as_mut().ok_or("form")?.saving_preferences = true;
    app.state.repository_mut()?.lock();
    let out = render(&mut app, 600.);
    assert!(app.export_form.is_none() && app.export_picker.is_none());
    assert!(app.state.query.is_empty() && app.state.backup_query.is_empty());
    assert!(ax(&out).iter().all(
        |value| !value.contains("queued-private-path") && !value.contains("Backup passphrase")
    ));
    assert!(!path.exists());
    Ok(())
}

#[test]
fn export_form_validation_masked_nodes_lock_and_responsive_controls() -> TestResult {
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path())?;
    let mut form = ready(&dir.path().join("public-export.authpbk"))?;
    form.confirmation = Zeroizing::new("mismatched-public-sentinel".into());
    assert!(form.validate().is_err());
    form.confirmation = backup_pass();
    assert!(form.validate().is_ok());
    form.passphrase = Zeroizing::new(" short ".into());
    assert!(form.validate().is_err());
    form.passphrase = Zeroizing::new("x".repeat(1025));
    assert!(form.validate().is_err());
    // Exact spaces/non-ASCII are validated in bytes, never silently trimmed.
    form.passphrase = Zeroizing::new("é".repeat(6));
    form.confirmation = Zeroizing::new("é".repeat(6));
    assert!(form.validate().is_ok());
    form.passphrase = backup_pass();
    form.confirmation = backup_pass();
    app.export_form = Some(form);
    for width in [340., 600., 960.] {
        let out = render(&mut app, width);
        let names = ax(&out);
        assert!(names.iter().any(|n| n == "Backup passphrase"));
        assert!(names.iter().any(|n| n == "Confirm backup passphrase"));
        assert!(names.iter().any(|n| n == "Back to Export & Import"));
        assert!(
            names
                .iter()
                .all(|n| !n.contains("disposable-export-backup-only"))
        );
        let passwords = out
            .platform_output
            .accesskit_update
            .as_ref()
            .ok_or("missing AX")?
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == egui::Role::PasswordInput)
            .count();
        assert_eq!(passwords, 2);
        assert!(
            !out.platform_output
                .commands
                .iter()
                .any(|c| matches!(c, egui::OutputCommand::CopyText(_)))
        );
    }
    app.lock();
    let names = ax(&render(&mut app, 340.));
    assert!(app.export_form.is_none());
    assert!(!app.export_hub);
    assert!(app.export_picker.is_none());
    assert!(
        names
            .iter()
            .all(|n| !n.contains("public-export") && !n.contains("Backup passphrase"))
    );
    Ok(())
}
#[test]
fn export_ui_worker_roundtrip_full_vault_no_plaintext_and_no_double_submit() -> TestResult {
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path())?;
    let expected = crate::storage::codec::encode(app.state.vault()?)?;
    let local = std::fs::read(dir.path().join("vault.bin"))?;
    let path = dir.path().join("public-export.authpbk");
    let mut form = ready(&path)?;
    app.start_export(&mut form);
    assert!(form.exporting);
    assert!(form.passphrase.is_empty());
    assert!(form.confirmation.is_empty());
    app.start_export(&mut form); // repeated activation cannot start another worker
    assert!(app.state.busy());
    assert_eq!(
        app.state.begin_export(
            Destination::create_new(&dir.path().join("second.authpbk"))?,
            backup_pass(),
            app.lifecycle.as_ref().map_err(|e| *e)?.watch(),
            || {}
        ),
        Err(crate::error::AppError::VaultBusy)
    );
    assert_eq!(finish(&mut app)?, Some(Ok(())));
    form.complete(Ok(()), &app.context);
    assert!(form.success);
    assert_eq!(std::fs::read(dir.path().join("vault.bin"))?, local);
    assert!(!dir.path().join("second.authpbk").exists());
    let bytes = std::fs::read(&path)?;
    for sentinel in [
        "Public export sentinel",
        "public-recovery-sentinel",
        "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
        "public preference sentinel",
        "disposable-export-backup-only",
    ] {
        assert!(
            !bytes
                .windows(sentinel.len())
                .any(|s| s == sentinel.as_bytes())
        );
    }
    let other = tempfile::tempdir()?;
    let mut restored = VaultStore::open_directory(other.path())?;
    restored.create(
        Zeroizing::new("independent-restored-vault".into()),
        Vault::new(Timestamp(1)),
    )?;
    let preview = crate::backup::preview(
        &restored,
        &path,
        backup_pass(),
        &OperationCancellation::default(),
    )?;
    assert_eq!(
        crate::storage::codec::encode(preview.incoming_vault())?.as_slice(),
        expected.as_slice()
    );
    let options = ImportOptions {
        mode: crate::backup::ImportMode::Replace,
        ..Default::default()
    };
    preview
        .plan(options)?
        .apply(&mut restored, &OperationCancellation::default())?;
    assert_eq!(restored.vault()?.accounts().len(), 1);
    assert!(
        restored.vault()?.backup_sets()[0].codes()[0]
            .used_at()
            .is_some()
    );
    Ok(())
}
#[test]
fn export_destination_review_is_nonconsenting_changed_and_unavailable_preserve_files() -> TestResult
{
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path())?;
    let path = dir.path().join("existing.authpbk");
    std::fs::write(&path, b"public-existing-file")?;
    let local = std::fs::read(dir.path().join("vault.bin"))?;
    let mut form = ready(&path)?;
    assert!(
        form.destination
            .as_ref()
            .is_some_and(|d| d.needs_overwrite())
    );
    // Merely reviewing (or cancelling/dropping a form) performs no write.
    assert_eq!(std::fs::read(&path)?, b"public-existing-file");
    form.confirmation = Zeroizing::new("different-confirmation".into());
    app.start_export(&mut form);
    assert!(!app.state.busy());
    assert_eq!(std::fs::read(&path)?, b"public-existing-file");
    form.confirmation = backup_pass();
    std::fs::write(&path, b"public-newer-file")?;
    app.start_export(&mut form);
    assert_eq!(finish(&mut app)?, Some(Err(BackupError::Changed)));
    assert_eq!(std::fs::read(&path)?, b"public-newer-file");
    assert_eq!(std::fs::read(dir.path().join("vault.bin"))?, local);
    assert!(matches!(
        Destination::review(&dir.path().join("missing-folder").join("out.authpbk")),
        Err(BackupError::Unavailable)
    ));
    form.complete(Err(BackupError::Unavailable), &app.context);
    assert!(form.destination.is_none());
    assert!(
        form.error
            .is_some_and(|e| e.contains("permissions") && e.contains("disk space"))
    );
    assert!(form.passphrase.is_empty());
    assert!(form.confirmation.is_empty());
    Ok(())
}
#[test]
fn export_lock_before_and_after_admission_never_publishes_late_path_or_session() -> TestResult {
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path())?;
    let path = dir.path().join("cancel-before.authpbk");
    let mut form = ready(&path)?;
    app.start_export(&mut form);
    app.export_form = Some(form);
    app.lock();
    assert_eq!(finish(&mut app)?, None);
    assert!(!path.exists());
    assert!(!app.state.is_unlocked());
    app.state.repository_mut()?.unlock(pass())?;
    let path = dir.path().join("cancel-admitted.authpbk");
    let mut form = ready(&path)?;
    let gate = Arc::new(CommitGate::default());
    let _release = gate.release_on_drop();
    app.state.hold_next_save_after_admission(gate.clone());
    app.start_export(&mut form);
    app.export_form = Some(form);
    assert!(gate.wait_entered(Duration::from_secs(15)));
    let started = Instant::now();
    app.lock();
    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(app.export_form.is_none());
    assert!(app.state.busy());
    let names = ax(&render(&mut app, 340.));
    assert!(
        names
            .iter()
            .all(|n| !n.contains("cancel-admitted") && !n.contains("Backup passphrase"))
    );
    assert!(!path.exists());
    gate.release();
    assert_eq!(finish(&mut app)?, None);
    assert!(!app.state.is_unlocked());
    assert!(
        path.exists(),
        "an admitted export finishes as a whole without late UI adoption"
    );
    Ok(())
}
#[test]
fn export_synthetic_root_overwrite_cancel_focus_busy_success_and_back() -> TestResult {
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path())?;
    let path = dir.path().join("named-existing.authpbk");
    std::fs::write(&path, b"public-existing-file")?;
    app.export_form = Some(ready(&path)?);
    let modal = click(&mut app, "Export encrypted backup")?;
    assert!(
        app.export_form
            .as_ref()
            .is_some_and(|f| f.overwrite.is_some()),
        "synthetic submit must open the overwrite review"
    );
    let labels = ax(&modal);
    assert!(labels.iter().any(|n| n.contains("named-existing.authpbk")));
    assert!(labels.iter().any(|n| n == "Replace existing backup file?"));
    assert!(!app.state.busy());
    let cancel_point = button_point(&modal, "Keep existing file")?;
    let focused = app
        .context
        .memory(|m| m.focused())
        .ok_or("no initial modal focus")?;
    let focus_bounds = modal
        .platform_output
        .accesskit_update
        .as_ref()
        .ok_or("AX")?
        .nodes
        .iter()
        .find(|(id, _)| id.0 == focused.value())
        .and_then(|(_, n)| n.bounds())
        .ok_or("missing focused bounds")?;
    assert!((f64::from(cancel_point.y) - (focus_bounds.y0 + focus_bounds.y1) / 2.).abs() < 1.);
    render_events(&mut app, 600., vec![key(egui::Key::Escape)]);
    assert!(
        app.export_form
            .as_ref()
            .is_some_and(|f| f.overwrite.is_none())
    );
    assert!(!app.state.busy());
    assert_eq!(std::fs::read(&path)?, b"public-existing-file");
    click(&mut app, "Export encrypted backup")?;
    click(&mut app, "Keep existing file")?;
    assert!(!app.state.busy());
    assert_eq!(std::fs::read(&path)?, b"public-existing-file");
    click(&mut app, "Export encrypted backup")?;
    click(&mut app, "Replace file and export")?;
    assert!(app.state.busy());
    let busy = render(&mut app, 340.);
    let labels = ax(&busy);
    assert!(labels.iter().any(|n| n == "Cancel and lock"));
    assert!(
        !busy
            .platform_output
            .accesskit_update
            .as_ref()
            .ok_or("AX")?
            .nodes
            .iter()
            .any(|(_, n)| n.role() == egui::Role::Button
                && n.label() == Some("Export encrypted backup"))
    );
    assert!(
        !busy
            .platform_output
            .accesskit_update
            .as_ref()
            .ok_or("AX")?
            .nodes
            .iter()
            .any(|(_, n)| n.role() == egui::Role::PasswordInput)
    );
    let result = finish(&mut app)?.ok_or("missing export result")?;
    app.export_form
        .as_mut()
        .ok_or("missing form")?
        .complete(result, &app.context);
    let success = render(&mut app, 600.);
    assert!(
        ax(&success)
            .iter()
            .any(|n| n.contains("Encrypted backup saved"))
    );
    click(&mut app, "Back to Export & Import")?;
    assert!(app.export_form.is_none());
    assert!(app.export_hub);
    assert_ne!(std::fs::read(&path)?, b"public-existing-file");
    Ok(())
}
#[test]
fn export_denied_file_and_private_lifecycle_signal_fail_closed() -> TestResult {
    use std::os::windows::fs::OpenOptionsExt;
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path())?;
    let path = dir.path().join("sharing-denied.authpbk");
    std::fs::write(&path, b"public-preserved-file")?;
    let mut form = ready(&path)?;
    let held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&path)?;
    app.start_export(&mut form);
    assert_eq!(finish(&mut app)?, Some(Err(BackupError::Unavailable)));
    drop(held);
    assert_eq!(std::fs::read(&path)?, b"public-preserved-file");
    let path = dir.path().join("lifecycle-cancelled.authpbk");
    let watch = app.lifecycle.as_ref().map_err(|e| *e)?.watch();
    app.lifecycle
        .as_ref()
        .map_err(|e| *e)?
        .notify_lock_for_test();
    app.state
        .begin_export(Destination::create_new(&path)?, backup_pass(), watch, || {})?;
    assert_eq!(finish(&mut app)?, None);
    assert!(!path.exists());
    assert!(!app.state.session_active());
    Ok(())
}
#[test]
fn export_close_and_idle_clear_forms_before_pending_work() -> TestResult {
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path())?;
    app.export_form = Some(ready(&dir.path().join("close-sentinel.authpbk"))?);
    // Dirty preferences must not defer clearing an open backup passphrase form.
    app.preferences_dirty = Some(Instant::now());
    let ctx = app.context.clone();
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(340., 780.),
        )),
        ..Default::default()
    };
    input
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .events
        .push(egui::ViewportEvent::Close);
    let mut closed = ctx.run_ui(input, |root| app.show(root));
    closed.textures_delta.clear();
    assert!(app.export_form.is_none());
    assert!(!app.state.session_active());
    assert!(
        ax(&closed)
            .iter()
            .all(|n| !n.contains("close-sentinel") && !n.contains("Backup passphrase"))
    );
    app.state.repository_mut()?.unlock(pass())?;
    app.closing = false;
    app.close_locked = false;
    app.exit_approved = false;
    let path = dir.path().join("idle-sentinel.authpbk");
    app.export_form = Some(ready(&path)?);
    app.state.activity(
        Instant::now()
            .checked_sub(Duration::from_secs(600))
            .ok_or("test time")?,
    );
    let mut form = app.export_form.take().ok_or("form")?;
    app.start_export(&mut form);
    app.export_form = Some(form);
    // Same real hidden-logic lifecycle path, without OS lock/suspend automation.
    app.background_logic(&ctx, Instant::now());
    assert!(app.export_form.is_none());
    assert!(!app.state.session_active());
    assert_eq!(finish(&mut app)?, None);
    assert!(!path.exists());
    Ok(())
}
#[test]
fn export_keyboard_submit_focus_survives_validation_correction_and_keep_existing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut app = app(directory.path())?;
    let path = directory.path().join("focus-existing.authpbk");
    std::fs::write(&path, b"public-keep-focus-file")?;
    let mut form = ready(&path)?;
    form.confirmation = Zeroizing::new("different-disposable-confirmation".into());
    app.export_form = Some(form);
    let identity = |out: &egui::FullOutput| -> Result<u64, Box<dyn std::error::Error>> {
        out.platform_output
            .accesskit_update
            .as_ref()
            .and_then(|tree| {
                tree.nodes.iter().find(|(_, n)| {
                    n.role() == egui::Role::Button && n.label() == Some("Export encrypted backup")
                })
            })
            .map(|(id, _)| id.0)
            .ok_or_else(|| "missing submit identity".into())
    };
    let initial = render(&mut app, 600.);
    let submit_id = identity(&initial)?;
    // Initial picker focus -> two password inputs -> Submit.
    for _ in 0..3 {
        render_events(&mut app, 600., vec![key(egui::Key::Tab)]);
    }
    assert_eq!(
        app.context.memory(|m| m.focused()).map(|id| id.value()),
        Some(submit_id)
    );
    render_events(&mut app, 600., vec![key(egui::Key::Enter)]);
    let invalid = render(&mut app, 600.);
    assert!(app.export_form.as_ref().is_some_and(|f| f.error.is_some()));
    assert_eq!(identity(&invalid)?, submit_id);
    assert_eq!(
        app.context.memory(|m| m.focused()).map(|id| id.value()),
        Some(submit_id)
    );
    assert!(!app.state.busy());
    let mut reverse_tab = key(egui::Key::Tab);
    if let egui::Event::Key { modifiers, .. } = &mut reverse_tab {
        *modifiers = egui::Modifiers::SHIFT;
    }
    render_events(&mut app, 600., vec![reverse_tab]);
    let mut select_all = key(egui::Key::A);
    if let egui::Event::Key { modifiers, .. } = &mut select_all {
        *modifiers = egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        };
    }
    render_events(&mut app, 600., vec![select_all]);
    let corrected = render_events(
        &mut app,
        600.,
        vec![egui::Event::Paste("disposable-export-backup-only".into())],
    );
    assert!(
        app.export_form
            .as_ref()
            .is_some_and(|f| f.error.is_none() && f.confirmation.as_str() == f.passphrase.as_str())
    );
    assert_eq!(identity(&corrected)?, submit_id);
    render_events(&mut app, 600., vec![key(egui::Key::Tab)]);
    assert_eq!(
        app.context.memory(|m| m.focused()).map(|id| id.value()),
        Some(submit_id)
    );
    render_events(&mut app, 600., vec![key(egui::Key::Enter)]);
    render(&mut app, 600.); // visible modal measurement pass
    assert!(
        app.export_form
            .as_ref()
            .is_some_and(|f| f.overwrite.is_some())
    );
    // Default Enter chooses Keep existing; shared dialog restores Submit focus.
    render_events(&mut app, 600., vec![key(egui::Key::Enter)]);
    render(&mut app, 600.);
    render(&mut app, 600.);
    assert!(
        app.export_form
            .as_ref()
            .is_some_and(|f| f.overwrite.is_none())
    );
    assert_eq!(
        app.context.memory(|m| m.focused()).map(|id| id.value()),
        Some(submit_id)
    );
    assert!(!app.state.busy());
    assert_eq!(std::fs::read(&path)?, b"public-keep-focus-file");
    Ok(())
}
