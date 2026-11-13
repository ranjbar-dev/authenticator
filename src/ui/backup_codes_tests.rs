//! Disposable encrypted fixtures; Context input is not native pointer evidence.
use super::backup_codes::{PAGE_SIZE, Projection, Screen};
use super::*;
use crate::{
    backup::{self, Destination, ImportMode, ImportOptions},
    domain::{
        otp::{OtpSettings, Secret},
        records::{AccountDraft, BackupCode, SortOrder, Timestamp, Vault},
    },
    storage::OperationCancellation,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn pass() -> Zeroizing<String> {
    Zeroizing::new("disposable-backup-codes".into())
}
fn draft(service: &str, account: Option<&str>) -> Result<AccountDraft, crate::error::AppError> {
    AccountDraft::new(
        service,
        Some("https://example.invalid"),
        account,
        Secret::from_base32("MY")?,
        OtpSettings::default(),
    )
}
fn fixture() -> Result<Vault, crate::error::AppError> {
    let mut v = Vault::new(Timestamp(1));
    let id = v.add_account(
        draft("Public Mail", Some("public@fixture.invalid"))?,
        Timestamp(2),
    )?;
    v.add_account(
        draft("Public Mail", Some("public+work@fixture.invalid"))?,
        Timestamp(3),
    )?;
    v.add_account(
        draft("Public Mail", Some("public@fixture.invalid"))?,
        Timestamp(4),
    )?;
    let mut used = BackupCode::new("public-used-recovery-only", Timestamp(4))?;
    used.set_used(true, Timestamp(4))?;
    v.add_backup_set(
        id,
        vec![
            used,
            BackupCode::new("public-available-recovery-only", Timestamp(4))?,
        ],
        Timestamp(5),
    )?;
    Ok(v)
}
fn fixture_app(
    vault: Vault,
    path: &std::path::Path,
) -> Result<AuthenticatorApp, Box<dyn std::error::Error>> {
    let mut store = VaultStore::open_directory(path)?;
    store.create(pass(), vault)?;
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut app = AuthenticatorApp::with_context(context, Ok(store), 0);
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
    let mut output = ctx.run_ui(
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
    output.textures_delta.clear();
    output
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
fn no_secrets(out: &egui::FullOutput) {
    for text in names(out) {
        assert!(
            !text.contains("public-used-recovery-only")
                && !text.contains("public-available-recovery-only")
                && !text.contains("public-trashed-recovery-only")
                && !text.contains("public-trashed-set-code")
                && !text.contains("public-hidden-parent-code")
                && !text.contains("GEZDGNBV")
        );
    }
    for command in &out.platform_output.commands {
        assert!(!matches!(command, egui::OutputCommand::CopyText(_)));
    }
}
fn click(app: &mut AuthenticatorApp, label: &str, occurrence: usize) -> TestResult {
    let out = render(app, 600., 16000., vec![]);
    let mut candidates = out
        .platform_output
        .accesskit_update
        .as_ref()
        .ok_or("tree")?
        .nodes
        .iter()
        .filter(|(_, n)| {
            matches!(n.role(), egui::Role::Button | egui::Role::RadioButton)
                && n.label() == Some(label)
        })
        .filter_map(|(_, n)| n.bounds())
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| a.y0.total_cmp(&b.y0));
    let bounds = candidates
        .get(occurrence)
        .ok_or_else(|| format!("missing button {label}"))?;
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
fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    }
}

#[test]
fn backup_display_placeholder_collision_has_exact_ids_in_list_summary_and_both_handoffs()
-> TestResult {
    let mut vault = Vault::new(Timestamp(1));
    let mut targets = Vec::new();
    for (index, (service, identifier, existing)) in [
        ("Public editor collision", None, false),
        (
            "Public editor collision",
            Some("No account identifier"),
            false,
        ),
        ("Public detail collision", None, true),
        (
            "Public detail collision",
            Some("No account identifier"),
            true,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let id = vault.add_account(draft(service, identifier)?, Timestamp(index as u64 + 2))?;
        let set = if existing {
            Some(vault.add_backup_set(
                id,
                vec![BackupCode::new(
                    "public-available-recovery-only",
                    Timestamp(index as u64 + 2),
                )?],
                Timestamp(index as u64 + 2),
            )?)
        } else {
            None
        };
        targets.push((id, set));
    }
    let directory = tempfile::tempdir()?;
    let mut app = fixture_app(vault, directory.path())?;
    let label = |id: AccountId| format!("Account ID: {}", id.as_uuid());
    for width in [340., 600., 960.] {
        let out = render(&mut app, width, 16000., vec![]);
        no_secrets(&out);
        let labels = names(&out);
        for (id, set) in &targets {
            assert_eq!(labels.contains(&label(*id)), set.is_some());
        }
    }
    for index in 0..targets.len() {
        app.backup_codes.select_accounts();
        let ordered =
            crate::ui::codes::ordered_accounts(app.state.vault()?, "", app.backup_codes.sort);
        let id = ordered[index];
        let expected = targets
            .iter()
            .find(|(target, _)| *target == id)
            .ok_or("target")?
            .1;
        let before = names(&render(&mut app, 600., 16000., vec![]));
        for (target, _) in &targets {
            assert!(before.contains(&label(*target)));
        }
        click(&mut app, "Select account", index)?;
        assert_eq!(app.backup_codes.selected, Some(id));
        let selected = names(&render(&mut app, 600., 16000., vec![]));
        assert!(
            selected.iter().filter(|s| **s == label(id)).count()
                > before.iter().filter(|s| **s == label(id)).count()
        );
        click(&mut app, "Continue", 0)?;
        match expected {
            Some(set) => assert_eq!(
                app.backup_codes.screen,
                Screen::Detail {
                    account: id,
                    set,
                    from_selection: true
                }
            ),
            None => assert_eq!(app.backup_codes.screen, Screen::Editor(id)),
        }
        for width in [340., 600., 960.] {
            let out = render(&mut app, width, 16000., vec![]);
            no_secrets(&out);
            let labels = names(&out);
            assert!(labels.contains(&label(id)));
            assert!(
                targets
                    .iter()
                    .filter(|(target, _)| *target != id)
                    .all(|(target, _)| !labels.contains(&label(*target)))
            );
        }
    }
    Ok(())
}

fn wait(app: &mut AuthenticatorApp) -> TestResult {
    let deadline = Instant::now() + Duration::from_secs(60);
    while app.state.busy() {
        render(app, 600., 900., vec![]);
        if Instant::now() >= deadline {
            return Err("backup-code worker timeout".into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Ok(())
}

fn keyboard_reaches(
    app: &mut AuthenticatorApp,
    width: f32,
    height: f32,
    label: &str,
) -> TestResult {
    for _ in 0..100 {
        let mut release = key(egui::Key::Tab, egui::Modifiers::NONE);
        if let egui::Event::Key { pressed, .. } = &mut release {
            *pressed = false;
        }
        let out = render(
            app,
            width,
            height,
            vec![key(egui::Key::Tab, egui::Modifiers::NONE), release],
        );
        let focused = app.context.memory(|m| m.focused());
        let reached = out
            .platform_output
            .accesskit_update
            .as_ref()
            .is_some_and(|t| {
                t.nodes.iter().any(|(id, n)| {
                    Some(id.0) == focused.map(|id| id.value()) && n.label() == Some(label)
                })
            });
        if reached {
            for _ in 0..30 {
                render(app, width, height, vec![]);
            }
            let out = render(app, width, height, vec![]);
            let bounds = out
                .platform_output
                .accesskit_update
                .as_ref()
                .ok_or("tree")?
                .nodes
                .iter()
                .find(|(id, _)| Some(id.0) == focused.map(|id| id.value()))
                .and_then(|(_, n)| n.bounds())
                .ok_or("focused control bounds")?;
            assert!(
                bounds.x0 >= 0.
                    && bounds.x1 <= f64::from(width)
                    && bounds.y0 >= 0.
                    && bounds.y1 <= f64::from(height),
                "focused {label} outside normal {width}x{height}: {bounds:?}"
            );
            return Ok(());
        }
    }
    Err(format!("keyboard did not reach {label}").into())
}

#[test]
fn backup_normal_viewports_keyboard_live_search_intended_account_and_wrapped_identity() -> TestResult
{
    for (width, height) in [(340., 780.), (600., 800.), (960., 900.)] {
        let directory = tempfile::tempdir()?;
        let mut v = fixture()?;
        let service =
            "Public service with a deliberately long name that remains readable on a narrow window";
        let account = "long-public-account-identifier@fixture.invalid";
        v.add_account(draft(service, Some(account))?, Timestamp(6))?;
        let intended = v.accounts()[1].id();
        let mut app = fixture_app(v, directory.path())?;
        render(&mut app, width, height, vec![]);
        keyboard_reaches(&mut app, width, height, "Add Backup Codes")?;
        render(
            &mut app,
            width,
            height,
            vec![key(egui::Key::Space, egui::Modifiers::NONE)],
        );
        assert_eq!(app.backup_codes.screen, Screen::Select);
        render(&mut app, width, height, vec![]);
        let full = render(&mut app, width, 16000., vec![]);
        for text in [service, account] {
            let bounds = full
                .platform_output
                .accesskit_update
                .as_ref()
                .ok_or("tree")?
                .nodes
                .iter()
                .find(|(_, n)| n.label() == Some(text) || n.value() == Some(text))
                .and_then(|(_, n)| n.bounds())
                .ok_or_else(|| format!("identity bounds for {text}: {:?}", names(&full)))?;
            assert!(bounds.x0 >= 0. && bounds.x1 <= f64::from(width));
            if text == service {
                assert!(bounds.y1 - bounds.y0 > f64::from(theme::SECTION.line_height));
            }
        }
        // Initial focus belongs to the selector's real labeled TextEdit.
        let out = render(
            &mut app,
            width,
            height,
            vec![egui::Event::Paste("work".into())],
        );
        no_secrets(&out);
        assert_eq!(app.state.backup_query, "work");
        assert!(app.state.query.is_empty());
        let full = render(&mut app, width, 16000., vec![]);
        assert!(
            names(&full)
                .iter()
                .any(|s| s == "public+work@fixture.invalid")
        );
        assert!(!names(&full).iter().any(|s| s == "public@fixture.invalid"));
        render(&mut app, width, height, vec![]);
        keyboard_reaches(&mut app, width, height, "Select account")?;
        render(
            &mut app,
            width,
            height,
            vec![key(egui::Key::Space, egui::Modifiers::NONE)],
        );
        assert_eq!(app.backup_codes.selected, Some(intended));
        keyboard_reaches(&mut app, width, height, "Continue")?;
        render(
            &mut app,
            width,
            height,
            vec![key(egui::Key::Space, egui::Modifiers::NONE)],
        );
        assert_eq!(app.backup_codes.screen, Screen::Editor(intended));
        no_secrets(&render(&mut app, width, height, vec![]));
        app.state.navigate(Route::Codes);
        render(&mut app, width, height, vec![]);
        assert_eq!(app.backup_codes.screen, Screen::List);
        assert!(app.backup_codes.selected.is_none());
    }
    Ok(())
}

#[test]
fn backup_account_pagination_retains_control_ids_and_search_resets_page() -> TestResult {
    let mut v = Vault::new(Timestamp(1));
    for index in 0..PAGE_SIZE + 3 {
        v.add_account(
            draft(&format!("Public account {index:02}"), None)?,
            Timestamp(2),
        )?;
    }
    let directory = tempfile::tempdir()?;
    let mut app = fixture_app(v, directory.path())?;
    app.backup_codes.select_accounts();
    let ids = |out: &egui::FullOutput| {
        out.platform_output
            .accesskit_update
            .as_ref()
            .map(|t| {
                t.nodes
                    .iter()
                    .filter(|(_, n)| {
                        n.role() == egui::Role::Button
                            && matches!(n.label(), Some("Previous" | "Next"))
                    })
                    .map(|(id, n)| (n.label().unwrap_or_default().to_owned(), *id))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };
    let first = render(&mut app, 600., 16000., vec![]);
    let original = ids(&first);
    assert_eq!(original.len(), 2);
    assert_eq!(
        names(&first)
            .iter()
            .filter(|s| s.as_str() == "Select account")
            .count(),
        PAGE_SIZE
    );
    click(&mut app, "Next", 0)?;
    let second = render(&mut app, 600., 16000., vec![]);
    assert_eq!(app.backup_codes.page, 1);
    assert_eq!(ids(&second), original);
    assert_eq!(
        names(&second)
            .iter()
            .filter(|s| s.as_str() == "Select account")
            .count(),
        3
    );
    click(&mut app, "Previous", 0)?;
    assert_eq!(app.backup_codes.page, 0);
    click(&mut app, "Next", 0)?;
    click(&mut app, "Clear search", 0)?;
    assert_eq!(app.backup_codes.page, 0);
    assert_eq!(ids(&render(&mut app, 600., 16000., vec![])), original);
    Ok(())
}

#[test]
fn backup_encrypted_reopen_roundtrip_and_adopted_import_refresh_only_on_success() -> TestResult {
    let source_dir = tempfile::tempdir()?;
    let mut source = VaultStore::open_directory(source_dir.path())?;
    let mut original = fixture()?;
    let account = original.accounts()[0].id();
    let set = original.backup_sets()[0].id();
    let mut trashed_code = BackupCode::new("public-trashed-recovery-only", Timestamp(5))?;
    trashed_code.state.deletion = DeletionState::Trashed { at: Timestamp(5) };
    original.backup_sets[0].codes.push(trashed_code);
    let trashed_set_owner =
        original.add_account(draft("Public trashed set", None)?, Timestamp(6))?;
    original.add_backup_set(
        trashed_set_owner,
        vec![BackupCode::new("public-trashed-set-code", Timestamp(6))?],
        Timestamp(6),
    )?;
    original.backup_sets[1].state.deletion = DeletionState::Trashed { at: Timestamp(6) };
    let hidden_parent = original.add_account(draft("Public hidden parent", None)?, Timestamp(7))?;
    original.add_backup_set(
        hidden_parent,
        vec![BackupCode::new("public-hidden-parent-code", Timestamp(7))?],
        Timestamp(7),
    )?;
    original.trash_account(
        hidden_parent,
        original.accounts().last().ok_or("parent")?.state(),
        Timestamp(8),
    )?;
    original.validate()?;
    source.create(pass(), original)?;
    drop(source);
    let mut source = VaultStore::open_directory(source_dir.path())?;
    source.unlock(pass())?;
    let before = Projection::new(source.vault()?, "", SortOrder::LatestFirst);
    assert_eq!(before.sets[&account].id, set);
    assert_eq!(
        (before.sets[&account].available, before.sets[&account].used),
        (1, 1)
    );
    let files = tempfile::tempdir()?;
    let path = files.path().join("public-backup.authpbk");
    let backup_pass = || Zeroizing::new("disposable-portable-backup".into());
    backup::export(
        &source,
        Destination::create_new(&path)?,
        backup_pass(),
        &OperationCancellation::default(),
    )?;
    let original_bytes = std::fs::read(&path)?;
    let local_dir = tempfile::tempdir()?;
    let mut app = fixture_app(fixture()?, local_dir.path())?;
    let local_account = app.state.vault()?.accounts()[0].id();
    let local_set = app.state.vault()?.backup_sets()[0].id();
    app.backup_codes.selected = Some(local_account);
    app.backup_codes.screen = Screen::Detail {
        account: local_account,
        set: local_set,
        from_selection: true,
    };
    app.state.backup_query = "local filter".into();
    for fail in [true, false] {
        let preview = backup::preview(
            app.state.repository_mut()?,
            &path,
            backup_pass(),
            &OperationCancellation::default(),
        )?;
        let plan = preview.plan(ImportOptions {
            mode: ImportMode::Replace,
            ..Default::default()
        })?;
        if fail {
            std::fs::write(&path, b"changed-owned-backup")?;
        }
        let mut form = ImportForm::new(egui::Id::unique("backup-code-import-test"));
        form.plan = Some(plan);
        app.apply_backup_plan(&mut form);
        app.backup_import = Some(form);
        wait(&mut app)?;
        if fail {
            assert!(
                app.backup_import
                    .as_ref()
                    .is_some_and(|f| f.error.is_some())
            );
            assert_eq!(app.backup_codes.selected, Some(local_account));
            assert_eq!(
                app.backup_codes.screen,
                Screen::Detail {
                    account: local_account,
                    set: local_set,
                    from_selection: true
                }
            );
            assert_eq!(app.state.backup_query, "local filter");
            std::fs::write(&path, &original_bytes)?;
        } else {
            assert!(app.backup_import.is_none());
            assert_eq!(app.backup_codes.screen, Screen::List);
            assert!(app.backup_codes.selected.is_none());
            assert!(app.state.backup_query.is_empty());
        }
    }
    app.state.navigate(Route::BackupCodes);
    let out = render(&mut app, 600., 16000., vec![]);
    no_secrets(&out);
    assert!(names(&out).iter().any(|s| s == "1 available · 1 used"));
    let p = Projection::new(app.state.vault()?, "", SortOrder::LatestFirst);
    assert_eq!(p.visible_sets(), vec![account]);
    assert_eq!(p.sets[&account].id, set);
    assert!(p.sets[&trashed_set_owner].trashed);
    assert!(!p.sets.contains_key(&hidden_parent));
    drop(app);
    let mut reopened = VaultStore::open_directory(local_dir.path())?;
    reopened.unlock(pass())?;
    let p = Projection::new(reopened.vault()?, "", SortOrder::LatestFirst);
    assert_eq!(p.sets[&account].id, set);
    assert_eq!((p.sets[&account].available, p.sets[&account].used), (1, 1));
    assert!(p.sets[&trashed_set_owner].trashed);
    assert!(!p.sets.contains_key(&hidden_parent));
    assert_eq!(reopened.vault()?.backup_sets()[0].codes().len(), 3);
    Ok(())
}

#[test]
fn backup_successful_mutation_invalidates_projection_even_when_aggregate_time_is_unchanged()
-> TestResult {
    let mut v = fixture()?;
    let future = Timestamp(crate::domain::clock::SystemClock.unix_seconds()? + 3600);
    v.state.updated_at = future;
    v.validate()?;
    let id = v.accounts()[0].id();
    let expected = v.accounts()[0].state();
    let directory = tempfile::tempdir()?;
    let mut app = fixture_app(v, directory.path())?;
    assert!(
        names(&render(&mut app, 600., 16000., vec![]))
            .iter()
            .any(|s| s == "1 available · 1 used")
    );
    app.backup_codes.selected = Some(id);
    app.backup_codes.screen = Screen::Detail {
        account: id,
        set: app.state.vault()?.backup_sets()[0].id(),
        from_selection: true,
    };
    assert!(app.begin_mutation(SaveIntent::Trash { id, expected }));
    wait(&mut app)?;
    assert_eq!(app.state.vault()?.state().updated_at, future);
    let out = render(&mut app, 600., 16000., vec![]);
    no_secrets(&out);
    assert!(names(&out).iter().any(|s| s == "No backup codes yet"));
    assert!(app.backup_codes.selected.is_none());
    assert_eq!(app.backup_codes.screen, Screen::List);
    drop(app);
    let mut reopened = VaultStore::open_directory(directory.path())?;
    reopened.unlock(pass())?;
    assert!(
        Projection::new(reopened.vault()?, "", SortOrder::LatestFirst)
            .visible_sets()
            .is_empty()
    );
    Ok(())
}

#[test]
fn backup_selector_six_sorts_casefold_ties_missing_identifier_and_independent_preferences()
-> TestResult {
    let mut v = Vault::new(Timestamp(1));
    let a = v.add_account(draft("Zulu", Some("alpha"))?, Timestamp(2))?;
    let b = v.add_account(draft("Alpha", Some("zebra"))?, Timestamp(3))?;
    let c = v.add_account(draft("ÉCOLE", Some("ÜSER"))?, Timestamp(4))?;
    let d = v.add_account(draft("Alpha", None)?, Timestamp(5))?;
    let mut tie = vec![b, d];
    tie.sort();
    for (sort, expected) in [
        (SortOrder::LatestFirst, vec![d, c, b, a]),
        (SortOrder::EarliestFirst, vec![a, b, c, d]),
        (
            SortOrder::ServiceAscending,
            [tie.clone(), vec![a, c]].concat(),
        ),
        (SortOrder::ServiceDescending, [vec![c, a], tie].concat()),
        (SortOrder::AccountAscending, vec![a, b, c, d]),
        (SortOrder::AccountDescending, vec![c, b, a, d]),
    ] {
        assert_eq!(Projection::new(&v, "", sort).accounts, expected);
    }
    let folded = v.add_account(draft("Straße", Some("Σς"))?, Timestamp(6))?;
    for query in ["STRASSE", "σσ"] {
        assert_eq!(
            Projection::new(&v, query, SortOrder::LatestFirst).accounts,
            vec![folded]
        );
    }
    let trash = v.add_account(draft("Hidden", None)?, Timestamp(7))?;
    v.trash_account(
        trash,
        v.accounts().last().ok_or("account")?.state(),
        Timestamp(8),
    )?;
    let folder = v.add_folder(
        "Only one",
        crate::domain::records::FolderColor::Blue,
        Timestamp(9),
    )?;
    v.add_membership(folder, a, Timestamp(10))?;
    let mut p = v.preferences().codes();
    p.query = Zeroizing::new("Zulu".into());
    p.folder = FolderFilter::Folder(folder);
    p.view = ViewMode::Grid;
    v.set_codes_preferences(p, Timestamp(11))?;
    assert_eq!(
        Projection::new(&v, "école", SortOrder::LatestFirst).accounts,
        vec![c]
    );
    assert_eq!(
        Projection::new(&v, "üser", SortOrder::LatestFirst).accounts,
        vec![c]
    );
    assert!(
        !Projection::new(&v, "", SortOrder::LatestFirst)
            .accounts
            .contains(&trash)
    );
    assert_eq!(&*v.preferences().codes().query, "Zulu");
    Ok(())
}

#[test]
fn backup_counts_exclude_parent_set_and_code_trash_preserve_exact_handoffs() -> TestResult {
    let mut v = fixture()?;
    let id = v.accounts()[0].id();
    let other = v.accounts()[1].id();
    let set = v.backup_sets()[0].id();
    let p = Projection::new(&v, "", SortOrder::LatestFirst);
    assert_eq!(p.visible_sets(), vec![id]);
    assert_eq!((p.sets[&id].available, p.sets[&id].used), (1, 1));
    let mut session = BackupSession::default();
    session.select_accounts();
    session.selected = Some(other);
    assert!(session.continue_to(&v));
    assert_eq!(session.screen, Screen::Editor(other));
    session.back();
    assert_eq!(session.screen, Screen::Select);
    assert_eq!(session.selected, Some(other));
    session.selected = Some(id);
    assert!(session.continue_to(&v));
    assert_eq!(
        session.screen,
        Screen::Detail {
            account: id,
            set,
            from_selection: true
        }
    );
    // Restore/delete domain UI belongs to Task18; this models authenticated DTO
    // states and validates the actual graph instead of inventing frontend data.
    v.backup_sets[0].codes[0].state.deletion = DeletionState::Trashed { at: Timestamp(4) };
    v.validate()?;
    assert_eq!(
        (
            Projection::new(&v, "", SortOrder::LatestFirst).sets[&id].available,
            Projection::new(&v, "", SortOrder::LatestFirst).sets[&id].used
        ),
        (1, 0)
    );
    v.backup_sets[0].state.deletion = DeletionState::Trashed { at: Timestamp(5) };
    v.validate()?;
    let p = Projection::new(&v, "", SortOrder::LatestFirst);
    assert!(p.visible_sets().is_empty());
    assert_eq!(p.sets[&id].id, set);
    assert!(p.sets[&id].trashed);
    assert_eq!(p.sets[&id].available, 0);
    session.back();
    session.selected = Some(id);
    assert!(session.continue_to(&v));
    assert!(matches!(session.screen,Screen::Detail {set:s,..} if s==set));
    assert!(
        v.add_backup_set(
            id,
            vec![BackupCode::new("public-collision", Timestamp(5))?],
            Timestamp(5)
        )
        .is_err()
    );
    v.trash_account(id, v.accounts()[0].state(), Timestamp(6))?;
    let p = Projection::new(&v, "", SortOrder::LatestFirst);
    assert!(!p.accounts.contains(&id));
    assert!(!p.sets.contains_key(&id));
    // Source graph restoration preserves original IDs and derived states.
    v.accounts[0].state.deletion = DeletionState::Active;
    v.backup_sets[0].state.deletion = DeletionState::Active;
    v.validate()?;
    assert_eq!(
        Projection::new(&v, "", SortOrder::LatestFirst).visible_sets(),
        vec![id]
    );
    Ok(())
}

#[test]
fn backup_root_empty_populated_no_match_selection_back_help_and_lock() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut app = fixture_app(fixture()?, directory.path())?;
    let id = app.state.vault()?.accounts()[0].id();
    let set = app.state.vault()?.backup_sets()[0].id();
    for width in [340., 600., 960.] {
        let out = render(&mut app, width, 16000., vec![]);
        no_secrets(&out);
        let labels = names(&out);
        assert!(labels.iter().any(|s| s == "1 available · 1 used"));
        assert!(labels.iter().any(|s| s == "Add Backup Codes"));
        assert!(labels.iter().any(|s| s == "View set"));
    }
    click(&mut app, "Learn about backup codes", 0)?;
    let out = render(&mut app, 600., 900., vec![]);
    assert!(
        names(&out)
            .iter()
            .any(|s| s.contains("issued by each service"))
    );
    no_secrets(&out);
    click(&mut app, "Close", 0)?;
    click(&mut app, "Add Backup Codes", 0)?;
    assert_eq!(app.backup_codes.screen, Screen::Select);
    let out = render(&mut app, 600., 16000., vec![]);
    assert!(names(&out).contains(&format!("Account ID: {}", id.as_uuid())));
    // Latest first: duplicate without a set, similar username, then original.
    click(&mut app, "Select account", 2)?;
    assert_eq!(app.backup_codes.selected, Some(id));
    let out = render(&mut app, 600., 16000., vec![]);
    assert!(names(&out).iter().any(|s| s == "Selected account"));
    click(&mut app, "Continue", 0)?;
    assert_eq!(
        app.backup_codes.screen,
        Screen::Detail {
            account: id,
            set,
            from_selection: true
        }
    );
    no_secrets(&render(&mut app, 340., 780., vec![]));
    render(
        &mut app,
        600.,
        900.,
        vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
    );
    assert_eq!(app.backup_codes.screen, Screen::Select);
    click(&mut app, "Back to Backup Codes", 0)?;
    app.state.backup_query = "absent".into();
    app.backup_codes.invalidate();
    let out = render(&mut app, 600., 16000., vec![]);
    assert!(names(&out).iter().any(|s| s == "No matching accounts"));
    click(&mut app, "Clear search", 0)?;
    assert!(app.state.backup_query.is_empty());
    app.backup_codes.selected = Some(id);
    app.backup_codes.screen = Screen::Detail {
        account: id,
        set,
        from_selection: false,
    };
    render(
        &mut app,
        600.,
        900.,
        vec![key(egui::Key::L, egui::Modifiers::CTRL)],
    );
    assert!(!app.state.is_unlocked());
    assert_eq!(app.backup_codes.screen, Screen::List);
    assert!(app.backup_codes.selected.is_none());
    let out = render(&mut app, 340., 780., vec![]);
    no_secrets(&out);
    assert!(
        !names(&out).iter().any(|s| s.contains("Public Mail")
            || s.contains("public@")
            || s.contains("available ·"))
    );
    drop(app);
    let empty_dir = tempfile::tempdir()?;
    let mut empty = fixture_app(Vault::new(Timestamp(1)), empty_dir.path())?;
    assert!(
        names(&render(&mut empty, 340., 16000., vec![]))
            .iter()
            .any(|s| s == "No backup codes yet")
    );
    click(&mut empty, "Add Backup Codes", 0)?;
    assert!(
        names(&render(&mut empty, 340., 16000., vec![]))
            .iter()
            .any(|s| s == "No accounts available")
    );
    click(&mut empty, "Add a 2FA account", 0)?;
    assert!(empty.review.is_some());
    assert_eq!(empty.state.route, Route::Codes);
    Ok(())
}
