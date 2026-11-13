//! Generated owned fixtures. Context events are not OS pointer/pixel evidence.
use super::*;
use crate::{
    domain::{
        otp::{OtpSettings, Secret},
        records::{AccountDraft, BackupCode, Timestamp, Vault},
    },
    error::AppError,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn pass() -> Zeroizing<String> {
    Zeroizing::new("disposable-recovery-editor".into())
}
fn fixture(existing: bool) -> Result<(Vault, AccountId), AppError> {
    let mut v = Vault::new(Timestamp(100));
    let id = v.add_account(
        AccountDraft::new(
            "Public recovery editor",
            None,
            Some("public@fixture.invalid"),
            Secret::from_base32("MY")?,
            OtpSettings::default(),
        )?,
        Timestamp(100),
    )?;
    if existing {
        let mut used = BackupCode::new("public-used-sentinel", Timestamp(100))?;
        used.set_used(true, Timestamp(100))?;
        v.add_backup_set(
            id,
            vec![
                used,
                BackupCode::new("public-removed-sentinel", Timestamp(100))?,
            ],
            Timestamp(100),
        )?;
    }
    Ok((v, id))
}
fn app(
    vault: Vault,
    path: &std::path::Path,
) -> Result<AuthenticatorApp, Box<dyn std::error::Error>> {
    let mut store = VaultStore::open_directory(path)?;
    store.create(pass(), vault)?;
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let mut app = AuthenticatorApp::with_context(ctx, Ok(store), 0);
    app.state.repository_mut()?.unlock(pass())?;
    app.state.navigate(Route::BackupCodes);
    Ok(app)
}
fn render(
    app: &mut AuthenticatorApp,
    width: f32,
    height: f32,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    let ctx = app.context.clone();
    let mut out = ctx.run_ui(
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
    out.textures_delta.clear();
    out
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
fn click(app: &mut AuthenticatorApp, label: &str) -> TestResult {
    // Immediate-mode modal Areas settle their measured size/focus across passes.
    // Observe the settled tree before deriving Context pointer coordinates.
    for _ in 0..3 {
        render(app, 600., 16000., vec![]);
    }
    let out = render(app, 600., 16000., vec![]);
    let bounds = out
        .platform_output
        .accesskit_update
        .as_ref()
        .ok_or("tree")?
        .nodes
        .iter()
        .find(|(_, n)| n.role() == egui::Role::Button && n.label() == Some(label))
        .and_then(|(_, n)| n.bounds())
        .ok_or_else(|| format!("missing action: {label}"))?;
    let pos = egui::pos2(
        ((bounds.x0 + bounds.x1) / 2.) as f32,
        ((bounds.y0 + bounds.y1) / 2.) as f32,
    );
    for pressed in [true, false] {
        render(
            app,
            600.,
            16000.,
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
    Ok(())
}
fn finish(app: &mut AuthenticatorApp) -> TestResult {
    let deadline = Instant::now() + Duration::from_secs(45);
    while app.state.busy()
        || app
            .recovery_editor
            .as_ref()
            .is_some_and(|e| e.job.is_some())
    {
        if Instant::now() > deadline {
            return Err("recovery worker timeout".into());
        }
        render(app, 600., 16000., vec![]);
        std::thread::sleep(Duration::from_millis(5));
    }
    render(app, 600., 16000., vec![]);
    Ok(())
}
fn edit(app: &mut AuthenticatorApp, id: AccountId, text: &str) -> TestResult {
    let mut editor = RecoveryEditor::new(app.state.vault()?, id)?;
    editor.text = Zeroizing::new(text.into());
    editor.invalidate();
    app.recovery_editor = Some(editor);
    click(app, "Preview changes")?;
    finish(app)
}
#[test]
fn recovery_select_preview_duplicate_review_save_list_and_encrypted_restart() -> TestResult {
    let dir = tempfile::tempdir()?;
    let (v, id) = fixture(false)?;
    let mut app = app(v, dir.path())?;
    click(&mut app, "Add Backup Codes")?;
    // Selector uses a RadioButton role; choose through the same stable ID state.
    app.backup_codes.selected = Some(id);
    click(&mut app, "Continue")?;
    render(&mut app, 600., 16000., vec![]);
    render(&mut app, 600., 16000., vec![]);
    let editor = app.recovery_editor.as_mut().ok_or("editor handoff")?;
    editor.text = Zeroizing::new("\u{feff} public-alpha \r\n\nβeta\npublic-alpha\n".into());
    editor.invalidate();
    assert!(editor.save_intent().is_err());
    click(&mut app, "Preview changes")?;
    finish(&mut app)?;
    assert!(matches!(
        app.recovery_editor.as_ref().ok_or("editor")?.save_intent(),
        Err(AppError::ReviewBackupDuplicates)
    ));
    let preview = names(&render(&mut app, 600., 16000., vec![]));
    assert!(preview.iter().any(|s| {
        s.contains("2 unique valid code(s) · 1 blank line(s) discarded · 1 exact duplicate")
    }));
    assert!(
        preview
            .iter()
            .all(|s| !s.contains("public-alpha") && !s.contains("βeta"))
    );
    click(&mut app, "Remove exact duplicates when saving")?;
    click(&mut app, "Save backup codes")?;
    finish(&mut app)?;
    assert!(app.recovery_editor.is_none());
    let set = &app.state.vault()?.backup_sets()[0];
    assert_eq!(set.codes().len(), 2);
    assert_eq!(
        set.codes()
            .iter()
            .map(|c| c.expose_value())
            .collect::<Vec<_>>(),
        vec!["public-alpha", "βeta"]
    );
    assert!(
        matches!(app.backup_codes.screen,backup_codes::Screen::Detail{account,..} if account==id)
    );
    let disk = std::fs::read(dir.path().join("vault.bin"))?;
    assert!(
        !disk
            .windows(b"public-alpha".len())
            .any(|s| s == b"public-alpha")
    );
    app.lock();
    app.state.repository_mut()?.unlock(pass())?;
    assert_eq!(app.state.vault()?.backup_sets()[0].codes().len(), 2);
    assert_eq!(std::fs::read_dir(dir.path())?.count(), 2); // ciphertext + session lock only
    Ok(())
}
#[test]
fn recovery_edit_confirmation_fault_retry_trash_and_portable_roundtrip() -> TestResult {
    let dir = tempfile::tempdir()?;
    let (v, id) = fixture(true)?;
    let mut app = app(v, dir.path())?;
    let old = crate::domain::recovery_edit::RecoveryTarget::capture(app.state.vault()?, id)?;
    edit(&mut app, id, "public-used-sentinel\npublic-new-sentinel")?;
    click(&mut app, "Save backup codes")?;
    assert!(!app.state.busy());
    click(&mut app, "Keep editing")?;
    old.check(app.state.vault()?)?;
    let encrypted = std::fs::read(dir.path().join("vault.bin"))?;
    std::fs::write(dir.path().join("vault.bin"), b"public-inert-write-conflict")?;
    click(&mut app, "Save backup codes")?;
    click(&mut app, "Save reviewed changes")?;
    finish(&mut app)?;
    assert!(
        app.recovery_editor
            .as_ref()
            .is_some_and(|e| e.text.contains("public-new-sentinel") && e.error.is_some())
    );
    old.check(app.state.vault()?)?;
    std::fs::write(dir.path().join("vault.bin"), encrypted)?;
    click(&mut app, "Save backup codes")?;
    click(&mut app, "Save reviewed changes")?;
    finish(&mut app)?;
    let current = &app.state.vault()?.backup_sets()[0];
    assert_eq!(
        current.codes()[0].id(),
        old.set().ok_or("set")?.codes()[0].id()
    );
    assert_eq!(current.codes()[0].used_at(), Some(Timestamp(100)));
    assert!(matches!(
        current.codes()[1].state().deletion,
        DeletionState::Trashed { .. }
    ));
    click(&mut app, "Edit backup codes")?;
    click(&mut app, "Delete backup-code set…")?;
    click(&mut app, "Keep editing")?;
    assert_eq!(
        app.state.vault()?.backup_sets()[0].state().deletion,
        DeletionState::Active
    );
    click(&mut app, "Delete backup-code set…")?;
    click(&mut app, "Move set to Trash")?;
    assert!(
        app.state.busy() || app.recovery_editor.as_ref().is_some_and(|e| e.saving),
        "delete was not submitted; form present {}, modal {:?}",
        app.recovery_editor.is_some(),
        names(&render(&mut app, 600., 16000., vec![]))
            .into_iter()
            .filter(|s| s.contains("Trash") || s.contains("Keep editing"))
            .collect::<Vec<_>>()
    );
    finish(&mut app)?;
    assert!(
        matches!(
            app.state.vault()?.backup_sets()[0].state().deletion,
            DeletionState::Trashed { .. }
        ),
        "delete completion: {:?}",
        app.recovery_editor
            .as_ref()
            .and_then(|e| e.error.as_deref())
    );
    assert_eq!(
        app.state.vault()?.accounts()[0].state().deletion,
        DeletionState::Active
    );
    app.lock();
    app.state.repository_mut()?.unlock(pass())?;
    let files = tempfile::tempdir()?;
    let path = files.path().join("public-roundtrip.authpbk");
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
    assert_eq!(
        preview.incoming_vault().backup_sets()[0].codes()[0].used_at(),
        Some(Timestamp(100))
    );
    Ok(())
}

#[test]
fn recovery_preview_ack_is_exact_input_bound_and_matches_account_creation() -> TestResult {
    let dir = tempfile::tempdir()?;
    let (v, id) = fixture(false)?;
    let mut app = app(v, dir.path())?;
    let input = "\u{feff}  public-alpha \r\n\nβeta\npublic-alpha\n";
    edit(&mut app, id, input)?;
    click(&mut app, "Remove exact duplicates when saving")?;
    assert!(
        app.recovery_editor
            .as_ref()
            .ok_or("form")?
            .save_intent()
            .is_ok()
    );
    app.recovery_editor
        .as_mut()
        .ok_or("form")?
        .text
        .push_str("public-beta");
    assert!(
        app.recovery_editor
            .as_ref()
            .ok_or("form")?
            .save_intent()
            .is_err()
    );
    click(&mut app, "Preview changes")?;
    finish(&mut app)?;
    assert!(matches!(
        app.recovery_editor.as_ref().ok_or("form")?.save_intent(),
        Err(AppError::ReviewBackupDuplicates)
    ));
    click(&mut app, "Remove exact duplicates when saving")?;
    let editor = app.recovery_editor.as_ref().ok_or("form")?;
    let standalone = match editor.save_intent()? {
        SaveIntent::RecoverySave { values, .. } => values,
        _ => return Err("intent".into()),
    };
    let mut create = ImportReview::manual();
    create.editors[0].service.push_str("Public creation");
    create.editors[0].secret.push_str("MY");
    create.backup_enabled = true;
    create.backup_text = editor.text.clone();
    create.acknowledge_backup_duplicates(true);
    let account = match create.save_intent(app.state.vault()?)? {
        SaveIntent::Create { codes, .. } => codes,
        _ => return Err("intent".into()),
    };
    assert_eq!(standalone, account);
    Ok(())
}
#[test]
fn recovery_cancel_focus_masks_widths_and_oversized_existing_set_are_safe() -> TestResult {
    let dir = tempfile::tempdir()?;
    let (v, id) = fixture(true)?;
    let mut app = app(v, dir.path())?;
    edit(&mut app, id, "public-unsaved-secret\npublic-unsaved-secret")?;
    for (width, height) in [(340., 780.), (600., 800.), (960., 900.), (340., 16000.)] {
        let out = render(&mut app, width, height, vec![]);
        assert!(
            names(&out).iter().all(
                |s| !s.contains("public-unsaved-secret") && !s.contains("public-used-sentinel")
            )
        );
        let nodes = &out
            .platform_output
            .accesskit_update
            .as_ref()
            .ok_or("tree")?
            .nodes;
        assert!(
            nodes
                .iter()
                .any(|(_, n)| n.role() == egui::Role::PasswordInput)
        );
        assert!(
            nodes
                .iter()
                .filter_map(|(_, n)| n.bounds())
                .filter(|b| b.y0 >= 0. && b.y1 <= f64::from(height))
                .all(|b| b.x0 >= -1. && b.x1 <= f64::from(width) + 1.)
        );
    }
    click(&mut app, "Cancel")?;
    click(&mut app, "Keep editing")?;
    assert!(
        app.recovery_editor
            .as_ref()
            .is_some_and(|e| e.text.contains("public-unsaved-secret"))
    );
    click(&mut app, "Cancel")?;
    click(&mut app, "Discard edits")?;
    assert!(app.recovery_editor.is_none());
    assert_eq!(app.state.vault()?.backup_sets()[0].codes().len(), 2);
    let mut large = app.state.vault()?.clone();
    large.backup_sets[0].codes = (0..300)
        .map(|i| BackupCode::new(&format!("{i:04}{}", "x".repeat(4092)), Timestamp(100)))
        .collect::<Result<Vec<_>, _>>()?;
    large.validate()?;
    let editor = RecoveryEditor::new(&large, id)?;
    assert!(editor.blocked && editor.text.is_empty() && editor.save_intent().is_err());
    let snapshot = crate::domain::recovery_edit::RecoveryTarget::capture(&large, id)?;
    app.lock();
    assert!(app.recovery_editor.is_none());
    snapshot.check(&large)?;
    assert_eq!(large.backup_sets()[0].codes().len(), 300);
    Ok(())
}
#[test]
fn recovery_lock_before_held_admitted_save_finishes_clears_form_and_never_adopts() -> TestResult {
    let dir = tempfile::tempdir()?;
    let (v, id) = fixture(false)?;
    let mut app = app(v, dir.path())?;
    edit(&mut app, id, "public-held-save-sentinel")?;
    app.recovery_editor.as_mut().ok_or("form")?.reveal = true;
    let gate = Arc::new(crate::storage::test_gate::CommitGate::default());
    let release = gate.release_on_drop();
    app.state.hold_next_save_after_admission(gate.clone());
    click(&mut app, "Save backup codes")?;
    assert!(gate.wait_entered(Duration::from_secs(10)));
    let started = Instant::now();
    app.lock();
    assert!(started.elapsed() < Duration::from_secs(1));
    let locked = names(&render(&mut app, 340., 780., vec![]));
    assert!(app.recovery_editor.is_none() && !app.state.session_active());
    assert!(
        locked
            .iter()
            .all(|s| !s.contains("public-held-save-sentinel")
                && !s.contains("Public recovery editor"))
    );
    assert!(!gate.is_released());
    drop(release);
    finish(&mut app)?;
    assert!(!app.state.is_unlocked() && app.status.is_none() && app.recovery_editor.is_none());
    app.state.repository_mut()?.unlock(pass())?;
    assert_eq!(
        app.state.vault()?.backup_sets()[0].codes()[0].expose_value(),
        "public-held-save-sentinel"
    );
    Ok(())
}

#[test]
fn recovery_dialog_defaults_to_keep_editing_escape_returns_to_actual_opener() -> TestResult {
    let dir = tempfile::tempdir()?;
    let (v, id) = fixture(true)?;
    let mut app = app(v, dir.path())?;
    edit(&mut app, id, "public-unsaved-input")?;
    click(&mut app, "Cancel")?;
    let mut out = render(&mut app, 600., 16000., vec![]);
    for _ in 0..3 {
        out = render(&mut app, 600., 16000., vec![]);
    }
    let focused = app.context.memory(|m| m.focused()).ok_or("modal focus")?;
    assert!(
        out.platform_output
            .accesskit_update
            .as_ref()
            .ok_or("tree")?
            .nodes
            .iter()
            .any(|(id, node)| id.0 == focused.value() && node.label() == Some("Keep editing"))
    );
    render(
        &mut app,
        600.,
        16000.,
        vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: Some(egui::Key::Escape),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    for _ in 0..4 {
        out = render(&mut app, 600., 16000., vec![]);
    }
    let focused = app.context.memory(|m| m.focused()).ok_or("return focus")?;
    assert!(
        out.platform_output
            .accesskit_update
            .as_ref()
            .ok_or("tree")?
            .nodes
            .iter()
            .any(|(id, node)| id.0 == focused.value() && node.label() == Some("Cancel"))
    );
    assert!(
        app.recovery_editor
            .as_ref()
            .is_some_and(|e| e.text.contains("public-unsaved-input"))
    );
    assert_eq!(app.state.vault()?.backup_sets()[0].codes().len(), 2);
    Ok(())
}

#[test]
fn recovery_direct_detail_delete_cancel_returns_to_current_control_at_normal_viewports()
-> TestResult {
    fn settle(app: &mut AuthenticatorApp, width: f32, height: f32) -> egui::FullOutput {
        for _ in 0..35 {
            render(app, width, height, vec![]);
        }
        render(app, width, height, vec![])
    }
    fn key_event(key: egui::Key, pressed: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn activate(
        app: &mut AuthenticatorApp,
        width: f32,
        height: f32,
        label: &str,
    ) -> Result<egui::Id, Box<dyn std::error::Error>> {
        for _ in 0..100 {
            let out = render(app, width, height, vec![]);
            let focused = app.context.memory(|m| m.focused());
            if out
                .platform_output
                .accesskit_update
                .as_ref()
                .is_some_and(|tree| {
                    tree.nodes.iter().any(|(id, node)| {
                        Some(id.0) == focused.map(|id| id.value())
                            && node.role() == egui::Role::Button
                            && node.label() == Some(label)
                    })
                })
            {
                let id = focused.ok_or("action focus")?;
                settle(app, width, height);
                for pressed in [true, false] {
                    render(
                        app,
                        width,
                        height,
                        vec![key_event(egui::Key::Enter, pressed)],
                    );
                }
                return Ok(id);
            }
            render(
                app,
                width,
                height,
                vec![
                    key_event(egui::Key::Tab, true),
                    key_event(egui::Key::Tab, false),
                ],
            );
        }
        Err(format!("keyboard did not reach {label}").into())
    }
    for large in [false, true] {
        let directory = tempfile::tempdir()?;
        let (mut vault, account) = fixture(true)?;
        if large {
            vault.backup_sets[0].codes = (0..300)
                .map(|i| BackupCode::new(&format!("{i:04}{}", "x".repeat(4092)), Timestamp(100)))
                .collect::<Result<Vec<_>, _>>()?;
            vault.validate()?;
        }
        let target = crate::domain::recovery_edit::RecoveryTarget::capture(&vault, account)?;
        let mut app = app(vault, directory.path())?;
        let before = std::fs::read(directory.path().join("vault.bin"))?;
        for (width, height) in [(340., 780.), (600., 800.), (960., 900.)] {
            app.backup_codes.screen = backup_codes::Screen::List;
            activate(&mut app, width, height, "View set")?;
            for cancel in 0..3 {
                let detail_opener = activate(&mut app, width, height, "Delete backup-code set…")?;
                let first = settle(&mut app, width, height);
                let focused = app
                    .context
                    .memory(|m| m.focused())
                    .ok_or("safe modal focus")?;
                assert!(
                    first
                        .platform_output
                        .accesskit_update
                        .as_ref()
                        .ok_or("tree")?
                        .nodes
                        .iter()
                        .any(|(id, node)| id.0 == focused.value()
                            && node.label() == Some("Keep editing"))
                );
                assert_eq!(app.recovery_editor.as_ref().ok_or("editor")?.blocked, large);
                match cancel {
                    0 => {
                        activate(&mut app, width, height, "Keep editing")?;
                    }
                    1 => {
                        for pressed in [true, false] {
                            render(
                                &mut app,
                                width,
                                height,
                                vec![key_event(egui::Key::Escape, pressed)],
                            );
                        }
                    }
                    _ => {
                        let pos = egui::pos2(1., 1.);
                        for pressed in [true, false] {
                            render(
                                &mut app,
                                width,
                                height,
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
                    }
                }
                let out = settle(&mut app, width, height);
                let focused = app
                    .context
                    .memory(|m| m.focused())
                    .ok_or("editor delete return focus")?;
                assert_ne!(
                    focused, detail_opener,
                    "the replaced detail button is no longer an opener"
                );
                let bounds = out
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .ok_or("tree")?
                    .nodes
                    .iter()
                    .find(|(id, node)| {
                        id.0 == focused.value()
                            && node.role() == egui::Role::Button
                            && node.label() == Some("Delete backup-code set…")
                    })
                    .and_then(|(_, node)| node.bounds())
                    .ok_or("live editor delete control")?;
                assert!(
                    bounds.x0 >= 0.
                        && bounds.x1 <= f64::from(width)
                        && bounds.y0 >= 0.
                        && bounds.y1 <= f64::from(height)
                );
                assert!(!app.state.busy());
                target.check(app.state.vault()?)?;
                assert_eq!(before, std::fs::read(directory.path().join("vault.bin"))?);
                // Return through the real editor ownership boundary before the
                // next direct detail action, retaining the same selected set.
                activate(&mut app, width, height, "Cancel")?;
                activate(&mut app, width, height, "Discard edits")?;
                settle(&mut app, width, height);
                assert!(app.recovery_editor.is_none());
                assert!(
                    matches!(app.backup_codes.screen, backup_codes::Screen::Detail { account: id, .. } if id == account)
                );
            }
        }
    }
    Ok(())
}
