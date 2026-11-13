//! Generated public fixtures only. Context events are not native OS input proof.
use super::*;
use crate::{
    domain::{
        otp::{OtpSettings, Secret},
        records::*,
    },
    platform::clipboard::{ClipboardError, TestObserver},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn pass() -> Zeroizing<String> {
    Zeroizing::new("disposable-detail-fixture".into())
}
fn fixture(count: usize, long: bool) -> Result<(Vault, AccountId), crate::error::AppError> {
    let mut vault = Vault::new(Timestamp(100));
    let id = vault.add_account(
        AccountDraft::new(
            "Public recovery detail",
            None,
            Some("public@fixture.invalid"),
            Secret::from_base32("MY")?,
            OtpSettings::default(),
        )?,
        Timestamp(100),
    )?;
    let codes = (0..count)
        .map(|i| {
            let value = if long && i == 0 {
                "x".repeat(4096)
            } else {
                format!("public-detail-code-{i}")
            };
            let mut code = BackupCode::new(&value, Timestamp(100))?;
            if i == 0 {
                code.set_used(true, Timestamp(100))?;
            }
            Ok(code)
        })
        .collect::<Result<Vec<_>, crate::error::AppError>>()?;
    vault.add_backup_set(id, codes, Timestamp(100))?;
    Ok((vault, id))
}
fn app(
    vault: Vault,
    path: &std::path::Path,
) -> Result<(AuthenticatorApp, TestObserver), Box<dyn std::error::Error>> {
    let mut store = VaultStore::open_directory(path)?;
    store.create(pass(), vault)?;
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let mut app = AuthenticatorApp::with_context(ctx, Ok(store), 0);
    let (clipboard, observer) = crate::platform::clipboard::test_fixture();
    app.clipboard = clipboard;
    app.state.repository_mut()?.unlock(pass())?;
    app.state.navigate(Route::BackupCodes);
    Ok((app, observer))
}
fn render(
    app: &mut AuthenticatorApp,
    size: [f32; 2],
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    let ctx = app.context.clone();
    let mut out = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size.into())),
            // Advance UI animation time deterministically. Tight CI loops must
            // still let wheel smoothing settle before the pointer-event hit.
            time: Some(ctx.input(|i| i.time) + 1. / 60.),
            events,
            ..Default::default()
        },
        |root| app.show(root),
    );
    out.textures_delta.clear();
    out
}
fn settle(app: &mut AuthenticatorApp, size: [f32; 2]) -> egui::FullOutput {
    for _ in 0..24 {
        render(app, size, vec![]);
    }
    render(app, size, vec![])
}
fn names(out: &egui::FullOutput) -> Vec<String> {
    out.platform_output
        .accesskit_update
        .as_ref()
        .into_iter()
        .flat_map(|t| &t.nodes)
        .flat_map(|(_, n)| {
            [n.label(), n.value()]
                .into_iter()
                .flatten()
                .map(str::to_owned)
        })
        .collect()
}
#[track_caller]
fn no_secrets(out: &egui::FullOutput) {
    assert!(
        names(out)
            .iter()
            .all(|v| !v.contains("public-detail-code-") && !v.contains(&"x".repeat(100)))
    );
    assert!(
        out.platform_output
            .commands
            .iter()
            .all(|v| !matches!(v, egui::OutputCommand::CopyText(_)))
    );
}
fn wheel(app: &mut AuthenticatorApp, size: [f32; 2], delta: f32) {
    render(
        app,
        size,
        vec![
            egui::Event::PointerMoved(egui::pos2(24., size[1] / 2.)),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., delta),
                modifiers: egui::Modifiers::NONE,
                phase: egui::TouchPhase::Move,
                source: egui::MouseWheelSource::Unknown,
            },
        ],
    );
    settle(app, size);
}
fn button(out: &egui::FullOutput, label: &str) -> Option<(u64, egui::Rect)> {
    button_nth(out, label, 0)
}
fn button_nth(out: &egui::FullOutput, label: &str, occurrence: usize) -> Option<(u64, egui::Rect)> {
    out.platform_output
        .accesskit_update
        .as_ref()?
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == egui::Role::Button && n.label() == Some(label))
        .nth(occurrence)
        .and_then(|(id, n)| {
            n.bounds().map(|b| {
                (
                    id.0,
                    egui::Rect::from_min_max(
                        egui::pos2(b.x0 as f32, b.y0 as f32),
                        egui::pos2(b.x1 as f32, b.y1 as f32),
                    ),
                )
            })
        })
}
fn click(app: &mut AuthenticatorApp, size: [f32; 2], label: &str) -> TestResult {
    click_nth(app, size, label, 0)
}
fn click_nth(
    app: &mut AuthenticatorApp,
    size: [f32; 2],
    label: &str,
    occurrence: usize,
) -> TestResult {
    let mut out = settle(app, size);
    // Wheel scrolling is deliberately used; drag-scroll must not capture actions.
    for _ in 0..30 {
        let bottom = size[1]
            - if app.clipboard_feedback.is_some() {
                theme::CLIPBOARD_FEEDBACK_HEIGHT
            } else {
                0.
            };
        if let Some((_, rect)) = button_nth(&out, label, occurrence)
            && rect.center().y > 0.
            && rect.center().y < bottom - 12.
        {
            let pos = rect.center();
            for pressed in [true, false] {
                render(
                    app,
                    size,
                    vec![
                        egui::Event::PointerMoved(pos),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                );
            }
            return Ok(());
        }
        let delta = button_nth(&out, label, occurrence).map_or(-320., |(_, rect)| {
            (bottom / 2. - rect.center().y).clamp(-2400., 2400.)
        });
        wheel(app, size, delta);
        out = settle(app, size);
    }
    Err(format!(
        "action unavailable: {label}; occurrence {occurrence}; button bounds {:?}",
        button_nth(&out, label, occurrence).map(|b| b.1)
    )
    .into())
}
fn key(app: &mut AuthenticatorApp, size: [f32; 2], key: egui::Key) {
    render(
        app,
        size,
        vec![egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    render(
        app,
        size,
        vec![egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed: false,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
}
fn tab_to(app: &mut AuthenticatorApp, size: [f32; 2], label: &str) -> TestResult {
    let out = settle(app, size);
    let target = button(&out, label).ok_or("keyboard target")?.0;
    for _ in 0..128 {
        if app
            .context
            .memory(|m| m.focused().is_some_and(|id| id.value() == target))
        {
            return Ok(());
        }
        key(app, size, egui::Key::Tab);
    }
    Err(format!("keyboard target unreachable: {label}").into())
}
fn finish(app: &mut AuthenticatorApp) -> TestResult {
    let until = Instant::now() + Duration::from_secs(45);
    while app.state.busy() {
        if Instant::now() > until {
            return Err("detail save timeout".into());
        }
        render(app, [600., 800.], vec![]);
        std::thread::sleep(Duration::from_millis(5));
    }
    settle(app, [600., 800.]);
    Ok(())
}
fn wait(condition: impl Fn() -> bool) -> TestResult {
    let until = Instant::now() + Duration::from_secs(5);
    while !condition() && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(10));
    }
    if condition() {
        Ok(())
    } else {
        Err("clipboard fixture timeout".into())
    }
}

#[test]
fn detail_pointer_copy_never_changes_used_ciphertext_and_actor_expiry_lock_failure() -> TestResult {
    let dir = tempfile::tempdir()?;
    let (v, _) = fixture(2, false)?;
    let (mut app, observer) = app(v, dir.path())?;
    click(&mut app, [600., 800.], "View set")?;
    no_secrets(&settle(&mut app, [600., 800.]));
    let disk = std::fs::read(dir.path().join("vault.bin"))?;
    click(&mut app, [600., 800.], "Copy code 2")?;
    wait(|| observer.matches("public-detail-code-1"))?;
    settle(&mut app, [600., 800.]);
    assert!(
        app.clipboard_feedback
            .is_some_and(|(s, _)| s.starts_with("Copied."))
    );
    assert!(
        app.state.vault()?.backup_sets()[0].codes()[1]
            .used_at()
            .is_none()
    );
    assert!(app.backup_codes.detail.undo.is_none());
    assert_eq!(std::fs::read(dir.path().join("vault.bin"))?, disk);
    observer.fail_next_copy(ClipboardError::Changed);
    click(&mut app, [600., 800.], "Copy code 1")?;
    for _ in 0..20 {
        settle(&mut app, [600., 800.]);
        if app
            .clipboard_feedback
            .is_some_and(|(s, _)| s.starts_with("The clipboard changed"))
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        app.clipboard_feedback
            .is_some_and(|(s, _)| s.starts_with("The clipboard changed"))
    );
    assert!(observer.matches("public-detail-code-1"));
    assert_eq!(std::fs::read(dir.path().join("vault.bin"))?, disk);
    // A newer external value, including identical text, is never cleared by Lock.
    observer.external_copy("public-detail-code-1");
    app.lock();
    std::thread::sleep(Duration::from_millis(150));
    assert!(observer.matches("public-detail-code-1"));
    app.state.repository_mut()?.unlock(pass())?;
    app.state.navigate(Route::BackupCodes);
    click(&mut app, [600., 800.], "View set")?;
    app.set_idle_timeout(300);
    finish(&mut app)?;
    assert!(app.begin_mutation(SaveIntent::ClipboardTimeout(1)));
    finish(&mut app)?;
    click(&mut app, [600., 800.], "Copy code 1")?;
    wait(|| observer.matches("public-detail-code-0"))?;
    wait(|| observer.matches(""))?; // independent of UI frames
    click(&mut app, [600., 800.], "Copy code 2")?;
    wait(|| observer.matches("public-detail-code-1"))?;
    app.lock();
    wait(|| observer.matches(""))?;
    no_secrets(&settle(&mut app, [340., 780.]));
    Ok(())
}

#[test]
fn detail_mark_undo_exact_history_encrypted_restart_and_portable_restore() -> TestResult {
    let dir = tempfile::tempdir()?;
    let (v, account) = fixture(2, false)?;
    let original_ids: Vec<_> = v.backup_sets()[0]
        .codes()
        .iter()
        .map(BackupCode::id)
        .collect();
    let (mut app, _) = app(v, dir.path())?;
    click(&mut app, [600., 800.], "View set")?;
    tab_to(&mut app, [600., 800.], "Mark code 1 available")?;
    key(&mut app, [600., 800.], egui::Key::Enter);
    finish(&mut app)?;
    assert!(
        app.state.vault()?.backup_sets()[0].codes()[0]
            .used_at()
            .is_none()
    );
    let undo = app
        .backup_codes
        .detail
        .undo
        .as_ref()
        .ok_or("actual poststate undo")?;
    undo.0.check(app.state.vault()?)?;
    tab_to(&mut app, [600., 800.], "Undo last used-state change")?;
    key(&mut app, [600., 800.], egui::Key::Enter);
    finish(&mut app)?;
    assert_eq!(
        app.state.vault()?.backup_sets()[0].codes()[0].used_at(),
        Some(Timestamp(100))
    );
    assert!(app.backup_codes.detail.undo.is_none());
    click(&mut app, [600., 800.], "Mark code 2 used")?;
    finish(&mut app)?;
    let used = app.state.vault()?.backup_sets()[0].codes()[1]
        .used_at()
        .ok_or("used timestamp")?;
    assert_eq!(app.state.vault()?.backup_sets()[0].state().updated_at, used);
    assert_eq!(app.state.vault()?.accounts()[0].state().updated_at, used);
    click(&mut app, [600., 800.], "Copy code 1")?;
    assert!(app.backup_codes.detail.undo.is_some());
    tab_to(&mut app, [600., 800.], "Edit backup codes")?;
    key(&mut app, [600., 800.], egui::Key::Enter);
    assert!(app.backup_codes.detail.undo.is_none());
    click(&mut app, [600., 800.], "Cancel")?;
    no_secrets(&settle(&mut app, [600., 800.]));
    app.lock();
    app.state.repository_mut()?.unlock(pass())?;
    assert_eq!(
        app.state.vault()?.backup_sets()[0].codes()[1].used_at(),
        Some(used)
    );
    let path = dir.path().join("public-portable.authpbk");
    crate::backup::export(
        app.state.repository_mut()?,
        crate::backup::Destination::create_new(&path)?,
        pass(),
        &crate::storage::OperationCancellation::default(),
    )?;
    let preview = crate::backup::preview(
        app.state.repository_mut()?,
        &path,
        pass(),
        &crate::storage::OperationCancellation::default(),
    )?;
    let incoming = preview.incoming_vault();
    incoming.validate()?;
    assert_eq!(incoming.backup_sets()[0].account_id(), account);
    assert_eq!(
        incoming.backup_sets()[0]
            .codes()
            .iter()
            .map(BackupCode::id)
            .collect::<Vec<_>>(),
        original_ids
    );
    assert_eq!(
        incoming.backup_sets()[0].codes()[0].used_at(),
        Some(Timestamp(100))
    );
    assert_eq!(incoming.backup_sets()[0].codes()[1].used_at(), Some(used));
    Ok(())
}

#[test]
fn detail_reveal_keyboard_narrow_values_pagination_navigation_lock_and_redaction() -> TestResult {
    for size in [[340., 780.], [600., 800.], [960., 900.]] {
        let dir = tempfile::tempdir()?;
        let (v, _) = fixture(21, true)?;
        let (mut app, observer) = app(v, dir.path())?;
        click(&mut app, size, "View set")?;
        no_secrets(&settle(&mut app, size));
        click(&mut app, size, "Reveal code 1")?;
        let revealed = settle(&mut app, size);
        assert!(
            names(&revealed)
                .iter()
                .any(|s| s.contains(&"x".repeat(100)))
        );
        assert!(!app.evidence_code_bounds.is_empty());
        assert!(
            app.evidence_code_bounds
                .iter()
                .all(|b| b.height() <= theme::MULTILINE_HEIGHT + 1.)
        );
        for label in [
            "Conceal code 1",
            "Copy code 1",
            "Mark code 1 available",
            "Delete code 1…",
        ] {
            let rect = button(&revealed, label).ok_or("row geometry")?.1;
            assert!(rect.width() >= 100. && rect.height() >= theme::CONTROL_HEIGHT);
            assert!(rect.min.x >= 0. && rect.max.x <= size[0]);
        }
        assert!(
            revealed
                .platform_output
                .accesskit_update
                .as_ref()
                .ok_or("tree")?
                .nodes
                .iter()
                .filter(|(_, n)| n.live().is_some())
                .all(
                    |(_, n)| n.label().is_none_or(|s| !s.contains(&"x".repeat(100)))
                        && n.value().is_none_or(|s| !s.contains(&"x".repeat(100)))
                )
        );
        tab_to(&mut app, size, "Conceal code 1")?;
        key(&mut app, size, egui::Key::Enter);
        no_secrets(&settle(&mut app, size));
        // Tab navigation reaches the independent Copy control.
        key(&mut app, size, egui::Key::Tab);
        let out = settle(&mut app, size);
        let copy_id = button(&out, "Copy code 1").ok_or("copy control")?.0;
        assert_eq!(
            app.context.memory(|m| m.focused().map(|id| id.value())),
            Some(copy_id)
        );
        key(&mut app, size, egui::Key::Enter);
        wait(|| observer.matches(&"x".repeat(4096)))?;
        click(&mut app, size, "Reveal code 1")?;
        click(&mut app, size, "Next codes")?;
        let next = settle(&mut app, size);
        no_secrets(&next);
        assert!(button(&next, "Reveal code 21").is_some());
        click(&mut app, size, "Previous codes")?;
        no_secrets(&settle(&mut app, size));
        click(&mut app, size, "Reveal code 1")?;
        // Leaving by the footer after values were drawn must retain this frame's
        // redaction rectangles until the owned evidence callback consumes them.
        click(&mut app, size, "Back")?;
        no_secrets(&settle(&mut app, size));
        click(&mut app, size, "View set")?;
        no_secrets(&settle(&mut app, size));
        click(&mut app, size, "Reveal code 1")?;
        if size[0] == 340. {
            app.state.activity(
                Instant::now()
                    .checked_sub(Duration::from_secs(301))
                    .ok_or("idle instant")?,
            );
        } else {
            app.lock();
        }
        no_secrets(&settle(&mut app, size));
        assert!(!app.state.session_active());
    }
    Ok(())
}

#[test]
fn detail_trash_confirm_cancel_focus_fault_retry_and_account_associations() -> TestResult {
    let dir = tempfile::tempdir()?;
    let (v, account) = fixture(2, false)?;
    let (mut app, _) = app(v, dir.path())?;
    click(&mut app, [600., 800.], "View set")?;
    let prior = crate::domain::recovery_edit::RecoveryTarget::capture(app.state.vault()?, account)?;
    tab_to(&mut app, [600., 800.], "Delete code 1…")?;
    key(&mut app, [600., 800.], egui::Key::Enter);
    key(&mut app, [600., 800.], egui::Key::Escape);
    let out = settle(&mut app, [600., 800.]);
    assert_eq!(
        app.context.memory(|m| m.focused().map(|id| id.value())),
        button(&out, "Delete code 1…").map(|b| b.0)
    );
    prior.check(app.state.vault()?)?;
    let disk = std::fs::read(dir.path().join("vault.bin"))?;
    std::fs::write(dir.path().join("vault.bin"), b"public-write-conflict")?;
    click(&mut app, [600., 800.], "Mark code 2 used")?;
    finish(&mut app)?;
    prior.check(app.state.vault()?)?;
    assert!(app.backup_codes.detail.undo.is_none());
    assert!(
        app.status
            .is_some_and(|s| s.starts_with("Changes could not be saved"))
    );
    std::fs::write(dir.path().join("vault.bin"), disk)?;
    click(&mut app, [600., 800.], "Mark code 2 used")?;
    finish(&mut app)?;
    let disk = std::fs::read(dir.path().join("vault.bin"))?;
    std::fs::write(
        dir.path().join("vault.bin"),
        b"public-second-write-conflict",
    )?;
    click(&mut app, [600., 800.], "Mark code 1 available")?;
    finish(&mut app)?;
    let undo = app
        .backup_codes
        .detail
        .undo
        .as_ref()
        .ok_or("failed write must keep previous Undo")?;
    undo.0.check(app.state.vault()?)?;
    assert_eq!(
        app.state.vault()?.backup_sets()[0].codes()[0].used_at(),
        Some(Timestamp(100))
    );
    std::fs::write(dir.path().join("vault.bin"), disk)?;
    click(&mut app, [600., 800.], "Delete code 1…")?;
    click(&mut app, [600., 800.], "Move code to Trash")?;
    finish(&mut app)?;
    let v = app.state.vault()?;
    assert_eq!(v.backup_sets()[0].codes().len(), 2);
    assert!(matches!(
        v.backup_sets()[0].codes()[0].state().deletion,
        DeletionState::Trashed { .. }
    ));
    assert_eq!(
        v.backup_sets()[0].codes()[0].used_at(),
        Some(Timestamp(100))
    );
    assert_eq!(v.accounts()[0].state().deletion, DeletionState::Active);
    assert!(app.backup_codes.detail.undo.is_none());
    tab_to(&mut app, [600., 800.], "Delete backup-code set…")?;
    key(&mut app, [600., 800.], egui::Key::Enter);
    click(&mut app, [600., 800.], "Move set to Trash")?;
    finish(&mut app)?;
    assert_eq!(app.backup_codes.screen, backup_codes::Screen::List);
    assert_eq!(
        app.state.vault()?.accounts()[0].state().deletion,
        DeletionState::Active
    );
    assert_eq!(app.state.vault()?.backup_sets()[0].codes().len(), 2);
    app.lock();
    app.state.repository_mut()?.unlock(pass())?;
    app.state.vault()?.validate()?;
    assert_eq!(app.state.vault()?.backup_sets()[0].account_id(), account);
    Ok(())
}

#[test]
fn detail_admitted_mark_lock_removes_reveal_and_undo_before_io_release_no_late_adoption()
-> TestResult {
    let dir = tempfile::tempdir()?;
    let (v, _) = fixture(2, false)?;
    let (mut app, _) = app(v, dir.path())?;
    click(&mut app, [600., 800.], "View set")?;
    click(&mut app, [600., 800.], "Mark code 1 available")?;
    finish(&mut app)?;
    click(&mut app, [600., 800.], "Reveal code 2")?;
    let gate = Arc::new(crate::storage::test_gate::CommitGate::default());
    let release = gate.release_on_drop();
    app.state.hold_next_save_after_admission(gate.clone());
    click(&mut app, [600., 800.], "Mark code 2 used")?;
    assert!(gate.wait_entered(Duration::from_secs(10)));
    let started = Instant::now();
    app.lock();
    assert!(started.elapsed() < Duration::from_secs(1));
    no_secrets(&settle(&mut app, [340., 780.]));
    assert!(app.backup_codes.detail.undo.is_none() && app.pending_recovery_code.is_none());
    assert!(!gate.is_released());
    drop(release);
    finish(&mut app)?;
    assert!(!app.state.is_unlocked() && app.status.is_none());
    app.state.repository_mut()?.unlock(pass())?;
    assert!(
        app.state.vault()?.backup_sets()[0].codes()[1]
            .used_at()
            .is_some()
    );
    Ok(())
}

#[test]
fn detail_import_failure_preserves_session_success_reused_ids_conceals_and_parent_trash_restores_history()
-> TestResult {
    let dir = tempfile::tempdir()?;
    let source_dir = tempfile::tempdir()?;
    let (v, account) = fixture(2, false)?;
    let mut incoming = v.clone();
    incoming.backup_sets[0].codes[0].value = Zeroizing::new("public-import-replacement".into());
    let mut source = VaultStore::open_directory(source_dir.path())?;
    source.create(pass(), incoming)?;
    source.unlock(pass())?;
    let path = source_dir.path().join("public-replace.authpbk");
    crate::backup::export(
        &source,
        crate::backup::Destination::create_new(&path)?,
        pass(),
        &crate::storage::OperationCancellation::default(),
    )?;
    let bytes = std::fs::read(&path)?;
    let (mut app, _) = app(v, dir.path())?;
    click(&mut app, [600., 800.], "View set")?;
    click(&mut app, [600., 800.], "Mark code 2 used")?;
    finish(&mut app)?;
    click(&mut app, [600., 800.], "Reveal code 1")?;
    for fail in [true, false] {
        let preview = crate::backup::preview(
            app.state.repository_mut()?,
            &path,
            pass(),
            &crate::storage::OperationCancellation::default(),
        )?;
        let plan = preview.plan(crate::backup::ImportOptions {
            mode: crate::backup::ImportMode::Replace,
            ..Default::default()
        })?;
        if fail {
            std::fs::write(&path, b"public-changed-owned-backup")?;
        }
        let mut form = ImportForm::new(egui::Id::unique("detail-import"));
        form.plan = Some(plan);
        app.apply_backup_plan(&mut form);
        app.backup_import = Some(form);
        finish(&mut app)?;
        if fail {
            assert!(
                app.backup_import
                    .as_ref()
                    .is_some_and(|f| f.error.is_some())
            );
            app.backup_import = None; // cancel failed import, returning to prior view
            assert!(app.backup_codes.detail.undo.is_some());
            assert!(
                names(&settle(&mut app, [600., 800.]))
                    .iter()
                    .any(|s| s.contains("public-detail-code-0"))
            );
            std::fs::write(&path, &bytes)?;
        } else {
            assert!(app.backup_import.is_none());
            assert_eq!(app.backup_codes.screen, backup_codes::Screen::List);
            app.state.navigate(Route::BackupCodes);
            click(&mut app, [600., 800.], "View set")?;
            let out = settle(&mut app, [600., 800.]);
            assert!(
                names(&out)
                    .iter()
                    .all(|s| !s.contains("public-import-replacement"))
            );
            assert!(app.backup_codes.detail.undo.is_none());
        }
    }
    // Task18's UI is pending. Exercise parent association policy using a
    // generated encrypted backend fixture; no production restore API is added.
    let used = app.state.vault()?.backup_sets()[0].codes()[0].used_at();
    let state = app.state.vault()?.accounts()[0].state();
    assert!(app.begin_mutation(SaveIntent::Trash {
        id: account,
        expected: state
    }));
    finish(&mut app)?;
    assert_eq!(app.backup_codes.screen, backup_codes::Screen::List);
    assert!(
        backup_codes::Projection::new(app.state.vault()?, "", SortOrder::LatestFirst)
            .visible_sets()
            .is_empty()
    );
    app.state.repository_mut()?.transaction(|v| {
        v.accounts[0].state.deletion = DeletionState::Active;
        Ok(())
    })?;
    app.backup_codes.invalidate();
    click(&mut app, [600., 800.], "View set")?;
    let out = settle(&mut app, [600., 800.]);
    assert!(
        names(&out)
            .iter()
            .all(|s| !s.contains("public-import-replacement"))
    );
    assert_eq!(app.state.vault()?.backup_sets()[0].account_id(), account);
    assert_eq!(
        app.state.vault()?.backup_sets()[0].codes()[0].used_at(),
        used
    );
    app.state.vault()?.validate()?;
    Ok(())
}

#[test]
fn detail_later_code_and_scrolled_list_focus_are_visible_after_save_undo_and_trash() -> TestResult {
    let dir = tempfile::tempdir()?;
    let (mut v, _) = fixture(21, false)?;
    for index in 0..19 {
        let id = v.add_account(
            AccountDraft::new(
                &format!("Public set {index:02}"),
                None,
                None,
                Secret::from_base32("MY")?,
                OtpSettings::default(),
            )?,
            Timestamp(100),
        )?;
        let codes = (0..21)
            .map(|i| BackupCode::new(&format!("public-detail-code-{i}"), Timestamp(100)))
            .collect::<Result<Vec<_>, _>>()?;
        v.add_backup_set(id, codes, Timestamp(100))?;
    }
    let (mut app, _) = app(v, dir.path())?;
    click_nth(&mut app, [600., 800.], "View set", 19)?;
    let out = settle(&mut app, [600., 800.]);
    assert_visible_focus(&app, &out, "Back", [600., 800.])?;
    click(&mut app, [600., 800.], "Mark code 20 used")?;
    finish(&mut app)?;
    let out = settle(&mut app, [600., 800.]);
    assert_visible_focus(&app, &out, "Mark code 20 available", [600., 800.])?;
    click(&mut app, [600., 800.], "Next codes")?;
    assert!(button(&settle(&mut app, [600., 800.]), "Reveal code 21").is_some());
    click(&mut app, [600., 800.], "Undo last used-state change")?;
    finish(&mut app)?;
    let out = settle(&mut app, [600., 800.]);
    assert_visible_focus(&app, &out, "Mark code 20 used", [600., 800.])?;
    click(&mut app, [600., 800.], "Delete code 20…")?;
    click(&mut app, [600., 800.], "Move code to Trash")?;
    finish(&mut app)?;
    let out = settle(&mut app, [600., 800.]);
    assert_visible_focus(&app, &out, "Reveal code 1", [600., 800.])?;
    no_secrets(&out);
    Ok(())
}
fn assert_visible_focus(
    app: &AuthenticatorApp,
    out: &egui::FullOutput,
    label: &str,
    size: [f32; 2],
) -> TestResult {
    let (id, bounds) = button(out, label).ok_or("focused action")?;
    assert_eq!(
        app.context.memory(|m| m.focused().map(|i| i.value())),
        Some(id)
    );
    assert!(
        bounds.min.y >= 0.
            && bounds.max.y <= size[1]
            && bounds.min.x >= 0.
            && bounds.max.x <= size[0],
        "focused {label} must be visible: {bounds:?}"
    );
    Ok(())
}
