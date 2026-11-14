//! Synthetic ROOT events are framework tests, not native Windows pointer proof.
use super::*;
use crate::{
    backup::{ImportMode, MergeChoice},
    domain::{
        otp::{Algorithm, Digits, OtpSettings, Period, Secret},
        records::{AccountDraft, BackupCode, FolderColor, Timestamp, Vault},
    },
    storage::test_gate::CommitGate,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn local_pass() -> Zeroizing<String> {
    Zeroizing::new("disposable-import-destination".into())
}
fn backup_pass() -> Zeroizing<String> {
    Zeroizing::new("disposable-native-backup".into())
}
fn fixture() -> Result<Vault, crate::error::AppError> {
    let mut v = Vault::new(Timestamp(1));
    for algorithm in [Algorithm::Sha1, Algorithm::Sha256, Algorithm::Sha512] {
        for digits in [Digits::Six, Digits::Eight] {
            for period in [
                Period::Seconds15,
                Period::Seconds30,
                Period::Seconds60,
                Period::Seconds120,
            ] {
                v.add_account(
                    AccountDraft::new(
                        &format!(
                            "Public {:?} {} {}",
                            algorithm,
                            digits.count(),
                            period.seconds()
                        ),
                        Some("https://example.invalid"),
                        Some("public-import-fixture"),
                        Secret::from_base32("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")?,
                        OtpSettings {
                            algorithm,
                            digits,
                            period,
                        },
                    )?,
                    Timestamp(2),
                )?;
            }
        }
    }
    let id = v.accounts()[0].id();
    let folder = v.add_folder(
        "Public folder with a deliberately long readable label",
        FolderColor::Purple,
        Timestamp(3),
    )?;
    v.add_membership(folder, id, Timestamp(4))?;
    let mut code = BackupCode::new("public-recovery-only", Timestamp(5))?;
    code.set_used(true, Timestamp(6))?;
    v.add_backup_set(
        id,
        vec![code, BackupCode::new("public-unused-only", Timestamp(6))?],
        Timestamp(7),
    )?;
    v.trash_account(id, v.accounts()[0].state(), Timestamp(8))?;
    v.preferences.auto_start = true;
    v.preferences.lock_timeout_seconds = 900;
    v.preferences.clipboard_timeout_seconds = 120;
    let mut codes = v.preferences().codes();
    codes.query = Zeroizing::new("Public".into());
    codes.view = ViewMode::Grid;
    v.set_codes_preferences(codes, Timestamp(9))?;
    Ok(v)
}
fn app(
    directory: &std::path::Path,
    vault: Vault,
) -> Result<AuthenticatorApp, Box<dyn std::error::Error>> {
    let mut store = VaultStore::open_directory(directory)?;
    store.create(local_pass(), vault)?;
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut app = AuthenticatorApp::with_context(context, Ok(store), 0);
    app.state.repository_mut()?.unlock(local_pass())?;
    app.state.navigate(Route::Settings);
    app.export_hub = true;
    Ok(app)
}
fn render_events(
    app: &mut AuthenticatorApp,
    width: f32,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    let ctx = app.context.clone();
    // Full form geometry for metadata assertions; dialogs use a normal-height
    // viewport. This is explicitly synthetic, never native pixel evidence.
    let height = if app
        .backup_import
        .as_ref()
        .is_some_and(|f| f.replace.is_some())
    {
        900.
    } else {
        24000.
    };
    let mut frame = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, height),
            )),
            events,
            ..Default::default()
        },
        |root| app.show(root),
    );
    frame.textures_delta.clear();
    frame
}
fn render(app: &mut AuthenticatorApp) -> egui::FullOutput {
    render_events(app, 600., Vec::new())
}
fn names(out: &egui::FullOutput) -> Vec<String> {
    out.platform_output
        .accesskit_update
        .as_ref()
        .map(|t| {
            t.nodes
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
fn wait(app: &mut AuthenticatorApp) -> TestResult {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        render(app);
        if !app.state.busy()
            && !app
                .backup_import
                .as_ref()
                .is_some_and(|f| f.saving_preferences)
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("backup UI worker timeout".into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn exported(path: &std::path::Path, vault: Vault) -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut source = app(directory.path(), vault)?;
    let mut form = ExportForm::new(egui::Id::unique("export-source"));
    form.path = Some(path.to_owned());
    form.destination = Some(crate::backup::Destination::review(path)?);
    form.passphrase = backup_pass();
    form.confirmation = backup_pass();
    source.start_export(&mut form);
    source.export_form = Some(form);
    wait(&mut source)?;
    assert!(source.export_form.as_ref().is_some_and(|f| f.success));
    Ok(())
}
fn preview(app: &mut AuthenticatorApp, path: &std::path::Path) -> TestResult {
    let mut form = ImportForm::new(egui::Id::unique("import-opener"));
    form.path = Some(path.to_owned());
    form.passphrase = backup_pass();
    app.start_backup_preview(&mut form);
    app.backup_import = Some(form);
    wait(app)
}
fn plan(app: &mut AuthenticatorApp) -> TestResult {
    let mut form = app.backup_import.take().ok_or("form")?;
    let preview = form.preview.clone().ok_or("preview")?;
    let options = form.plan_options();
    app.start_backup_job(
        &mut form,
        super::super::state::BackupRequest::Plan(preview, options),
        ImportBusy::Plan,
    );
    app.backup_import = Some(form);
    wait(app)
}
fn apply(app: &mut AuthenticatorApp) -> TestResult {
    let mut form = app.backup_import.take().ok_or("form")?;
    app.apply_backup_plan(&mut form);
    app.backup_import = Some(form);
    wait(app)
}
fn click(app: &mut AuthenticatorApp, label: &str) -> TestResult {
    render(app);
    render(app);
    let frame = render(app);
    let bounds = frame
        .platform_output
        .accesskit_update
        .as_ref()
        .ok_or("AX")?
        .nodes
        .iter()
        .find(|(_, n)| n.role() == egui::Role::Button && n.label() == Some(label))
        .and_then(|(_, n)| n.bounds())
        .ok_or("button missing")?;
    let pos = egui::pos2(
        ((bounds.x0 + bounds.x1) / 2.) as f32,
        ((bounds.y0 + bounds.y1) / 2.) as f32,
    );
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
    render(app);
    Ok(())
}
fn viewport_frame(
    app: &mut AuthenticatorApp,
    width: f32,
    height: f32,
    tab: bool,
) -> egui::FullOutput {
    let ctx = app.context.clone();
    let mut events = Vec::new();
    if tab {
        for pressed in [true, false] {
            events.push(egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: Some(egui::Key::Tab),
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            });
        }
    }
    let mut frame = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, height),
            )),
            events,
            ..Default::default()
        },
        |root| app.show(root),
    );
    frame.textures_delta.clear();
    frame
}
fn keyboard_reaches(
    app: &mut AuthenticatorApp,
    width: f32,
    height: f32,
    label: &str,
) -> TestResult {
    viewport_frame(app, width, height, false);
    for _ in 0..100 {
        let frame = viewport_frame(app, width, height, true);
        let focused = app.context.memory(|m| m.focused());
        let reached = frame
            .platform_output
            .accesskit_update
            .as_ref()
            .is_some_and(|tree| {
                tree.nodes.iter().any(|(id, n)| {
                    Some(id.0) == focused.map(|id| id.value()) && n.label() == Some(label)
                })
            });
        if reached {
            for _ in 0..30 {
                viewport_frame(app, width, height, false);
            }
            let next = viewport_frame(app, width, height, false);
            let bounds = next
                .platform_output
                .accesskit_update
                .as_ref()
                .ok_or("AX")?
                .nodes
                .iter()
                .find(|(id, n)| {
                    Some(id.0) == focused.map(|id| id.value()) && n.label() == Some(label)
                })
                .and_then(|(_, n)| n.bounds())
                .ok_or("focused bounds")?;
            assert!(
                bounds.y0 >= 0. && bounds.y1 <= f64::from(height),
                "focused primary action {label:?} must scroll into {width}x{height} viewport: {bounds:?}"
            );
            assert!(
                bounds.x0 >= 0. && bounds.x1 <= f64::from(width),
                "focused action must remain inside viewport width"
            );
            return Ok(());
        }
    }
    Err("keyboard did not reach primary action".into())
}
#[test]
fn backup_import_actual_export_to_fresh_store_full_graph_all_otp_and_explicit_preferences()
-> TestResult {
    let files = tempfile::tempdir()?;
    let path = files.path().join("portable.authpbk");
    let original = fixture()?;
    exported(&path, original.clone())?;
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path(), Vault::new(Timestamp(1)))?;
    let local_id = app.state.vault()?.id();
    let preferences_id = app.state.vault()?.preferences().id();
    preview(&mut app, &path)?;
    let form = app.backup_import.as_mut().ok_or("form")?;
    assert!(form.passphrase.is_empty());
    assert_eq!(
        form.preview.as_ref().ok_or("preview")?.counts().accounts,
        24
    );
    assert!(!form.options.import_display_preferences && !form.options.import_security_preferences);
    form.options.mode = ImportMode::Replace;
    form.options.import_display_preferences = true;
    form.options.import_security_preferences = true;
    plan(&mut app)?;
    let form = app.backup_import.as_ref().ok_or("form")?;
    let candidate = form.plan.as_ref().ok_or("plan")?.candidate_for_review();
    assert!(!candidate.preferences().auto_start());
    let before = std::fs::read(dir.path().join("vault.bin"))?;
    click(&mut app, "Apply reviewed import")?;
    assert!(
        app.backup_import
            .as_ref()
            .is_some_and(|f| f.replace.is_some())
    );
    assert_eq!(before, std::fs::read(dir.path().join("vault.bin"))?);
    click(&mut app, "Keep current vault · go back")?;
    assert!(
        app.backup_import
            .as_ref()
            .is_some_and(|f| f.replace.is_none()),
        "synthetic cancel must close replacement dialog"
    );
    assert_eq!(before, std::fs::read(dir.path().join("vault.bin"))?);
    click(&mut app, "Apply reviewed import")?;
    assert!(
        app.backup_import
            .as_ref()
            .is_some_and(|f| f.replace.is_some()),
        "synthetic retry must reopen replacement dialog"
    );
    // The shared modal scrolls long reviews. Tab from initially focused Keep
    // current brings the destructive action into its clipped scroll holder.
    render_events(
        &mut app,
        600.,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: Some(egui::Key::Tab),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    for _ in 0..30 {
        render(&mut app);
    }
    click(&mut app, "Replace collections and import")?;
    assert!(
        app.state.busy(),
        "synthetic explicit replacement must start worker"
    );
    wait(&mut app)?;
    assert!(app.backup_import.is_none());
    let saved = app.state.vault()?;
    assert_eq!(saved.id(), local_id);
    assert_eq!(saved.preferences().id(), preferences_id);
    let mut graph_expected = original.clone();
    graph_expected.id = saved.id();
    graph_expected.state = saved.state();
    graph_expected.preferences = saved.preferences().clone();
    assert!(
        crate::storage::codec::encode(&graph_expected)?.as_slice()
            == crate::storage::codec::encode(saved)?.as_slice(),
        "complete metadata/association/used/Trash graph must survive unchanged"
    );
    assert_eq!(saved.accounts().len(), 24);
    assert_eq!(saved.folders().len(), 1);
    assert_eq!(saved.memberships().len(), 1);
    assert_eq!(
        saved.backup_sets()[0].codes()[0].used_at(),
        Some(Timestamp(6))
    );
    assert!(saved.backup_sets()[0].codes()[1].used_at().is_none());
    assert!(matches!(
        saved.accounts()[0].state().deletion,
        DeletionState::Trashed { .. }
    ));
    assert_eq!(saved.preferences().lock_timeout_seconds(), 900);
    assert_eq!(saved.preferences().clipboard_timeout_seconds(), 120);
    assert!(!saved.preferences().auto_start());
    assert_eq!(app.state.query, "Public");
    assert_eq!(
        app.codes_preferences.as_ref().ok_or("cache")?.view,
        ViewMode::Grid
    );
    for (a, b) in original.accounts().iter().zip(saved.accounts()) {
        assert_eq!(a.id(), b.id());
        assert_eq!(a.state(), b.state());
        assert_eq!(a.credential().service(), b.credential().service());
        assert_eq!(
            a.credential().account_identifier(),
            b.credential().account_identifier()
        );
        assert_eq!(a.credential().website(), b.credential().website());
        assert_eq!(a.credential().settings(), b.credential().settings());
        let settings = a.credential().settings();
        let source = otp::current_code(
            a.credential().secret(),
            settings,
            &codes::FrameClock(Ok(59)),
        )?;
        let restored = otp::current_code(
            b.credential().secret(),
            settings,
            &codes::FrameClock(Ok(59)),
        )?;
        assert!(source.expose_value() == restored.expose_value());
        // RFC6238 independent published HMAC-SHA1 value at t=59.
        if settings.algorithm == Algorithm::Sha1
            && settings.digits == Digits::Eight
            && settings.period == Period::Seconds30
        {
            assert!(restored.expose_value() == "94287082");
        }
    }
    Ok(())
}
#[test]
fn backup_import_unique_folder_and_paginated_associations_show_actual_candidate_states()
-> TestResult {
    let files = tempfile::tempdir()?;
    let path = files.path().join("associations.authpbk");
    let mut source = fixture()?;
    let folder = source.folders()[0].id();
    for index in 1..7 {
        source.add_membership(folder, source.accounts()[index].id(), Timestamp(9))?;
    }
    source.memberships[1].state.deletion = DeletionState::Trashed { at: Timestamp(9) };
    source.validate()?;
    exported(&path, source)?;
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path(), Vault::new(Timestamp(1)))?;
    preview(&mut app, &path)?;
    let preview = app
        .backup_import
        .as_ref()
        .and_then(|f| f.preview.as_ref())
        .ok_or("preview")?;
    assert_eq!(
        preview.folder_decisions(0),
        Some(crate::backup::FolderDecisions {
            keep_local: false,
            keep_both: false
        })
    );
    assert!(preview.folder_decisions(1).is_none());
    let labels = names(&render(&mut app));
    assert!(
        labels
            .iter()
            .any(|s| s
                == "No current folder conflict. Import as a new folder with its associations.")
    );
    assert!(!labels.iter().any(|s| s == "Folder decision"));
    plan(&mut app)?;
    let candidate = app
        .backup_import
        .as_ref()
        .and_then(|f| f.plan.as_ref())
        .ok_or("plan")?
        .candidate_for_review();
    let links = candidate
        .memberships()
        .iter()
        .map(|link| {
            (
                format!("Membership ID {}", link.id().as_uuid()),
                format!("Account ID {}", link.account_id().as_uuid()),
            )
        })
        .collect::<Vec<_>>();
    let folder_id = format!(
        "{:?} · ID {}",
        candidate.folders()[0].color(),
        candidate.folders()[0].id().as_uuid()
    );
    for width in [340., 600., 960.] {
        let labels = names(&render_events(&mut app, width, Vec::new()));
        assert!(
            labels
                .iter()
                .any(|s| s == "Resulting account–folder associations")
        );
        assert!(labels.contains(&folder_id));
        assert!(
            labels
                .iter()
                .any(|s| s == "Association: created 4 · updated 4 · Active")
        );
        assert!(
            labels
                .iter()
                .any(|s| s == "Association: created 9 · updated 9 · Trash since 9")
        );
        for (id, account) in &links[..5] {
            assert!(labels.contains(id) && labels.contains(account));
        }
        assert!(!labels.contains(&links[5].0));
        assert!(labels.iter().all(|s| !s.contains("public-recovery-only")
            && !s.contains("public-unused-only")
            && !s.contains("GEZDGNBV")));
    }
    let pagination_ids = |out: &egui::FullOutput| {
        out.platform_output
            .accesskit_update
            .as_ref()
            .map(|tree| {
                tree.nodes
                    .iter()
                    .filter(|(_, n)| {
                        n.role() == egui::Role::Button
                            && matches!(
                                n.label(),
                                Some("Previous association page" | "Next association page")
                            )
                    })
                    .map(|(id, n)| (n.label().unwrap_or_default().to_owned(), *id))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };
    let original_ids = pagination_ids(&render(&mut app));
    assert_eq!(original_ids.len(), 2);
    click(&mut app, "Next association page")?;
    let frame = render(&mut app);
    assert_eq!(pagination_ids(&frame), original_ids);
    let labels = names(&frame);
    for (id, account) in &links[5..] {
        assert!(labels.contains(id) && labels.contains(account));
    }
    assert!(!labels.contains(&links[0].0));
    click(&mut app, "Previous association page")?;
    let frame = render(&mut app);
    assert_eq!(pagination_ids(&frame), original_ids);
    let labels = names(&frame);
    assert!(labels.contains(&links[0].0) && !labels.contains(&links[5].0));
    Ok(())
}
#[test]
fn backup_import_wrong_password_tamper_future_local_reference_and_oversize_preserve_session()
-> TestResult {
    let files = tempfile::tempdir()?;
    let path = files.path().join("backup.authpbk");
    exported(&path, fixture()?)?;
    let bytes = std::fs::read(&path)?;
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path(), Vault::new(Timestamp(1)))?;
    let before = std::fs::read(dir.path().join("vault.bin"))?;
    for kind in 0..6 {
        let selected = files.path().join(format!("case-{kind}.vault"));
        let mut data = bytes.clone();
        match kind {
            1 => {
                let i = data.len() - 1;
                data[i] ^= 1;
            }
            2 => data[8..12].copy_from_slice(&999u32.to_le_bytes()),
            3 => data = b"AUTHLOC1unsupported-local".to_vec(),
            4 => data = b"unproven-reference-private-format".to_vec(),
            5 => data = vec![0; 144 + 64 * 1024 * 1024 + 17],
            _ => {}
        }
        std::fs::write(&selected, data)?;
        let mut form = ImportForm::new(egui::Id::unique("bad"));
        form.path = Some(selected);
        form.passphrase = if kind == 0 {
            Zeroizing::new("incorrect-backup-passphrase".into())
        } else {
            backup_pass()
        };
        app.start_backup_preview(&mut form);
        app.backup_import = Some(form);
        wait(&mut app)?;
        let form = app.backup_import.as_ref().ok_or("form")?;
        assert!(
            form.preview.is_none()
                && form.plan.is_none()
                && form.error.is_some()
                && form.passphrase.is_empty()
        );
        if matches!(kind, 2..=4) {
            assert!(form.error.is_some_and(|text| text.contains("unsupported")));
        }
        assert_eq!(before, std::fs::read(dir.path().join("vault.bin"))?);
        assert!(app.state.is_unlocked());
    }
    Ok(())
}
#[test]
fn backup_import_default_merge_keep_both_folder_reuse_and_back_rebuild() -> TestResult {
    let files = tempfile::tempdir()?;
    let path = files.path().join("backup.authpbk");
    let original = fixture()?;
    exported(&path, original.clone())?;
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path(), original)?;
    preview(&mut app, &path)?;
    plan(&mut app)?;
    let summary = app
        .backup_import
        .as_ref()
        .and_then(|f| f.plan.as_ref())
        .ok_or("plan")?
        .summary();
    assert_eq!(summary.skipped_account_bundles, 24);
    assert_eq!(summary.after.accounts, 24);
    assert!(!summary.security_preferences_imported && !summary.display_preferences_imported);
    let before = std::fs::read(dir.path().join("vault.bin"))?;
    click(&mut app, "Back to import choices")?;
    let form = app.backup_import.as_mut().ok_or("form")?;
    assert!(form.preview.is_some() && form.plan.is_none());
    form.options
        .account_choices
        .push((0, MergeChoice::KeepBoth));
    form.options.folder_choices.push((0, MergeChoice::KeepBoth));
    plan(&mut app)?;
    let summary = app
        .backup_import
        .as_ref()
        .and_then(|f| f.plan.as_ref())
        .ok_or("plan")?
        .summary();
    assert_eq!(summary.after.accounts, 25);
    assert_eq!(summary.after.folders, 2);
    assert_eq!(summary.after.used_codes, 2);
    assert_eq!(before, std::fs::read(dir.path().join("vault.bin"))?);
    apply(&mut app)?;
    assert_eq!(app.state.vault()?.accounts().len(), 25);
    assert_eq!(app.state.vault()?.folders().len(), 2);
    Ok(())
}
#[test]
fn backup_import_pending_preferences_flush_failure_retry_and_default_security() -> TestResult {
    let files = tempfile::tempdir()?;
    let path = files.path().join("backup.authpbk");
    exported(&path, fixture()?)?;
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path(), Vault::new(Timestamp(1)))?;
    render(&mut app);
    app.state.query = "pending local display".into();
    let cache = app.codes_preferences.as_mut().ok_or("cache")?;
    cache.query = Zeroizing::new(app.state.query.clone());
    cache.view = ViewMode::Grid;
    app.preferences_changed();
    app.state
        .repository_mut()?
        .fail_next_backup_write_for_test(true);
    preview(&mut app, &path)?;
    assert!(
        app.backup_import
            .as_ref()
            .is_some_and(|f| f.preview.is_none() && f.passphrase.is_empty() && f.error.is_some())
    );
    app.state
        .repository_mut()?
        .fail_next_backup_write_for_test(false);
    let mut form = app.backup_import.take().ok_or("form")?;
    form.passphrase = backup_pass();
    app.start_backup_preview(&mut form);
    app.backup_import = Some(form);
    wait(&mut app)?;
    let form = app.backup_import.as_mut().ok_or("form")?;
    assert!(
        form.preview
            .as_ref()
            .ok_or("preview")?
            .local_vault()
            .preferences()
            .codes()
            .query
            .as_str()
            == "pending local display"
    );
    form.options.mode = ImportMode::Replace;
    plan(&mut app)?;
    let candidate = app
        .backup_import
        .as_ref()
        .and_then(|f| f.plan.as_ref())
        .ok_or("plan")?
        .candidate_for_review();
    assert_eq!(candidate.preferences().lock_timeout_seconds(), 300);
    assert_eq!(candidate.preferences().clipboard_timeout_seconds(), 30);
    assert!(!candidate.preferences().auto_start());
    apply(&mut app)?;
    assert_eq!(app.state.query, "pending local display");
    Ok(())
}
#[test]
fn backup_import_source_local_stale_write_failure_and_retry_are_atomic() -> TestResult {
    let files = tempfile::tempdir()?;
    let path = files.path().join("backup.authpbk");
    exported(&path, fixture()?)?;
    let bytes = std::fs::read(&path)?;
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path(), Vault::new(Timestamp(1)))?;
    let before = std::fs::read(dir.path().join("vault.bin"))?;
    for kind in 0..3 {
        preview(&mut app, &path)?;
        plan(&mut app)?;
        match kind {
            0 => std::fs::write(&path, b"changed source")?,
            1 => app
                .state
                .repository_mut()?
                .transaction(|v| v.set_lock_timeout(60, Timestamp(10)))?,
            _ => app
                .state
                .repository_mut()?
                .fail_next_backup_write_for_test(true),
        }
        let current = std::fs::read(dir.path().join("vault.bin"))?;
        apply(&mut app)?;
        assert_eq!(current, std::fs::read(dir.path().join("vault.bin"))?);
        assert!(app.state.vault()?.accounts().is_empty());
        assert!(
            app.backup_import
                .as_ref()
                .is_some_and(|f| f.error.is_some() && f.preview.is_none())
        );
        std::fs::write(&path, &bytes)?;
        app.state
            .repository_mut()?
            .fail_next_backup_write_for_test(false);
    }
    assert_ne!(before, std::fs::read(dir.path().join("vault.bin"))?);
    preview(&mut app, &path)?;
    plan(&mut app)?;
    apply(&mut app)?;
    assert!(!app.state.vault()?.accounts().is_empty());
    Ok(())
}
#[test]
fn backup_import_lock_scrubs_completed_preview_and_admitted_write_without_late_success()
-> TestResult {
    let files = tempfile::tempdir()?;
    let path = files.path().join("backup.authpbk");
    exported(&path, fixture()?)?;
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path(), Vault::new(Timestamp(1)))?;
    preview(&mut app, &path)?;
    plan(&mut app)?;
    let before = std::fs::read(dir.path().join("vault.bin"))?;
    let gate = Arc::new(CommitGate::default());
    let _release = gate.release_on_drop();
    app.state.hold_next_save_after_admission(gate.clone());
    let mut form = app.backup_import.take().ok_or("form")?;
    app.apply_backup_plan(&mut form);
    app.backup_import = Some(form);
    assert!(gate.wait_entered(Duration::from_secs(15)));
    let started = Instant::now();
    app.lock();
    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(!gate.is_released());
    assert!(
        app.backup_import.is_none() && app.backup_picker.is_none() && app.state.query.is_empty()
    );
    let labels = names(&render(&mut app));
    assert!(labels.iter().all(|l| !l.contains("public-import-fixture")
        && !l.contains("Backup passphrase")
        && !l.contains("Apply reviewed")));
    assert_eq!(before, std::fs::read(dir.path().join("vault.bin"))?);
    gate.release();
    wait(&mut app)?;
    assert!(!app.state.is_unlocked());
    assert!(app.capture_message.is_none());
    app.state.repository_mut()?.unlock(local_pass())?;
    assert!(!app.state.vault()?.accounts().is_empty());
    preview(&mut app, &path)?;
    app.lock();
    wait(&mut app)?;
    assert!(app.backup_import.is_none());
    Ok(())
}
#[test]
fn backup_import_masked_validation_responsive_bounded_preview_and_help_escape() -> TestResult {
    let files = tempfile::tempdir()?;
    let path = files.path().join("backup.authpbk");
    exported(&path, fixture()?)?;
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path(), Vault::new(Timestamp(1)))?;
    click(&mut app, "Import backup")?;
    let before = render(&mut app);
    let submit_id = |frame: &egui::FullOutput| {
        frame
            .platform_output
            .accesskit_update
            .as_ref()
            .and_then(|tree| {
                tree.nodes
                    .iter()
                    .find(|(_, n)| {
                        n.role() == egui::Role::Button
                            && n.label() == Some("Unlock backup and preview")
                    })
                    .map(|(id, _)| *id)
            })
    };
    let id = submit_id(&before).ok_or("submit identity")?;
    click(&mut app, "Unlock backup and preview")?;
    let invalid = render(&mut app);
    assert_eq!(
        submit_id(&invalid),
        Some(id),
        "error insertion must retain submit identity for keyboard correction"
    );
    assert!(
        app.backup_import
            .as_ref()
            .is_some_and(|f| f.error.is_some())
    );
    let form = app.backup_import.as_mut().ok_or("form")?;
    form.path = Some(path.clone());
    form.passphrase = backup_pass();
    for width in [340., 600., 960.] {
        let frame = render_events(&mut app, width, Vec::new());
        let labels = names(&frame);
        assert!(
            labels
                .iter()
                .all(|l| !l.contains("disposable-native-backup"))
        );
        assert!(labels.iter().any(|l| l.contains('•')));
    }
    preview(&mut app, &path)?;
    for width in [340., 600., 960.] {
        let labels = names(&render_events(&mut app, width, Vec::new()));
        assert!(labels.iter().any(|l| l.contains("Page 1 of")));
        assert!(
            labels
                .iter()
                .all(|l| !l.contains("public-recovery-only") && !l.contains("GEZDGNBV"))
        );
    }
    app.dialog = Some(c::Dialog::new("Offline help", egui::Id::unique("help")));
    app.help_open = true;
    render(&mut app);
    render_events(
        &mut app,
        600.,
        vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: Some(egui::Key::Escape),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert!(
        app.dialog.is_none()
            && app
                .backup_import
                .as_ref()
                .is_some_and(|f| f.preview.is_some())
    );
    app.lock();
    assert!(app.backup_import.is_none());
    Ok(())
}
#[test]
fn backup_import_normal_viewports_keyboard_scroll_reaches_each_primary_action() -> TestResult {
    let files = tempfile::tempdir()?;
    let path = files.path().join("backup.authpbk");
    exported(&path, fixture()?)?;
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path(), Vault::new(Timestamp(1)))?;
    app.backup_import = Some(ImportForm::new(egui::Id::unique("normal-viewport")));
    for (width, height) in [(340., 780.), (600., 800.), (960., 900.)] {
        keyboard_reaches(&mut app, width, height, "Unlock backup and preview")?;
    }
    preview(&mut app, &path)?;
    for (width, height) in [(340., 780.), (600., 800.), (960., 900.)] {
        keyboard_reaches(&mut app, width, height, "Review exact import changes")?;
    }
    plan(&mut app)?;
    for (width, height) in [(340., 780.), (600., 800.), (960., 900.)] {
        keyboard_reaches(&mut app, width, height, "Apply reviewed import")?;
    }
    Ok(())
}
#[test]
fn backup_import_pre_admission_lifecycle_deadline_and_early_lock_preserve_bytes() -> TestResult {
    let files = tempfile::tempdir()?;
    let path = files.path().join("backup.authpbk");
    exported(&path, fixture()?)?;
    let dir = tempfile::tempdir()?;
    let mut app = app(dir.path(), Vault::new(Timestamp(1)))?;
    let before = std::fs::read(dir.path().join("vault.bin"))?;
    let mut form = ImportForm::new(egui::Id::unique("early-lock"));
    form.path = Some(path.clone());
    form.passphrase = backup_pass();
    app.start_backup_preview(&mut form);
    assert_eq!(
        app.state.begin_backup(
            super::super::state::BackupRequest::Preview(path.clone(), backup_pass()),
            app.lifecycle.as_ref().map_err(|e| *e)?.watch(),
            || {}
        ),
        Err(crate::error::AppError::VaultBusy)
    );
    app.backup_import = Some(form);
    app.lock();
    wait(&mut app)?;
    assert!(!app.state.is_unlocked() && app.backup_import.is_none());
    assert_eq!(before, std::fs::read(dir.path().join("vault.bin"))?);
    app.state.repository_mut()?.unlock(local_pass())?;
    preview(&mut app, &path)?;
    plan(&mut app)?;
    let mut form = app.backup_import.take().ok_or("form")?;
    let plan = form.plan.take().ok_or("plan")?;
    let lifecycle = app.lifecycle.as_ref().map_err(|e| *e)?;
    let watch = lifecycle.watch();
    lifecycle.notify_lock_for_test();
    app.state.begin_backup(
        super::super::state::BackupRequest::Apply(Box::new(plan)),
        watch,
        || {},
    )?;
    drop(form);
    wait(&mut app)?;
    assert!(!app.state.is_unlocked());
    assert!(app.state.take_backup_result().is_none());
    assert_eq!(before, std::fs::read(dir.path().join("vault.bin"))?);
    app.state.repository_mut()?.unlock(local_pass())?;
    app.state.activity(
        Instant::now()
            .checked_sub(Duration::from_secs(600))
            .ok_or("clock")?,
    );
    let mut form = ImportForm::new(egui::Id::unique("expired"));
    form.path = Some(path);
    form.passphrase = backup_pass();
    app.start_backup_preview(&mut form);
    app.backup_import = Some(form);
    wait(&mut app)?;
    assert!(!app.state.is_unlocked() && app.backup_import.is_none());
    assert_eq!(before, std::fs::read(dir.path().join("vault.bin"))?);
    Ok(())
}
