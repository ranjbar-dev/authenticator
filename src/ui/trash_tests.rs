//! Generated public records; Context events are not native Windows input proof.
use super::*;
use crate::domain::{
    otp::{OtpSettings, Secret},
    records::*,
    trash::{self as domain_trash, ItemId},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn trash_close_during_admitted_delete_clears_presentation_before_commit_and_never_late_adopts()
-> TestResult {
    let dir = tempfile::tempdir()?;
    let mut v = fixture(2)?;
    let account = v.accounts[0].id();
    let other = v.accounts[1].id();
    own_trash(&mut v.accounts[0].state);
    let mut app = app(v, dir.path())?;
    let size = [600., 800.];
    open(&mut app, size)?;
    click(
        &mut app,
        size,
        &action(ItemId::Account(account), "Permanently delete"),
    )?;
    let bytes = std::fs::read(dir.path().join("vault.bin"))?;
    let gate = Arc::new(crate::storage::test_gate::CommitGate::default());
    let release = gate.release_on_drop();
    app.state.hold_next_save_after_admission(gate.clone());
    click(&mut app, size, "Permanently delete reviewed records")?;
    assert!(gate.wait_entered(Duration::from_secs(10)));
    let started = Instant::now();
    let output = request_close(&mut app, size);
    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(!gate.is_released());
    assert!(app.trash.is_none() && !app.state.session_active());
    assert!(
        labels(&output)
            .iter()
            .all(|s| !s.contains("Public identical") && !s.contains("Permanently delete reviewed"))
    );
    no_secrets(&output);
    assert_eq!(std::fs::read(dir.path().join("vault.bin"))?, bytes);
    drop(release);
    finish(&mut app, size)?;
    assert!(!app.state.is_unlocked() && app.status.is_none());
    app.state.repository_mut()?.unlock(pass())?;
    assert!(
        !app.state
            .vault()?
            .accounts()
            .iter()
            .any(|a| a.id() == account)
    );
    assert_eq!(app.state.vault()?.accounts()[0].id(), other);
    assert_eq!(
        app.state.vault()?.backup_sets()[0].codes()[0].used_at(),
        Some(Timestamp(100))
    );
    app.state.vault()?.validate()?;
    Ok(())
}

#[test]
fn trash_portable_all_kinds_restore_and_failed_successful_reused_id_import_invalidation()
-> TestResult {
    let dir = tempfile::tempdir()?;
    eprintln!(
        "owned public Task18 portable fixture: {}",
        dir.path().display()
    );
    let mut vault = fixture(2)?;
    let account = vault.accounts[0].id();
    let folder = vault.folders[0].id();
    let member = vault.memberships[0].id();
    let set = vault.backup_sets[0].id();
    let code = vault.backup_sets[0].codes[0].id();
    own_trash(&mut vault.accounts[0].state);
    own_trash(&mut vault.folders[0].state);
    own_trash(&mut vault.memberships[0].state);
    own_trash(&mut vault.backup_sets[0].state);
    own_trash(&mut vault.backup_sets[0].codes[0].state);
    let original = vault.clone();
    let mut source = VaultStore::open_directory(dir.path().join("source"))?;
    source.create_with_parameters(
        Zeroizing::new("disposable-source-trash-pass".into()),
        vault,
        crate::storage::KdfParameters {
            memory_kib: 65_536,
            iterations: 3,
            lanes: 1,
        },
    )?;
    let backup_pass = || Zeroizing::new("disposable-portable-trash-only".into());
    let path = dir.path().join("public-trash.authpbk");
    let cancel = crate::storage::OperationCancellation::default();
    crate::backup::export(
        &source,
        crate::backup::Destination::create_new(&path)?,
        backup_pass(),
        &cancel,
    )?;
    let backup_bytes = std::fs::read(&path)?;
    let mut destination = VaultStore::open_directory(dir.path().join("destination"))?;
    destination.create_with_parameters(
        pass(),
        Vault::new(Timestamp(100)),
        crate::storage::KdfParameters {
            memory_kib: 65_536,
            iterations: 3,
            lanes: 1,
        },
    )?;
    let preview = crate::backup::preview(&destination, &path, backup_pass(), &cancel)?;
    preview
        .plan(crate::backup::ImportOptions::default())?
        .apply(&mut destination, &cancel)?;
    same_business(destination.vault()?, &original);
    assert_eq!(
        domain_trash::items(destination.vault()?),
        domain_trash::items(&original)
    );
    let mut app = from_store(destination)?;
    let size = [600., 800.];
    open(&mut app, size)?;
    for id in [
        ItemId::Account(account),
        ItemId::Folder(folder),
        ItemId::Set(set),
        ItemId::Code(set, code),
        ItemId::Membership(member),
    ] {
        click(&mut app, size, &action(id, "Restore"))?;
        click(&mut app, size, "Restore record")?;
        finish(&mut app, size)?;
    }
    assert!(domain_trash::items(app.state.vault()?).is_empty());
    assert_eq!(app.state.vault()?.accounts()[0].id(), account);
    assert!(
        app.state.vault()?.accounts()[0]
            .credential()
            .duplicate_criteria(original.accounts()[0].credential())
            .0
    );
    assert_eq!(
        app.state.vault()?.backup_sets()[0].codes()[0].used_at(),
        Some(Timestamp(100))
    );
    assert_eq!(app.state.vault()?.memberships()[0].id(), member);
    // Keep a real reviewed presentation while the import result boundary is
    // exercised directly. Navigation normally closes this subpage beforehand.
    app.state.repository_mut()?.transaction(|v| {
        own_trash(&mut v.backup_sets[0].codes[0].state);
        Ok(())
    })?;
    click(&mut app, size, &action(ItemId::Code(set, code), "Restore"))?;
    let reviewed = app
        .trash
        .as_ref()
        .and_then(|s| s.review.as_ref())
        .ok_or("review")?
        .review
        .clone();
    for fail in [true, false] {
        let preview =
            crate::backup::preview(app.state.repository_mut()?, &path, backup_pass(), &cancel)?;
        let plan = preview.plan(crate::backup::ImportOptions {
            mode: crate::backup::ImportMode::Replace,
            ..Default::default()
        })?;
        if fail {
            std::fs::write(&path, b"owned changed backup source")?;
        }
        let mut form = ImportForm::new(egui::Id::unique("trash-import-invalidation"));
        form.plan = Some(plan);
        app.apply_backup_plan(&mut form);
        app.backup_import = Some(form);
        finish(&mut app, size)?;
        if fail {
            assert!(
                app.backup_import
                    .as_ref()
                    .is_some_and(|f| f.error.is_some())
            );
            assert!(app.trash.as_ref().is_some_and(|s| s.review.is_some()));
            reviewed.check(app.state.vault()?)?;
            app.backup_import = None;
            std::fs::write(&path, &backup_bytes)?;
        } else {
            assert!(app.trash.is_none() && app.backup_import.is_none());
            assert_eq!(app.state.route, Route::Codes);
            same_business(app.state.vault()?, &original);
            assert_eq!(domain_trash::items(app.state.vault()?).len(), 5);
        }
    }
    app.state.vault()?.validate()?;
    Ok(())
}
fn pass() -> Zeroizing<String> {
    Zeroizing::new("disposable-trash-fixture".into())
}
fn fixture(accounts: usize) -> Result<Vault, crate::error::AppError> {
    let mut v = Vault::new(Timestamp(100));
    let folder = v.add_folder(
        "Public \"quoted\" folder",
        FolderColor::Green,
        Timestamp(100),
    )?;
    for i in 0..accounts {
        let name = if i < 2 {
            "Public identical".into()
        } else {
            format!("Public account {i:03}")
        };
        let identifier = if i == 1 {
            Some("No account identifier")
        } else {
            None
        };
        let seed = data_encoding::BASE32_NOPAD.encode(&[0x41, i as u8]);
        let id = v.add_account(
            AccountDraft::new(
                &name,
                None,
                identifier,
                Secret::from_base32(&seed)?,
                OtpSettings::default(),
            )?,
            Timestamp(100),
        )?;
        v.add_membership(folder, id, Timestamp(100))?;
        let mut code = BackupCode::new(&format!("public-trash-code-{i}-used"), Timestamp(100))?;
        code.set_used(true, Timestamp(100))?;
        v.add_backup_set(
            id,
            vec![
                code,
                BackupCode::new(&format!("public-trash-code-{i}-available"), Timestamp(100))?,
            ],
            Timestamp(100),
        )?;
    }
    Ok(v)
}
fn own_trash(state: &mut RecordState) {
    state.deletion = DeletionState::Trashed { at: Timestamp(100) };
}
fn app(
    vault: Vault,
    path: &std::path::Path,
) -> Result<AuthenticatorApp, Box<dyn std::error::Error>> {
    let mut store = VaultStore::open_directory(path)?;
    store.create_with_parameters(
        pass(),
        vault,
        crate::storage::KdfParameters {
            memory_kib: 65_536,
            iterations: 3,
            lanes: 1,
        },
    )?;
    from_store(store)
}
fn from_store(store: VaultStore) -> Result<AuthenticatorApp, Box<dyn std::error::Error>> {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let mut app = AuthenticatorApp::with_context(ctx, Ok(store), 0);
    app.clipboard = crate::platform::clipboard::test_fixture().0;
    app.state.repository_mut()?.unlock(pass())?;
    Ok(app)
}
fn frame(app: &mut AuthenticatorApp, size: [f32; 2], events: Vec<egui::Event>) -> egui::FullOutput {
    let ctx = app.context.clone();
    let mut output = ctx.run_ui(
        egui::RawInput {
            time: Some(ctx.input(|i| i.time) + 1. / 60.),
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size.into())),
            events,
            ..Default::default()
        },
        |root| app.show(root),
    );
    output.textures_delta.clear();
    output
}
fn settle(app: &mut AuthenticatorApp, size: [f32; 2]) -> egui::FullOutput {
    for _ in 0..24 {
        frame(app, size, vec![]);
    }
    frame(app, size, vec![])
}
fn labels(out: &egui::FullOutput) -> Vec<String> {
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
fn control(out: &egui::FullOutput, name: &str) -> Option<(u64, egui::Rect)> {
    out.platform_output
        .accesskit_update
        .as_ref()?
        .nodes
        .iter()
        .find_map(|(id, n)| {
            if n.label() != Some(name)
                || !matches!(
                    n.role(),
                    egui::Role::Button | egui::Role::Tab | egui::Role::CheckBox
                )
            {
                return None;
            }
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
fn painted_visible(out: &egui::FullOutput, rect: egui::Rect) -> bool {
    fn matching(shape: &egui::epaint::Shape, rect: egui::Rect, clip: egui::Rect) -> bool {
        match shape {
            egui::epaint::Shape::Vec(shapes) => shapes.iter().any(|s| matching(s, rect, clip)),
            egui::epaint::Shape::Rect(shape) => {
                shape.rect.center().distance(rect.center()) < 1.
                    && (shape.rect.width() - rect.width()).abs() < 4.
                    && (shape.rect.height() - rect.height()).abs() < 4.
                    && clip.contains_rect(rect.shrink(1.))
            }
            _ => false,
        }
    }
    out.shapes
        .iter()
        .any(|s| matching(&s.shape, rect, s.clip_rect))
}
fn click_unsettled(app: &mut AuthenticatorApp, size: [f32; 2], name: &str) -> TestResult {
    let mut output = settle(app, size);
    for _ in 0..48 {
        if let Some((_, rect)) = control(&output, name)
            && rect.min.y >= 0.
            && rect.max.y < size[1] - 8.
            && painted_visible(&output, rect)
            // AX bounds may include content below a nested modal viewport.
            // Require a hit on the modal itself, then wheel inside its body
            // until clipped actions become real pointer targets.
            && app.context.memory(|m| m.top_modal_layer()).is_none_or(|layer| app.context.layer_id_at(rect.center()) == Some(layer))
        {
            for pressed in [true, false] {
                frame(
                    app,
                    size,
                    vec![
                        egui::Event::PointerMoved(rect.center()),
                        egui::Event::PointerButton {
                            pos: rect.center(),
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                );
            }
            return Ok(());
        }
        let delta = control(&output, name).map_or(-360., |(_, r)| {
            (size[1] / 2. - r.center().y).clamp(-2400., 2400.)
        });
        frame(
            app,
            size,
            vec![
                egui::Event::PointerMoved(egui::pos2(size[0] / 2., size[1] / 2.)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta),
                    modifiers: egui::Modifiers::NONE,
                    phase: egui::TouchPhase::Move,
                    source: egui::MouseWheelSource::Unknown,
                },
            ],
        );
        output = settle(app, size);
    }
    Err(format!(
        "unavailable control {name}: {:?}",
        control(&output, name).map(|(_, r)| r)
    )
    .into())
}
fn click(app: &mut AuthenticatorApp, size: [f32; 2], name: &str) -> TestResult {
    click_unsettled(app, size, name)?;
    if app.trash.as_ref().is_some_and(|s| s.preparing) {
        finish(app, size)?;
    }
    Ok(())
}
fn key(app: &mut AuthenticatorApp, size: [f32; 2], key: egui::Key) {
    for pressed in [true, false] {
        frame(
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
fn tab_to(app: &mut AuthenticatorApp, size: [f32; 2], name: &str) -> TestResult {
    let id = control(&settle(app, size), name)
        .ok_or("missing keyboard target")?
        .0;
    for _ in 0..160 {
        if app
            .context
            .memory(|m| m.focused().is_some_and(|focus| focus.value() == id))
        {
            return Ok(());
        }
        key(app, size, egui::Key::Tab);
    }
    Err(format!("unreachable keyboard target {name}").into())
}
fn focused(
    app: &AuthenticatorApp,
    out: &egui::FullOutput,
    name: &str,
    size: [f32; 2],
) -> TestResult {
    let (id, rect) = control(out, name).ok_or("focused control missing")?;
    assert_eq!(
        app.context.memory(|m| m.focused().map(|id| id.value())),
        Some(id)
    );
    assert!(
        rect.min.x >= 0. && rect.max.x <= size[0] && rect.min.y >= 0. && rect.max.y <= size[1],
        "focus must be visible: {rect:?}"
    );
    assert!(
        painted_visible(out, rect),
        "focus must be inside its painted clip"
    );
    Ok(())
}
fn no_secrets(out: &egui::FullOutput) {
    assert!(
        labels(out)
            .iter()
            .all(|s| !s.contains("public-trash-code-") && !s.contains("GEZDGNBV"))
    );
    assert!(
        out.platform_output
            .commands
            .iter()
            .all(|c| !matches!(c, egui::OutputCommand::CopyText(_)))
    );
}
fn open(app: &mut AuthenticatorApp, size: [f32; 2]) -> TestResult {
    app.state.navigate(Route::Settings);
    let output = settle(app, size);
    let label = labels(&output)
        .into_iter()
        .find(|s| s.starts_with("Trash · "))
        .ok_or("Trash Settings action")?;
    click(app, size, &label)?;
    assert!(app.trash.is_some());
    Ok(())
}
fn action(id: ItemId, verb: &str) -> String {
    format!("{verb} {} · ID: {}", id.kind(), id.uuid())
}
fn finish(app: &mut AuthenticatorApp, size: [f32; 2]) -> TestResult {
    let deadline = Instant::now() + Duration::from_secs(45);
    while app.state.busy() {
        if Instant::now() >= deadline {
            return Err("Trash worker timeout".into());
        }
        frame(app, size, vec![]);
        std::thread::sleep(Duration::from_millis(5));
    }
    settle(app, size);
    Ok(())
}

fn request_close(app: &mut AuthenticatorApp, size: [f32; 2]) -> egui::FullOutput {
    let ctx = app.context.clone();
    let mut input = egui::RawInput {
        time: Some(ctx.input(|i| i.time) + 1. / 60.),
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size.into())),
        ..Default::default()
    };
    input
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .events
        .push(egui::ViewportEvent::Close);
    let mut output = ctx.run_ui(input, |root| app.show(root));
    output.textures_delta.clear();
    output
}
fn finish_close(app: &mut AuthenticatorApp, size: [f32; 2]) -> TestResult {
    let deadline = Instant::now() + Duration::from_secs(45);
    while !app.exit_approved {
        if Instant::now() >= deadline {
            return Err("display flush did not finish closing".into());
        }
        frame(app, size, vec![]);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(!app.state.is_unlocked());
    Ok(())
}
fn same_business(now: &Vault, original: &Vault) {
    assert_eq!(now.accounts().len(), original.accounts().len());
    for (a, b) in now.accounts().iter().zip(original.accounts()) {
        assert_eq!((a.id(), a.state(), a.icon()), (b.id(), b.state(), b.icon()));
        assert!(a.credential().duplicate_criteria(b.credential()).0);
        assert_eq!(
            (
                a.credential().service(),
                a.credential().website(),
                a.credential().account_identifier()
            ),
            (
                b.credential().service(),
                b.credential().website(),
                b.credential().account_identifier()
            )
        );
    }
    assert_eq!(now.folders().len(), original.folders().len());
    for (a, b) in now.folders().iter().zip(original.folders()) {
        assert_eq!(
            (a.id(), a.name(), a.color(), a.state()),
            (b.id(), b.name(), b.color(), b.state())
        );
    }
    assert_eq!(now.memberships().len(), original.memberships().len());
    for (a, b) in now.memberships().iter().zip(original.memberships()) {
        assert_eq!(
            (a.id(), a.folder_id(), a.account_id(), a.state()),
            (b.id(), b.folder_id(), b.account_id(), b.state())
        );
    }
    assert_eq!(now.backup_sets().len(), original.backup_sets().len());
    for (a, b) in now.backup_sets().iter().zip(original.backup_sets()) {
        assert_eq!(
            (a.id(), a.account_id(), a.state(), a.codes().len()),
            (b.id(), b.account_id(), b.state(), b.codes().len())
        );
        for (a, b) in a.codes().iter().zip(b.codes()) {
            assert_eq!(
                (a.id(), a.state(), a.used_at(), a.expose_value()),
                (b.id(), b.state(), b.used_at(), b.expose_value())
            );
        }
    }
}

fn install_close_view(app: &mut AuthenticatorApp, variant: usize, size: [f32; 2]) -> TestResult {
    let account = app
        .state
        .vault()?
        .accounts()
        .iter()
        .find(|a| a.state().deletion == DeletionState::Active)
        .ok_or("active close fixture")?
        .id();
    let set = app
        .state
        .vault()?
        .backup_sets()
        .iter()
        .find(|s| s.account_id() == account)
        .ok_or("set")?
        .id();
    match variant {
        0 => {
            app.state.navigate(Route::Codes);
            settle(app, size);
            click(app, size, "Manage folders")?;
            click(app, size, "Create folder")?;
            app.folders
                .as_mut()
                .and_then(|m| m.editor.as_mut())
                .ok_or("folder editor")?
                .name
                .push_str("public-close-unsaved-folder");
        }
        1 => {
            open(app, size)?;
            let id = domain_trash::items(app.state.vault()?)
                .first()
                .ok_or("Trash close fixture")?
                .id;
            click(app, size, &action(id, "Restore"))?;
            finish(app, size)?;
            assert!(app.trash.as_ref().is_some_and(|s| s.review.is_some()));
        }
        2 => {
            app.state.navigate(Route::BackupCodes);
            app.recovery_editor = Some(RecoveryEditor::new(app.state.vault()?, account)?);
            app.recovery_editor
                .as_mut()
                .ok_or("editor")?
                .text
                .push_str("\npublic-close-unsaved-recovery");
        }
        3 => {
            app.state.navigate(Route::Codes);
            settle(app, size);
            click(app, size, "Delete account")?;
            assert!(app.delete.is_some());
        }
        4 => {
            app.state.navigate(Route::BackupCodes);
            app.backup_codes.screen = backup_codes::Screen::Detail {
                account,
                set,
                from_selection: false,
            };
            click(app, size, "Delete code 1…")?;
            assert!(app.backup_codes.detail.has_dialog());
        }
        _ => {
            app.state.navigate(Route::Codes);
            let mut review = ImportReview::manual();
            review.editors[0]
                .service
                .push_str("public-close-unsaved-account");
            review.editors[0]
                .secret
                .push_str("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");
            app.review = Some(review);
        }
    }
    settle(app, size);
    Ok(())
}

#[test]
fn trash_close_matrix_discards_unsubmitted_views_flushes_latest_preferences_and_preserves_business()
-> TestResult {
    for variant in 0..6 {
        let dir = tempfile::tempdir()?;
        let mut v = fixture(2)?;
        own_trash(&mut v.accounts[0].state);
        let original = v.clone();
        let mut app = app(v, dir.path())?;
        let size = [600., 800.];
        install_close_view(&mut app, variant, size)?;
        app.state.query = format!("latest-close-intent-{variant}");
        let expected = app.state.query.clone();
        let p = app.codes_preferences.as_mut().ok_or("preferences")?;
        p.query = Zeroizing::new(expected.clone());
        p.sort = SortOrder::AccountDescending;
        p.view = ViewMode::Grid;
        app.preferences_changed();
        let output = request_close(&mut app, size);
        assert!(
            app.trash.is_none()
                && app.folders.is_none()
                && app.recovery_editor.is_none()
                && app.delete.is_none()
                && !app.backup_codes.detail.has_dialog()
                && app.review.is_none()
        );
        no_secrets(&output);
        assert!(
            labels(&output)
                .iter()
                .all(|s| !s.contains("public-close-unsaved"))
        );
        finish_close(&mut app, size)?;
        drop(app);
        let mut reopened = VaultStore::open_directory(dir.path())?;
        reopened.unlock(pass())?;
        assert_eq!(
            reopened.vault()?.preferences().codes().query.as_str(),
            expected
        );
        assert_eq!(
            reopened.vault()?.preferences().codes().sort,
            SortOrder::AccountDescending
        );
        assert_eq!(reopened.vault()?.preferences().codes().view, ViewMode::Grid);
        same_business(reopened.vault()?, &original);
        reopened.vault()?.validate()?;
    }
    Ok(())
}

fn close_faults(variant: usize) -> TestResult {
    let dir = tempfile::tempdir()?;
    let mut v = fixture(2)?;
    own_trash(&mut v.accounts[0].state);
    let original_business = v.clone();
    let mut initial = VaultStore::open_directory(dir.path())?;
    initial.create_with_parameters(
        pass(),
        v,
        crate::storage::KdfParameters {
            memory_kib: 65_536,
            iterations: 3,
            lanes: 1,
        },
    )?;
    drop(initial);
    for retry in [true, false] {
        let bytes = std::fs::read(dir.path().join("vault.bin"))?;
        let mut app = from_store(VaultStore::open_directory(dir.path())?)?;
        let size = [340., 780.];
        install_close_view(&mut app, variant, size)?;
        app.state.query = if retry {
            "public-close-retry"
        } else {
            "public-close-discard"
        }
        .into();
        app.codes_preferences.as_mut().ok_or("prefs")?.query =
            Zeroizing::new(app.state.query.clone());
        app.preferences_changed();
        std::fs::write(
            dir.path().join("vault.bin"),
            b"owned close preference conflict",
        )?;
        request_close(&mut app, size);
        let deadline = Instant::now() + Duration::from_secs(30);
        while !app.preferences_error {
            if Instant::now() >= deadline {
                return Err("missing recoverable close error".into());
            }
            frame(&mut app, size, vec![]);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(app.closing && !app.exit_approved && !app.close_locked);
        assert!(app.trash.is_none() && app.folders.is_none());
        assert_eq!(
            std::fs::read(dir.path().join("vault.bin"))?,
            b"owned close preference conflict"
        );
        if retry {
            std::fs::write(dir.path().join("vault.bin"), &bytes)?;
            click(&mut app, size, "Retry saving display changes")?;
        } else {
            click(&mut app, size, "Close without saving display changes")?;
        }
        finish_close(&mut app, size)?;
        drop(app);
        if !retry {
            assert_eq!(
                std::fs::read(dir.path().join("vault.bin"))?,
                b"owned close preference conflict"
            );
            std::fs::write(dir.path().join("vault.bin"), &bytes)?;
        }
        let mut reopened = VaultStore::open_directory(dir.path())?;
        reopened.unlock(pass())?;
        assert_eq!(
            reopened.vault()?.preferences().codes().query.as_str(),
            "public-close-retry"
        );
        same_business(reopened.vault()?, &original_business);
    }
    Ok(())
}
#[test]
fn trash_close_error_retry_and_explicit_discard_preserve_latest_intent_and_disk() -> TestResult {
    close_faults(1)
}
#[test]
fn trash_folder_close_error_retry_and_explicit_discard_preserve_latest_intent_and_disk()
-> TestResult {
    close_faults(0)
}

#[test]
fn trash_readonly_preparation_lock_and_close_discard_late_result_without_writing() -> TestResult {
    for close in [false, true] {
        let dir = tempfile::tempdir()?;
        let mut v = fixture(2)?;
        let account = v.accounts[0].id();
        own_trash(&mut v.accounts[0].state);
        let mut app = app(v, dir.path())?;
        let size = [600., 800.];
        open(&mut app, size)?;
        let bytes = std::fs::read(dir.path().join("vault.bin"))?;
        let gate = Arc::new(crate::storage::test_gate::CommitGate::default());
        let release = gate.release_on_drop();
        app.state.hold_next_trash_review(gate.clone());
        click_unsettled(&mut app, size, &action(ItemId::Account(account), "Restore"))?;
        assert!(gate.wait_entered(Duration::from_secs(10)));
        assert!(app.trash.as_ref().is_some_and(|s| s.preparing));
        let progress = settle(&mut app, size);
        assert!(
            labels(&progress)
                .iter()
                .any(|s| s == "Reviewing Trash contents…")
        );
        assert!(
            labels(&progress)
                .iter()
                .all(|s| !s.contains("Public identical"))
        );
        let started = Instant::now();
        if close {
            request_close(&mut app, size);
        } else {
            app.lock();
        }
        assert!(started.elapsed() < Duration::from_secs(1));
        assert!(app.trash.is_none() && !app.state.session_active());
        assert!(!gate.is_released());
        assert_eq!(std::fs::read(dir.path().join("vault.bin"))?, bytes);
        drop(release);
        finish(&mut app, size)?;
        assert!(!app.state.is_unlocked() && app.state.take_trash_review().is_none());
        assert_eq!(std::fs::read(dir.path().join("vault.bin"))?, bytes);
        no_secrets(&settle(&mut app, size));
    }
    Ok(())
}

#[test]
fn trash_equal_time_stale_review_rejects_without_overwrite_and_current_review_can_retry()
-> TestResult {
    let dir = tempfile::tempdir()?;
    let mut v = fixture(2)?;
    let account = v.accounts[0].id();
    own_trash(&mut v.accounts[0].state);
    let mut app = app(v, dir.path())?;
    let size = [600., 800.];
    open(&mut app, size)?;
    click(&mut app, size, &action(ItemId::Account(account), "Restore"))?;
    finish(&mut app, size)?;
    app.state.repository_mut()?.transaction(|v| {
        v.accounts[0].credential = AccountDraft::new(
            "Public equal-time replacement",
            None,
            None,
            Secret::from_base32("ME")?,
            OtpSettings::default(),
        )?;
        Ok(())
    })?;
    let bytes = std::fs::read(dir.path().join("vault.bin"))?;
    click(&mut app, size, "Restore record")?;
    finish(&mut app, size)?;
    assert!(app.trash.as_ref().is_some_and(|s| {
        s.error
            .as_ref()
            .is_some_and(|e| e.contains("contents changed"))
    }));
    assert_eq!(std::fs::read(dir.path().join("vault.bin"))?, bytes);
    click(&mut app, size, "Close review and show current items")?;
    click(&mut app, size, &action(ItemId::Account(account), "Restore"))?;
    finish(&mut app, size)?;
    click(&mut app, size, "Restore record")?;
    finish(&mut app, size)?;
    assert_eq!(
        app.state.vault()?.accounts()[0].credential().service(),
        "Public equal-time replacement"
    );
    assert_eq!(
        app.state.vault()?.accounts()[0].state().deletion,
        DeletionState::Active
    );
    Ok(())
}

#[test]
fn trash_context_widths_pagination_safe_dialog_cancel_focus_keyboard_and_metadata_only()
-> TestResult {
    let dir = tempfile::tempdir()?;
    let mut vault = fixture(26)?;
    for a in &mut vault.accounts {
        own_trash(&mut a.state);
    }
    let mut app = app(vault, dir.path())?;
    for size in [[340., 780.], [600., 800.], [960., 900.]] {
        open(&mut app, size)?;
        let output = settle(&mut app, size);
        no_secrets(&output);
        focused(&app, &output, "Back to Settings", size)?;
        assert!(labels(&output).iter().any(|s| s == "26 item(s) in Trash"));
        let restores = labels(&output)
            .into_iter()
            .filter(|s| s.starts_with("Restore Account · ID:"))
            .collect::<Vec<_>>();
        assert_eq!(restores.len(), 20);
        click(&mut app, size, "Next")?;
        let output = settle(&mut app, size);
        let target = labels(&output)
            .into_iter()
            .find(|s| s.starts_with("Restore Account · ID:"))
            .ok_or("page2 restore")?;
        let disk = std::fs::read(dir.path().join("vault.bin"))?;
        for cancel in [0, 1, 2] {
            click(&mut app, size, &target)?;
            let output = settle(&mut app, size);
            focused(&app, &output, "Keep in Trash", size)?;
            no_secrets(&output);
            if cancel == 0 {
                click(&mut app, size, "Keep in Trash")?;
            } else if cancel == 1 {
                key(&mut app, size, egui::Key::Escape);
            } else {
                for pressed in [true, false] {
                    frame(
                        &mut app,
                        size,
                        vec![
                            egui::Event::PointerMoved(egui::pos2(2., 2.)),
                            egui::Event::PointerButton {
                                pos: egui::pos2(2., 2.),
                                button: egui::PointerButton::Primary,
                                pressed,
                                modifiers: egui::Modifiers::NONE,
                            },
                        ],
                    );
                }
            }
            let output = settle(&mut app, size);
            focused(&app, &output, &target, size)?;
            assert_eq!(std::fs::read(dir.path().join("vault.bin"))?, disk);
        }
        tab_to(&mut app, size, &target)?;
        key(&mut app, size, egui::Key::Enter);
        tab_to(&mut app, size, "Restore record")?;
        key(&mut app, size, egui::Key::Enter);
        finish(&mut app, size)?;
        assert_eq!(domain_trash::items(app.state.vault()?).len(), 25);
        let restored = app
            .state
            .vault()?
            .accounts()
            .iter()
            .find(|a| a.state().deletion == DeletionState::Active)
            .ok_or("restored account")?
            .id();
        // Re-trash through the real existing account mutation, keeping the fixture
        // shape for the next width without resetting the UI to an oversized view.
        let state = app
            .state
            .vault()?
            .accounts()
            .iter()
            .find(|a| a.id() == restored)
            .ok_or("account")?
            .state();
        assert!(app.begin_mutation(SaveIntent::Trash {
            id: restored,
            expected: state
        }));
        finish(&mut app, size)?;
        click(&mut app, size, "Back to Settings")?;
        let output = settle(&mut app, size);
        focused(&app, &output, "Trash · 26 item(s)", size)?;
    }
    Ok(())
}

#[test]
fn trash_restore_parent_path_preserves_own_child_trash_and_duplicate_consent_is_explicit()
-> TestResult {
    let dir = tempfile::tempdir()?;
    let mut v = fixture(2)?;
    let account = v.accounts[0].id();
    let set = v.backup_sets[0].id();
    let code = v.backup_sets[0].codes[0].id();
    own_trash(&mut v.accounts[0].state);
    own_trash(&mut v.backup_sets[0].state);
    own_trash(&mut v.backup_sets[0].codes[0].state);
    let duplicate = v.accounts[0].credential().clone();
    v.add_account(duplicate, Timestamp(100))?;
    let mut app = app(v, dir.path())?;
    let size = [340., 780.];
    open(&mut app, size)?;
    click(&mut app, size, &action(ItemId::Code(set, code), "Restore"))?;
    let output = settle(&mut app, size);
    assert!(
        labels(&output)
            .iter()
            .any(|s| s.contains("Restore these parents first"))
    );
    click(
        &mut app,
        size,
        &action(ItemId::Account(account), "Review parent restore"),
    )?;
    let output = settle(&mut app, size);
    focused(&app, &output, "Keep in Trash", size)?;
    assert!(
        labels(&output)
            .iter()
            .any(|s| s.contains("Matching entries already exist"))
    );
    tab_to(
        &mut app,
        size,
        "Restore as a separate record and keep matching entries",
    )?;
    key(&mut app, size, egui::Key::Space);
    click(&mut app, size, "Restore record")?;
    finish(&mut app, size)?;
    assert_eq!(app.state.vault()?.accounts().len(), 3);
    assert_eq!(
        ItemId::Account(account)
            .state(app.state.vault()?)
            .ok_or("state")?
            .deletion,
        DeletionState::Active
    );
    assert_ne!(
        ItemId::Set(set)
            .state(app.state.vault()?)
            .ok_or("set")?
            .deletion,
        DeletionState::Active
    );
    click(&mut app, size, &action(ItemId::Set(set), "Restore"))?;
    click(&mut app, size, "Restore record")?;
    finish(&mut app, size)?;
    assert_ne!(
        ItemId::Code(set, code)
            .state(app.state.vault()?)
            .ok_or("code")?
            .deletion,
        DeletionState::Active
    );
    click(&mut app, size, &action(ItemId::Code(set, code), "Restore"))?;
    click(&mut app, size, "Restore record")?;
    finish(&mut app, size)?;
    assert_eq!(
        app.state.vault()?.backup_sets()[0].codes()[0].used_at(),
        Some(Timestamp(100))
    );
    no_secrets(&settle(&mut app, size));
    app.state.vault()?.validate()?;
    Ok(())
}

#[test]
fn trash_permanent_scope_fault_retry_latest_preferences_and_encrypted_restart() -> TestResult {
    let dir = tempfile::tempdir()?;
    let mut v = fixture(2)?;
    let folder = v.folders[0].id();
    own_trash(&mut v.folders[0].state);
    let original = v.clone();
    let mut app = app(v, dir.path())?;
    let size = [600., 800.];
    settle(&mut app, size);
    app.state.query = "latest intent before Trash".into();
    let preferences = app.codes_preferences.as_mut().ok_or("preferences")?;
    preferences.query = Zeroizing::new(app.state.query.clone());
    preferences.sort = SortOrder::AccountDescending;
    preferences.view = ViewMode::Grid;
    app.preferences_changed();
    open(&mut app, size)?;
    click(
        &mut app,
        size,
        &action(ItemId::Folder(folder), "Permanently delete"),
    )?;
    let output = settle(&mut app, size);
    focused(&app, &output, "Keep in Trash", size)?;
    assert!(labels(&output).iter().any(|s| {
        s.contains("0 account(s), 1 folder(s), 2 membership(s), 0 backup-code set(s), 0 code(s)")
    }));
    assert!(
        labels(&output)
            .iter()
            .any(|s| s.contains("SSD") && s.contains("not guaranteed"))
    );
    let bytes = std::fs::read(dir.path().join("vault.bin"))?;
    std::fs::write(dir.path().join("vault.bin"), b"owned public Trash conflict")?;
    click(&mut app, size, "Permanently delete reviewed records")?;
    assert!(
        app.state.busy() && app.trash.as_ref().is_some_and(|s| s.saving),
        "pointer confirmation must queue the real worker"
    );
    finish(&mut app, size)?;
    assert!(
        app.trash
            .as_ref()
            .is_some_and(|s| s.error.is_some() && !s.saving)
    );
    assert_eq!(app.state.vault()?.folders().len(), 1);
    assert!(app.preferences_dirty.is_some());
    assert_eq!(
        std::fs::read(dir.path().join("vault.bin"))?,
        b"owned public Trash conflict"
    );
    std::fs::write(dir.path().join("vault.bin"), bytes)?;
    click(&mut app, size, "Permanently delete reviewed records")?;
    finish(&mut app, size)?;
    assert!(app.state.vault()?.folders().is_empty() && app.state.vault()?.memberships().is_empty());
    assert!(app.preferences_dirty.is_none());
    assert_eq!(
        app.state.vault()?.preferences().codes().query.as_str(),
        "latest intent before Trash"
    );
    for (now, old) in app
        .state
        .vault()?
        .accounts()
        .iter()
        .zip(original.accounts())
    {
        assert_eq!(now.id(), old.id());
        assert!(now.credential().duplicate_criteria(old.credential()).0);
    }
    assert_eq!(
        app.state.vault()?.backup_sets()[0].codes()[0].used_at(),
        Some(Timestamp(100))
    );
    app.lock();
    drop(app);
    let mut store = VaultStore::open_directory(dir.path())?;
    store.unlock(pass())?;
    assert!(store.vault()?.folders().is_empty());
    assert_eq!(store.vault()?.accounts().len(), 2);
    assert_eq!(store.vault()?.preferences().codes().view, ViewMode::Grid);
    assert_eq!(
        store.vault()?.preferences().codes().sort,
        SortOrder::AccountDescending
    );
    store.vault()?.validate()?;
    Ok(())
}

#[test]
fn trash_empty_confirmation_last_code_scope_and_admitted_lock_clear_review_before_release()
-> TestResult {
    let dir = tempfile::tempdir()?;
    let mut v = fixture(2)?;
    v.backup_sets[0].codes.remove(1);
    own_trash(&mut v.backup_sets[0].codes[0].state);
    let account = v.accounts[0].id();
    let set = v.backup_sets[0].id();
    let code = v.backup_sets[0].codes[0].id();
    let mut app = app(v, dir.path())?;
    let size = [600., 800.];
    open(&mut app, size)?;
    click(&mut app, size, "Empty trash…")?;
    let output = settle(&mut app, size);
    focused(&app, &output, "Keep in Trash", size)?;
    assert!(
        labels(&output)
            .iter()
            .any(|s| s.contains("final stored code"))
    );
    assert!(labels(&output).iter().any(|s| {
        s.contains("0 account(s), 0 folder(s), 0 membership(s), 1 backup-code set(s), 1 code(s)")
    }));
    click(&mut app, size, "Keep in Trash")?;
    assert!(ItemId::Code(set, code).state(app.state.vault()?).is_some());
    click(&mut app, size, "Empty trash…")?;
    let gate = Arc::new(crate::storage::test_gate::CommitGate::default());
    let release = gate.release_on_drop();
    app.state.hold_next_save_after_admission(gate.clone());
    click(&mut app, size, "Permanently delete reviewed records")?;
    assert!(gate.wait_entered(Duration::from_secs(10)));
    let started = Instant::now();
    app.lock();
    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(app.trash.is_none() && !app.state.session_active());
    no_secrets(&settle(&mut app, [340., 780.]));
    assert!(!gate.is_released());
    drop(release);
    finish(&mut app, [340., 780.])?;
    assert!(!app.state.is_unlocked() && app.status.is_none());
    app.state.repository_mut()?.unlock(pass())?;
    assert!(
        app.state
            .vault()?
            .accounts()
            .iter()
            .any(|a| a.id() == account)
    );
    assert!(
        !app.state
            .vault()?
            .backup_sets()
            .iter()
            .any(|s| s.id() == set)
    );
    assert_eq!(app.state.vault()?.backup_sets().len(), 1);
    app.state.vault()?.validate()?;
    Ok(())
}
