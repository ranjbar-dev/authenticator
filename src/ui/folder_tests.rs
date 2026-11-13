//! Public generated records in owned TempDirs. Context input is not native proof.
use super::*;
use crate::{
    domain::{
        folder_edit::FolderTarget,
        otp::{OtpSettings, Secret},
        records::*,
    },
    error::AppError,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn pass() -> Zeroizing<String> {
    Zeroizing::new("disposable-folders-only".into())
}
fn fixture(count: usize) -> Result<Vault, AppError> {
    let mut v = Vault::new(Timestamp(100));
    for i in 0..count {
        let name = if i < 2 {
            "Public identical account".to_owned()
        } else {
            format!("Public {i:04}")
        };
        let identifier = if i == 1 {
            Some("No account identifier")
        } else {
            None
        };
        let id = v.add_account(
            AccountDraft::new(
                &name,
                None,
                identifier,
                Secret::from_base32(if i == 1 { "M4" } else { "MY" })?,
                OtpSettings::default(),
            )?,
            Timestamp(100),
        )?;
        let mut code = BackupCode::new(&format!("public-folder-code-{i}"), Timestamp(100))?;
        code.set_used(true, Timestamp(100))?;
        v.add_backup_set(id, vec![code], Timestamp(100))?;
    }
    Ok(v)
}
fn app(v: Vault, path: &std::path::Path) -> Result<AuthenticatorApp, Box<dyn std::error::Error>> {
    let mut store = VaultStore::open_directory(path)?;
    store.create_with_parameters(
        pass(),
        v,
        crate::storage::KdfParameters {
            memory_kib: 65_536,
            iterations: 3,
            lanes: 1,
        },
    )?;
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let mut app = AuthenticatorApp::with_context(ctx, Ok(store), 0);
    app.clipboard = crate::platform::clipboard::test_fixture().0;
    app.state.repository_mut()?.unlock(pass())?;
    Ok(app)
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
    for _ in 0..8 {
        render(app, size, vec![]);
    }
    render(app, size, vec![])
}
fn names(out: &egui::FullOutput) -> Vec<String> {
    out.platform_output
        .accesskit_update
        .as_ref()
        .into_iter()
        .flat_map(|u| &u.nodes)
        .flat_map(|(_, n)| {
            [n.label(), n.value()]
                .into_iter()
                .flatten()
                .map(str::to_owned)
        })
        .collect()
}
fn node(out: &egui::FullOutput, label: &str) -> Option<(u64, egui::Rect)> {
    out.platform_output
        .accesskit_update
        .as_ref()?
        .nodes
        .iter()
        .find(|(_, n)| {
            matches!(
                n.role(),
                egui::Role::Button
                    | egui::Role::RadioButton
                    | egui::Role::CheckBox
                    | egui::Role::TextInput
                    | egui::Role::ComboBox
            ) && n.label() == Some(label)
        })
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
fn click(app: &mut AuthenticatorApp, size: [f32; 2], label: &str) -> TestResult {
    let mut out = settle(app, size);
    for _ in 0..36 {
        if let Some((_, rect)) = node(&out, label)
            && rect.top() > 2.
            && rect.bottom() < size[1] - 4.
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
        let delta = node(&out, label).map_or(-320., |(_, r)| {
            (size[1] / 2. - r.center().y).clamp(-2400., 2400.)
        });
        wheel(app, size, delta);
        out = settle(app, size);
    }
    Err(format!("control unavailable: {label}; {:?}", node(&out, label)).into())
}
fn key(app: &mut AuthenticatorApp, size: [f32; 2], key: egui::Key) {
    for pressed in [true, false] {
        render(
            app,
            size,
            vec![egui::Event::Key {
                key,
                physical_key: Some(key),
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
    }
}
fn tab_to(app: &mut AuthenticatorApp, size: [f32; 2], label: &str) -> TestResult {
    let target = node(&settle(app, size), label)
        .ok_or("keyboard control missing")?
        .0;
    for _ in 0..180 {
        if app
            .context
            .memory(|m| m.focused().is_some_and(|id| id.value() == target))
        {
            return Ok(());
        }
        key(app, size, egui::Key::Tab);
    }
    Err(format!("Tab cannot reach {label}").into())
}
fn finish(app: &mut AuthenticatorApp, size: [f32; 2]) -> TestResult {
    let until = Instant::now() + Duration::from_secs(45);
    while app.state.busy() {
        if Instant::now() > until {
            return Err("folder save timeout".into());
        }
        render(app, size, vec![]);
        std::thread::sleep(Duration::from_millis(5));
    }
    settle(app, size);
    Ok(())
}
fn open(app: &mut AuthenticatorApp, id: Option<FolderId>) -> TestResult {
    settle(app, [600., 800.]);
    let mut manager = FolderManager::new();
    manager.editor = Some(folders::Editor::new(app.state.vault()?, id)?);
    app.folders = Some(manager);
    Ok(())
}
fn submit(app: &mut AuthenticatorApp, trash: bool) -> TestResult {
    let mut manager = app.folders.take().ok_or("manager")?;
    app.start_folder_save(&mut manager, trash);
    assert!(manager.editor.as_ref().is_some_and(|e| e.saving));
    app.folders = Some(manager);
    Ok(())
}
#[test]
fn folders_pointer_keyboard_widths_multiselect_discard_and_focus() -> TestResult {
    let dir = tempfile::tempdir()?;
    let mut app = app(fixture(23)?, dir.path())?;
    let size = [340., 780.];
    click(&mut app, size, "Manage folders")?;
    click(&mut app, size, "Create folder")?;
    settle(&mut app, size);
    render(
        &mut app,
        size,
        vec![egui::Event::Text("Public Unicode é folder".into())],
    );
    assert_eq!(
        app.folders
            .as_ref()
            .and_then(|m| m.editor.as_ref())
            .ok_or("editor")?
            .name
            .as_str(),
        "Public Unicode é folder"
    );
    click(&mut app, size, "Folder color: Green")?;
    let ids: Vec<_> = app
        .state
        .vault()?
        .accounts()
        .iter()
        .map(Account::id)
        .collect();
    // Six radiobutton states have a named selected cue; each row has a unique AX name.
    for size in [[340., 780.], [600., 800.], [960., 900.]] {
        let out = settle(&mut app, size);
        let nodes = &out
            .platform_output
            .accesskit_update
            .as_ref()
            .ok_or("tree")?
            .nodes;
        assert_eq!(
            nodes
                .iter()
                .filter(|(_, n)| n.role() == egui::Role::RadioButton)
                .count(),
            6
        );
        assert!(
            names(&out)
                .iter()
                .all(|s| !s.contains("public-folder-code-") && !s.contains("GEZDGNB"))
        );
        assert!(
            nodes
                .iter()
                .filter(|(_, n)| matches!(
                    n.role(),
                    egui::Role::Button
                        | egui::Role::RadioButton
                        | egui::Role::CheckBox
                        | egui::Role::TextInput
                ))
                .filter_map(|(_, n)| n.bounds())
                .all(|b| b.x0 >= -1. && b.x1 <= f64::from(size[0]) + 1.),
            "horizontal control overflow"
        );
    }
    click(&mut app, size, "Next")?;
    let member = format!(
        "Public identical account · No account identifier · Account ID: {}",
        ids[0].as_uuid()
    );
    click(&mut app, size, &member)?;
    assert!(
        app.folders
            .as_ref()
            .and_then(|m| m.editor.as_ref())
            .ok_or("editor")?
            .selected
            .contains(&ids[0])
    );
    // Keyboard activation of a real checkbox also works; no element-click shortcut.
    let other = format!(
        "Public identical account · No account identifier · Account ID: {}",
        ids[1].as_uuid()
    );
    tab_to(&mut app, size, &other)?;
    key(&mut app, size, egui::Key::Space);
    assert!(
        app.folders
            .as_ref()
            .and_then(|m| m.editor.as_ref())
            .ok_or("editor")?
            .selected
            .contains(&ids[1])
    );
    click(&mut app, size, "Save folder")?;
    finish(&mut app, size)?;
    assert!(app.folders.as_ref().is_some_and(|m| m.editor.is_none()));
    assert_eq!(app.state.vault()?.folders()[0].color(), FolderColor::Green);
    assert_eq!(app.state.vault()?.memberships().len(), 2);
    let out = settle(&mut app, size);
    let back = node(&out, "Back to 2FA Codes").ok_or("live manager Back")?;
    assert!(back.1.top() >= 0. && back.1.bottom() < size[1]);
    assert_eq!(
        app.context.memory(|m| m.focused().map(|id| id.value())),
        Some(back.0)
    );
    click(&mut app, size, "Edit folder")?;
    app.folders
        .as_mut()
        .and_then(|m| m.editor.as_mut())
        .ok_or("editor")?
        .name
        .push_str(" unsaved");
    click(&mut app, size, "Cancel")?;
    let out = settle(&mut app, size);
    let keep = node(&out, "Keep editing").ok_or("safe default")?;
    assert_eq!(
        app.context.memory(|m| m.focused().map(|id| id.value())),
        Some(keep.0)
    );
    key(&mut app, size, egui::Key::Escape);
    assert!(
        app.folders
            .as_ref()
            .and_then(|m| m.editor.as_ref())
            .is_some_and(|e| e.name.ends_with(" unsaved"))
    );
    click(&mut app, size, "Back to folders")?;
    click(&mut app, size, "Discard edits")?;
    assert_eq!(
        app.state.vault()?.folders()[0].name(),
        "Public Unicode é folder"
    );
    click(&mut app, size, "Back to 2FA Codes")?;
    let out = settle(&mut app, size);
    let manage = node(&out, "Manage folders").ok_or("manage focus")?;
    assert_eq!(
        app.context.memory(|m| m.focused().map(|id| id.value())),
        Some(manage.0)
    );
    assert!(manage.1.top() >= 0. && manage.1.bottom() <= size[1]);
    Ok(())
}
#[test]
fn folders_dirty_codes_preferences_atomic_save_trash_failure_retry_and_encrypted_reopen()
-> TestResult {
    let dir = tempfile::tempdir()?;
    let mut app = app(fixture(3)?, dir.path())?;
    let size = [600., 800.];
    settle(&mut app, size);
    let ids: Vec<_> = app
        .state
        .vault()?
        .accounts()
        .iter()
        .map(Account::id)
        .collect();
    app.codes_preferences.as_mut().ok_or("prefs")?.query =
        Zeroizing::new("latest dirty intent".into());
    app.codes_preferences.as_mut().ok_or("prefs")?.sort = SortOrder::AccountDescending;
    app.codes_preferences.as_mut().ok_or("prefs")?.view = ViewMode::Grid;
    app.state.query = "latest dirty intent".into();
    app.preferences_changed();
    open(&mut app, None)?;
    let editor = app
        .folders
        .as_mut()
        .and_then(|m| m.editor.as_mut())
        .ok_or("editor")?;
    editor.name.push_str("Public durable folder");
    editor.selected.extend([ids[0], ids[1]]);
    let old_bytes = std::fs::read(dir.path().join("vault.bin"))?;
    std::fs::write(dir.path().join("vault.bin"), b"owned public write conflict")?;
    submit(&mut app, false)?;
    finish(&mut app, size)?;
    assert!(app.state.vault()?.folders().is_empty());
    assert!(app.preferences_dirty.is_some());
    assert!(
        app.folders
            .as_ref()
            .and_then(|m| m.editor.as_ref())
            .is_some_and(|e| e.name.as_str() == "Public durable folder"
                && e.selected.len() == 2
                && e.error.is_some()
                && !e.saving)
    );
    assert_eq!(
        std::fs::read(dir.path().join("vault.bin"))?,
        b"owned public write conflict"
    );
    std::fs::write(dir.path().join("vault.bin"), old_bytes)?;
    submit(&mut app, false)?;
    finish(&mut app, size)?;
    let folder = app.state.vault()?.folders()[0].id();
    assert_eq!(
        app.state.vault()?.preferences().codes().query.as_str(),
        "latest dirty intent"
    );
    assert_eq!(
        app.state.vault()?.preferences().codes().sort,
        SortOrder::AccountDescending
    );
    assert_eq!(
        app.state.vault()?.preferences().codes().view,
        ViewMode::Grid
    );
    // Queue a preference save, change intent while it is pending, then rename.
    let mut older = app.codes_preferences.clone().ok_or("prefs")?;
    older.query = Zeroizing::new("older queued".into());
    assert!(app.begin_mutation(SaveIntent::CodesPreferences(older)));
    app.codes_preferences.as_mut().ok_or("prefs")?.query =
        Zeroizing::new("newer while pending".into());
    app.codes_preferences.as_mut().ok_or("prefs")?.folder = FolderFilter::Folder(folder);
    app.state.query = "newer while pending".into();
    app.preferences_changed();
    finish(&mut app, size)?;
    open(&mut app, Some(folder))?;
    app.folders
        .as_mut()
        .and_then(|m| m.editor.as_mut())
        .ok_or("editor")?
        .name = Zeroizing::new("Renamed durable".into());
    submit(&mut app, false)?;
    finish(&mut app, size)?;
    assert_eq!(
        app.state.vault()?.preferences().codes().query.as_str(),
        "newer while pending"
    );
    assert_eq!(
        app.state.vault()?.preferences().codes().folder,
        FolderFilter::Folder(folder)
    );
    // Latest query/sort/view and current folder must be normalized in one transaction.
    app.codes_preferences.as_mut().ok_or("prefs")?.query = Zeroizing::new("trash latest".into());
    app.state.query = "trash latest".into();
    app.preferences_changed();
    open(&mut app, Some(folder))?;
    let account_snapshot = crate::storage::codec::encode(app.state.vault()?)?;
    submit(&mut app, true)?;
    finish(&mut app, size)?;
    assert_eq!(
        app.codes_preferences.as_ref().ok_or("cache")?.folder,
        FolderFilter::All
    );
    assert_eq!(
        app.state.vault()?.preferences().codes().folder,
        FolderFilter::All
    );
    assert_eq!(app.state.query, "trash latest");
    assert!(app.preferences_dirty.is_none());
    let before = crate::storage::codec::decode(&account_snapshot)?;
    for (a, old) in app.state.vault()?.accounts().iter().zip(before.accounts()) {
        assert_eq!((a.id(), a.state()), (old.id(), old.state()));
        assert!(a.credential().duplicate_criteria(old.credential()).0);
    }
    assert_eq!(
        app.state.vault()?.backup_sets()[0].codes()[0].used_at(),
        Some(Timestamp(100))
    );
    assert_eq!(app.state.vault()?.memberships().len(), 2);
    let bytes = std::fs::read(dir.path().join("vault.bin"))?;
    for text in ["Renamed durable", "trash latest", "public-folder-code-0"] {
        assert!(!bytes.windows(text.len()).any(|w| w == text.as_bytes()));
    }
    app.lock();
    drop(app);
    let mut store = VaultStore::open_directory(dir.path())?;
    store.unlock(pass())?;
    assert_eq!(store.vault()?.folders()[0].name(), "Renamed durable");
    assert_eq!(
        store.vault()?.preferences().codes().folder,
        FolderFilter::All
    );
    assert_eq!(
        store.vault()?.preferences().codes().query.as_str(),
        "trash latest"
    );
    assert_eq!(store.vault()?.preferences().codes().view, ViewMode::Grid);
    store.vault()?.validate()?;
    Ok(())
}
#[test]
fn folders_lock_clears_form_before_held_admitted_write_and_never_late_adopts() -> TestResult {
    let dir = tempfile::tempdir()?;
    let mut app = app(fixture(2)?, dir.path())?;
    open(&mut app, None)?;
    app.folders
        .as_mut()
        .and_then(|m| m.editor.as_mut())
        .ok_or("editor")?
        .name
        .push_str("Public held-folder metadata");
    let gate = Arc::new(crate::storage::test_gate::CommitGate::default());
    let release = gate.release_on_drop();
    app.state.hold_next_save_after_admission(gate.clone());
    submit(&mut app, false)?;
    assert!(gate.wait_entered(Duration::from_secs(10)));
    let start = Instant::now();
    app.lock();
    assert!(start.elapsed() < Duration::from_secs(1));
    let out = settle(&mut app, [340., 780.]);
    assert!(app.folders.is_none() && !app.state.session_active());
    assert!(
        names(&out)
            .iter()
            .all(|s| !s.contains("Public held-folder") && !s.contains("Folder name"))
    );
    assert!(!gate.is_released());
    drop(release);
    finish(&mut app, [340., 780.])?;
    assert!(!app.state.is_unlocked() && app.status.is_none() && app.folders.is_none());
    app.state.repository_mut()?.unlock(pass())?;
    assert_eq!(
        app.state.vault()?.folders()[0].name(),
        "Public held-folder metadata"
    );
    Ok(())
}
#[test]
fn folders_backup_repeat_import_retains_ids_trash_membership_states_and_conflict_policy()
-> TestResult {
    let dir = tempfile::tempdir()?;
    let destination_dir = tempfile::tempdir()?;
    let mut dest = VaultStore::open_directory(destination_dir.path())?;
    dest.create_with_parameters(
        pass(),
        Vault::new(Timestamp(100)),
        crate::storage::KdfParameters {
            memory_kib: 65_536,
            iterations: 3,
            lanes: 1,
        },
    )?;
    let mut vault = fixture(2)?;
    let id = vault.accounts()[0].id();
    let target = FolderTarget::capture(&vault, None)?;
    let f = vault.save_folder(
        &target,
        crate::domain::folder_edit::FolderUpdate {
            name: "Unfiled",
            color: FolderColor::Purple,
            selected: &std::collections::BTreeSet::from([id]),
            duplicate_consent: false,
        },
        Timestamp(100),
    )?;
    let link = vault.memberships()[0].id();
    let mut source = VaultStore::open_directory(dir.path().join("source"))?;
    source.create_with_parameters(
        pass(),
        vault,
        crate::storage::KdfParameters {
            memory_kib: 65_536,
            iterations: 3,
            lanes: 1,
        },
    )?;
    let path = dir.path().join("public-folders.vault");
    let cancel = crate::storage::OperationCancellation::default();
    crate::backup::export(
        &source,
        crate::backup::Destination::create_new(&path)?,
        pass(),
        &cancel,
    )?;
    for _ in 0..2 {
        let preview = crate::backup::preview(&dest, &path, pass(), &cancel)?;
        let plan = preview.plan(crate::backup::ImportOptions::default())?;
        plan.apply(&mut dest, &cancel)?;
        dest.vault()?.validate()?;
        assert_eq!(dest.vault()?.accounts().len(), 2);
        assert_eq!(dest.vault()?.folders().len(), 1);
        assert_eq!(dest.vault()?.memberships().len(), 1);
        assert_eq!(dest.vault()?.folders()[0].id(), f);
        assert_eq!(dest.vault()?.memberships()[0].id(), link);
    }
    let t = FolderTarget::capture(dest.vault()?, Some(f))?;
    dest.transaction(|v| v.trash_folder(&t, Timestamp(1)))?;
    let preview = crate::backup::preview(&dest, &path, pass(), &cancel)?;
    preview
        .plan(crate::backup::ImportOptions::default())?
        .apply(&mut dest, &cancel)?;
    assert!(matches!(
        dest.vault()?.folders()[0].state().deletion,
        DeletionState::Trashed { .. }
    ));
    assert_eq!(
        dest.vault()?.backup_sets()[0].codes()[0].used_at(),
        Some(Timestamp(100))
    );
    assert_eq!(
        dest.vault()?.memberships()[0].state().deletion,
        DeletionState::Active
    );
    Ok(())
}
