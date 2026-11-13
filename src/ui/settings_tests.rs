//! Public generated fixtures; Context events are synthetic, not native input.
use super::*;
use crate::domain::{
    otp::{OtpSettings, Secret},
    records::*,
};
use crate::platform::startup::{self, Status};
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn settings_navigation_keeps_visible_heading_counts_and_first_action() -> TestResult {
    fn visible_text(shape: &egui::epaint::Shape, clip: egui::Rect, wanted: &str) -> bool {
        match shape {
            egui::epaint::Shape::Vec(shapes) => {
                shapes.iter().any(|s| visible_text(s, clip, wanted))
            }
            egui::epaint::Shape::Text(text) if text.galley.job.text == wanted => {
                clip.contains_rect(egui::Rect::from_min_size(text.pos, text.galley.size()))
            }
            _ => false,
        }
    }
    let dir = tempfile::tempdir()?;
    for (index, size) in [[340., 780.], [600., 800.], [960., 900.]]
        .into_iter()
        .enumerate()
    {
        let mut app = app(fixture(2)?, &dir.path().join(index.to_string()))?;
        click(&mut app, size, "Settings")?;
        let out = settle(&mut app, size);
        for wanted in [
            "Settings",
            "2FA Codes · 2 active account(s)",
            "Backup Codes · 2 active set(s) · 2 available · 2 used",
        ] {
            assert!(
                out.shapes
                    .iter()
                    .any(|s| visible_text(&s.shape, s.clip_rect, wanted)),
                "initial Settings navigation must keep visible summary text: {wanted}, width {}",
                size[0]
            );
        }
        let (_, export) = control(&out, "Export & Import").ok_or("missing export action")?;
        assert!(export.min.y >= 0. && export.max.y <= size[1] && painted_visible(&out, export));
        tab_to(&mut app, size, "Export & Import")?;
        let out = settle(&mut app, size);
        focused(&app, &out, "Export & Import", size)?;
    }
    Ok(())
}

#[test]
fn settings_each_lock_action_conceals_the_triggering_frame_without_an_extra_pass() -> TestResult {
    fn lock_controls(out: &egui::FullOutput) -> Vec<egui::Rect> {
        out.platform_output
            .accesskit_update
            .as_ref()
            .into_iter()
            .flat_map(|tree| &tree.nodes)
            .filter_map(|(_, node)| {
                if node.role() != egui::Role::Button || node.label() != Some("Lock vault") {
                    return None;
                }
                node.bounds().map(|b| {
                    egui::Rect::from_min_max(
                        egui::pos2(b.x0 as f32, b.y0 as f32),
                        egui::pos2(b.x1 as f32, b.y1 as f32),
                    )
                })
            })
            .collect()
    }
    fn private_label(text: &str) -> bool {
        text.contains("2 active account(s)")
            || text.contains("2 active set(s)")
            || text.contains("Saved choice:")
            || text == "5 minutes (default)"
            || text == "30 seconds (default)"
            || text == "Trash · 0 item(s)"
    }
    fn private_paint(shape: &egui::epaint::Shape) -> bool {
        match shape {
            egui::epaint::Shape::Vec(shapes) => shapes.iter().any(private_paint),
            egui::epaint::Shape::Text(text) => private_label(&text.galley.job.text),
            _ => false,
        }
    }
    let dir = tempfile::tempdir()?;
    for (size_index, size) in [[340., 780.], [600., 800.], [960., 900.]]
        .into_iter()
        .enumerate()
    {
        for pass_budget in [1, 2] {
            let mut target_index = 0;
            loop {
                let path = dir
                    .path()
                    .join(format!("{size_index}-{pass_budget}-{target_index}"));
                let mut app = app(fixture(2)?, &path)?;
                app.context.options_mut(|options| {
                    options.max_passes = std::num::NonZeroUsize::new(pass_budget)
                        .unwrap_or(std::num::NonZeroUsize::MIN);
                });
                app.state.navigate(Route::Settings);
                let mut output = settle(&mut app, size);
                let count = lock_controls(&output).len();
                assert!(count > 0, "Settings must retain an accessible Lock action");
                assert!(labels(&output).iter().any(|s| private_label(s)));
                let mut selected = None;
                for _ in 0..48 {
                    let target = lock_controls(&output).get(target_index).copied();
                    if let Some(rect) = target
                        && rect.min.y >= 0.
                        && rect.max.y < size[1] - 8.
                        && painted_visible(&output, rect)
                    {
                        selected = Some(rect);
                        break;
                    }
                    let delta = target.map_or(-200., |r| {
                        (size[1] / 2. - r.center().y).clamp(-2400., 2400.)
                    });
                    frame(
                        &mut app,
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
                    output = settle(&mut app, size);
                }
                let rect = selected.ok_or("actual Settings Lock action unavailable")?;
                frame(
                    &mut app,
                    size,
                    vec![
                        egui::Event::PointerMoved(rect.center()),
                        egui::Event::PointerButton {
                            pos: rect.center(),
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                );
                let clicked = frame(
                    &mut app,
                    size,
                    vec![
                        egui::Event::PointerMoved(rect.center()),
                        egui::Event::PointerButton {
                            pos: rect.center(),
                            button: egui::PointerButton::Primary,
                            pressed: false,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                );
                assert!(!app.state.session_active());
                assert!(
                    labels(&clicked).iter().all(|s| !private_label(s)),
                    "triggering locked AX frame retains Settings metadata; action={target_index}, width={}, pass_budget={pass_budget}",
                    rect.width()
                );
                assert!(
                    clicked.shapes.iter().all(|s| !private_paint(&s.shape)),
                    "triggering locked paint retains Settings metadata"
                );
                assert!(labels(&clicked).iter().any(|s| s == "Unlock vault"));
                target_index += 1;
                if target_index >= count {
                    break;
                }
            }
        }
    }
    Ok(())
}

#[test]
fn settings_counts_distinguish_own_trash_from_hidden_children_and_restore_exactly() -> TestResult {
    let mut vault = fixture(3)?;
    assert_eq!(
        settings::Counts::new(&vault),
        settings::Counts {
            accounts: 3,
            sets: 3,
            available: 3,
            used: 3,
            trash: 0
        }
    );
    own_trash(&mut vault.accounts[0].state);
    own_trash(&mut vault.backup_sets[1].state);
    own_trash(&mut vault.backup_sets[2].codes[0].state);
    assert_eq!(
        settings::Counts::new(&vault),
        settings::Counts {
            accounts: 2,
            sets: 1,
            available: 1,
            used: 0,
            trash: 3
        }
    );
    crate::domain::trash::Review::new(
        &vault,
        crate::domain::trash::Operation::Restore(crate::domain::trash::ItemId::Account(
            vault.accounts[0].id(),
        )),
    )?
    .apply(&mut vault, false, Timestamp(100))?;
    assert_eq!(
        settings::Counts::new(&vault),
        settings::Counts {
            accounts: 3,
            sets: 2,
            available: 2,
            used: 1,
            trash: 2
        }
    );
    vault.validate()?;
    Ok(())
}

#[test]
fn settings_context_widths_help_keyboard_startup_and_locked_privacy() -> TestResult {
    let dir = tempfile::tempdir()?;
    let mut app = app(fixture(2)?, dir.path())?;
    let (service, fake) = startup::testing::fake();
    app.settings.service = service;
    app.state.navigate(Route::Settings);
    for (index, size) in [[340., 780.], [600., 800.], [960., 900.]]
        .into_iter()
        .enumerate()
    {
        let out = settle(&mut app, size);
        let text = labels(&out);
        assert!(text.iter().any(|s| s == "2FA Codes · 2 active account(s)"));
        assert!(
            text.iter()
                .any(|s| s == "Backup Codes · 2 active set(s) · 2 available · 2 used")
        );
        no_secrets(&out);
        if index == 0 {
            click(&mut app, size, "Enable Auto Start")?;
            assert!(app.settings.pending_startup && app.state.busy());
            finish(&mut app, size)?;
            assert!(app.state.vault()?.preferences().auto_start());
            let out = settle(&mut app, size);
            focused(&app, &out, "Disable Auto Start", size)?;
        }
        if index == 1 {
            click(&mut app, size, "5 minutes (default)")?;
            click(&mut app, size, "15 minutes")?;
            assert!(app.settings.pending_settings && app.state.busy());
            finish(&mut app, size)?;
            assert_eq!(app.state.vault()?.preferences().lock_timeout_seconds(), 900);
            let out = settle(&mut app, size);
            focused(&app, &out, "15 minutes", size)?;
            click(&mut app, size, "30 seconds (default)")?;
            tab_to(&mut app, size, "60 seconds")?;
            key(&mut app, size, egui::Key::Enter);
            assert!(app.settings.pending_settings && app.state.busy());
            finish(&mut app, size)?;
            assert_eq!(
                app.state.vault()?.preferences().clipboard_timeout_seconds(),
                60
            );
            let out = settle(&mut app, size);
            focused(&app, &out, "60 seconds", size)?;
        }
        click(&mut app, size, "Windows time help")?;
        let out = settle(&mut app, size);
        assert!(
            labels(&out)
                .iter()
                .any(|s| s.contains("does not change the system clock"))
        );
        focused(&app, &out, "Close", size)?;
        key(&mut app, size, egui::Key::Escape);
        let out = settle(&mut app, size);
        focused(&app, &out, "Windows time help", size)?;
        tab_to(&mut app, size, "Backup codes help")?;
        key(&mut app, size, egui::Key::Enter);
        let out = settle(&mut app, size);
        assert!(
            labels(&out)
                .iter()
                .any(|s| s.contains("Copy never marks a code used"))
        );
        key(&mut app, size, egui::Key::Escape);
    }
    for (label, wanted) in [
        ("QR and manual entry help", "HOTP and proprietary"),
        ("Encrypted backup and recovery help", "native .authpbk"),
        ("Trash and deletion help", "no automatic expiry"),
        ("Passphrase and locking help", "no passphrase reset"),
    ] {
        click(&mut app, [340., 780.], label)?;
        let out = settle(&mut app, [340., 780.]);
        assert!(
            labels(&out).iter().any(|s| s.contains(wanted)),
            "topic {label}"
        );
        no_secrets(&out);
        click(&mut app, [340., 780.], "Close")?;
    }
    assert_eq!(
        *fake.changes.lock().unwrap_or_else(|e| e.into_inner()),
        vec![true]
    );
    app.lock();
    let out = settle(&mut app, [340., 780.]);
    assert!(labels(&out).iter().all(|s| !s.contains("2 active account")
        && !s.contains("Saved choice")
        && !s.contains("Public identical")));
    no_secrets(&out);
    assert!(app.settings.actual.is_none() && !app.help_open);
    Ok(())
}

#[test]
fn settings_atomic_failure_retry_and_reopen_keep_newest_codes_intent() -> TestResult {
    let dir = tempfile::tempdir()?;
    let mut app = app(fixture(2)?, dir.path())?;
    let size = [340., 780.];
    app.state.navigate(Route::Settings);
    settle(&mut app, size);
    app.state.query = "latest settings intent".into();
    let prefs = app.codes_preferences.as_mut().ok_or("preferences")?;
    prefs.query = Zeroizing::new(app.state.query.clone());
    prefs.sort = SortOrder::AccountDescending;
    prefs.view = ViewMode::Grid;
    app.preferences_changed();
    app.state
        .repository_mut()?
        .fail_next_backup_write_for_test(true);
    let before = std::fs::read(dir.path().join("vault.bin"))?;
    app.start_security_settings(900, 60);
    assert!(app.state.busy());
    finish(&mut app, size)?;
    assert_eq!(std::fs::read(dir.path().join("vault.bin"))?, before);
    assert_eq!(app.state.vault()?.preferences().lock_timeout_seconds(), 300);
    assert!(app.preferences_dirty.is_some());
    assert!(app.settings.feedback.is_some_and(|(_, error)| error));
    app.state
        .repository_mut()?
        .fail_next_backup_write_for_test(false);
    app.start_security_settings(900, 60);
    finish(&mut app, size)?;
    assert!(app.preferences_dirty.is_none());
    app.lock();
    app.state.repository_mut()?.unlock(pass())?;
    let p = app.state.vault()?.preferences();
    assert_eq!(
        (
            p.lock_timeout_seconds(),
            p.clipboard_timeout_seconds(),
            p.auto_start()
        ),
        (900, 60, false)
    );
    assert_eq!(
        (p.codes().query.as_str(), p.sort(), p.view()),
        (
            "latest settings intent",
            SortOrder::AccountDescending,
            ViewMode::Grid
        )
    );
    assert_eq!(app.state.vault()?.accounts().len(), 2);
    for sentinel in [
        b"latest settings intent".as_slice(),
        b"public-settings-code-",
    ] {
        assert!(
            !std::fs::read(dir.path().join("vault.bin"))?
                .windows(sentinel.len())
                .any(|w| w == sentinel)
        );
    }
    Ok(())
}

#[test]
fn settings_startup_reconciliation_is_readonly_and_os_failure_retry_is_truthful() -> TestResult {
    let dir = tempfile::tempdir()?;
    let mut vault = fixture(1)?;
    vault.set_auto_start(true, Timestamp(100))?;
    let mut app = app(vault, dir.path())?;
    let (service, fake) = startup::testing::fake();
    app.settings.service = service;
    app.state.navigate(Route::Settings);
    let size = [600., 800.];
    let before = std::fs::read(dir.path().join("vault.bin"))?;
    let out = settle(&mut app, size);
    assert!(
        fake.changes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty()
    );
    assert_eq!(std::fs::read(dir.path().join("vault.bin"))?, before);
    assert!(
        labels(&out)
            .iter()
            .any(|s| s.contains("saved choice and Windows registration differ"))
    );
    *fake.fail_change.lock().unwrap_or_else(|e| e.into_inner()) = true;
    click(&mut app, size, "Apply saved startup choice")?;
    finish(&mut app, size)?;
    assert!(app.state.vault()?.preferences().auto_start());
    assert!(app.settings.feedback.is_some_and(|(_, e)| e));
    assert_eq!(app.settings.actual, Some(Ok(Status::Missing)));
    *fake.fail_change.lock().unwrap_or_else(|e| e.into_inner()) = false;
    click(&mut app, size, "Apply saved startup choice")?;
    finish(&mut app, size)?;
    assert_eq!(app.settings.actual, Some(Ok(Status::Registered)));
    app.state
        .repository_mut()?
        .fail_next_backup_write_for_test(true);
    let count = fake.changes.lock().unwrap_or_else(|e| e.into_inner()).len();
    click(&mut app, size, "Disable Auto Start")?;
    finish(&mut app, size)?;
    assert!(app.state.vault()?.preferences().auto_start());
    assert_eq!(
        fake.changes.lock().unwrap_or_else(|e| e.into_inner()).len(),
        count
    );
    app.state
        .repository_mut()?
        .fail_next_backup_write_for_test(false);
    click(&mut app, size, "Disable Auto Start")?;
    finish(&mut app, size)?;
    assert!(!app.state.vault()?.preferences().auto_start());
    assert_eq!(app.settings.actual, Some(Ok(Status::Missing)));
    app.state
        .repository_mut()?
        .transaction(|v| v.set_auto_start(true, Timestamp(100)))?;
    app.lock();
    app.state.repository_mut()?.unlock(pass())?;
    settle(&mut app, size);
    assert_eq!(
        fake.changes.lock().unwrap_or_else(|e| e.into_inner()).len(),
        count + 1,
        "unlock cannot automatically repair stale desired preference"
    );
    Ok(())
}

#[test]
fn settings_lock_during_admitted_preference_commit_never_registers_startup_late() -> TestResult {
    let dir = tempfile::tempdir()?;
    let mut app = app(fixture(1)?, dir.path())?;
    let (service, fake) = startup::testing::fake();
    app.settings.service = service;
    let size = [340., 780.];
    app.state.navigate(Route::Settings);
    settle(&mut app, size);
    let gate = Arc::new(crate::storage::test_gate::CommitGate::default());
    let release = gate.release_on_drop();
    app.state.hold_next_save_after_admission(gate.clone());
    app.start_startup_change(true);
    assert!(gate.wait_entered(Duration::from_secs(10)));
    let started = Instant::now();
    app.lock();
    let out = frame(&mut app, size, vec![]);
    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(!gate.is_released());
    assert!(
        labels(&out)
            .iter()
            .all(|s| !s.contains("Saved choice") && !s.contains("Public identical"))
    );
    drop(release);
    finish(&mut app, size)?;
    assert!(
        fake.changes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty()
    );
    assert!(!app.state.is_unlocked() && app.state.take_startup_result().is_none());
    app.state.repository_mut()?.unlock(pass())?;
    assert!(
        app.state.vault()?.preferences().auto_start(),
        "already admitted whole desired-preference commit may finish"
    );
    assert_eq!(app.state.vault()?.accounts().len(), 1);
    Ok(())
}

#[test]
fn settings_close_keeps_latest_preferences_without_triggering_startup_and_clock_observation_spans_routes()
-> TestResult {
    use std::sync::atomic::AtomicU64;
    struct TestClock(Arc<AtomicU64>);
    impl Clock for TestClock {
        fn unix_seconds(&self) -> Result<u64, crate::error::AppError> {
            Ok(self.0.load(Ordering::SeqCst))
        }
    }
    let dir = tempfile::tempdir()?;
    let mut app = app(fixture(1)?, dir.path())?;
    let (service, fake) = startup::testing::fake();
    app.settings.service = service;
    let clock = Arc::new(AtomicU64::new(59));
    app.wall_clock = Box::new(TestClock(clock.clone()));
    let size = [600., 800.];
    app.state.navigate(Route::Settings);
    settle(&mut app, size);
    clock.store(120, Ordering::SeqCst);
    let out = settle(&mut app, size);
    assert!(
        app.clock.shifted
            && labels(&out)
                .iter()
                .any(|s| s.contains("clock changed unexpectedly"))
    );
    app.state.navigate(Route::Codes);
    let out = settle(&mut app, size);
    let draft = app.state.vault()?.accounts()[0].credential();
    let current = otp::current_code(
        draft.secret(),
        draft.settings(),
        &super::super::codes::FrameClock(Ok(120)),
    )?;
    assert!(labels(&out).contains(&super::super::codes::formatted(current.expose_value())));
    app.state.navigate(Route::Settings);
    settle(&mut app, size);
    app.state.query = "close newest settings query".into();
    let p = app.codes_preferences.as_mut().ok_or("prefs")?;
    p.query = Zeroizing::new(app.state.query.clone());
    p.view = ViewMode::Grid;
    app.preferences_changed();
    click(&mut app, size, "Encrypted backup and recovery help")?;
    request_close(&mut app, size);
    finish_close(&mut app, size)?;
    assert!(
        fake.changes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty()
    );
    app.state.repository_mut()?.unlock(pass())?;
    assert_eq!(
        app.state.vault()?.preferences().codes().query.as_str(),
        "close newest settings query"
    );
    assert_eq!(app.state.vault()?.preferences().view(), ViewMode::Grid);
    Ok(())
}
#[test]
fn settings_failed_action_does_not_block_private_close_flush_and_retry() -> TestResult {
    let dir = tempfile::tempdir()?;
    let mut app = app(fixture(1)?, dir.path())?;
    let size = [340., 780.];
    app.state.navigate(Route::Settings);
    settle(&mut app, size);
    app.state.query = "latest query after failed settings".into();
    app.codes_preferences.as_mut().ok_or("prefs")?.query = Zeroizing::new(app.state.query.clone());
    app.preferences_changed();
    app.state
        .repository_mut()?
        .fail_next_backup_write_for_test(true);
    app.start_security_settings(900, 60);
    finish(&mut app, size)?;
    assert!(app.settings.feedback.is_some_and(|(_, error)| error));
    let out = request_close(&mut app, size);
    assert!(
        labels(&out)
            .iter()
            .all(|s| !s.contains("latest query after failed settings")
                && !s.contains("Public identical"))
    );
    assert!(
        app.state.busy(),
        "prior Settings failure must not strand Close before its eligible preference flush"
    );
    finish(&mut app, size)?;
    assert!(app.preferences_error && app.closing && !app.exit_approved);
    app.state
        .repository_mut()?
        .fail_next_backup_write_for_test(false);
    click(&mut app, size, "Retry saving display changes")?;
    finish_close(&mut app, size)?;
    app.state.repository_mut()?.unlock(pass())?;
    let p = app.state.vault()?.preferences();
    assert_eq!(
        (p.lock_timeout_seconds(), p.clipboard_timeout_seconds()),
        (300, 30)
    );
    assert_eq!(
        p.codes().query.as_str(),
        "latest query after failed settings"
    );
    Ok(())
}

fn pass() -> Zeroizing<String> {
    Zeroizing::new("disposable-settings-fixture".into())
}

#[test]
fn settings_held_save_preserves_even_newer_codes_intent_before_encrypted_reopen() -> TestResult {
    let dir = tempfile::tempdir()?;
    let mut app = app(fixture(1)?, dir.path())?;
    let size = [340., 780.];
    app.state.navigate(Route::Settings);
    settle(&mut app, size);
    app.state.query = "first submitted query".into();
    app.codes_preferences.as_mut().ok_or("prefs")?.query = Zeroizing::new(app.state.query.clone());
    app.preferences_changed();
    let gate = Arc::new(crate::storage::test_gate::CommitGate::default());
    let release = gate.release_on_drop();
    app.state.hold_next_save_after_admission(gate.clone());
    app.start_security_settings(900, 60);
    assert!(gate.wait_entered(Duration::from_secs(10)));
    app.state.query = "even newer live query".into();
    let p = app.codes_preferences.as_mut().ok_or("prefs")?;
    p.query = Zeroizing::new(app.state.query.clone());
    p.view = ViewMode::Grid;
    p.sort = SortOrder::ServiceDescending;
    app.preferences_changed();
    drop(release);
    finish(&mut app, size)?;
    assert_eq!(app.state.query, "even newer live query");
    assert_eq!(
        app.codes_preferences
            .as_ref()
            .ok_or("prefs")?
            .query
            .as_str(),
        "even newer live query"
    );
    app.preferences_dirty = Instant::now().checked_sub(Duration::from_secs(1));
    frame(&mut app, size, vec![]);
    finish(&mut app, size)?;
    app.lock();
    app.state.repository_mut()?.unlock(pass())?;
    let p = app.state.vault()?.preferences();
    assert_eq!(
        (p.lock_timeout_seconds(), p.clipboard_timeout_seconds()),
        (900, 60)
    );
    assert_eq!(
        (p.codes().query.as_str(), p.view(), p.sort()),
        (
            "even newer live query",
            ViewMode::Grid,
            SortOrder::ServiceDescending
        )
    );
    Ok(())
}

#[test]
fn settings_inaccessible_or_conflicting_startup_can_save_off_without_claiming_removal() -> TestResult
{
    let dir = tempfile::tempdir()?;
    let mut app = app(fixture(1)?, dir.path())?;
    let size = [340., 780.];
    app.state.navigate(Route::Settings);
    for status in [Err(startup::Error::Unavailable), Ok(Status::Conflict)] {
        app.state
            .repository_mut()?
            .transaction(|v| v.set_auto_start(true, Timestamp(100)))?;
        app.settings.reset();
        let (service, fake) = startup::testing::fake();
        *fake.status.lock().unwrap_or_else(|e| e.into_inner()) = status;
        *fake.fail_change.lock().unwrap_or_else(|e| e.into_inner()) = status.is_err();
        app.settings.service = service;
        settle(&mut app, size);
        assert_eq!(app.settings.actual, Some(status));
        assert!(
            fake.changes
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_empty()
        );
        click(&mut app, size, "Disable Auto Start")?;
        finish(&mut app, size)?;
        assert!(!app.state.vault()?.preferences().auto_start());
        assert!(app.settings.feedback.is_some_and(|(_, error)| error));
        assert_eq!(app.settings.actual, Some(status));
        assert_eq!(
            *fake.changes.lock().unwrap_or_else(|e| e.into_inner()),
            vec![false]
        );
        let out = settle(&mut app, size);
        assert!(
            labels(&out)
                .iter()
                .any(|s| s.contains("does not guarantee removal"))
        );
        click(&mut app, size, "Check startup registration")?;
        settle(&mut app, size);
        assert_eq!(
            *fake.changes.lock().unwrap_or_else(|e| e.into_inner()),
            vec![false]
        );
        assert_eq!(
            *fake.status.lock().unwrap_or_else(|e| e.into_inner()),
            status
        );
    }
    Ok(())
}

#[test]
fn settings_import_replace_keeps_local_startup_and_rechecks_without_os_side_effects() -> TestResult
{
    let dir = tempfile::tempdir()?;
    let mut incoming = fixture(2)?;
    incoming.set_auto_start(true, Timestamp(100))?;
    incoming.set_lock_timeout(60, Timestamp(100))?;
    incoming.set_clipboard_timeout(15, Timestamp(100))?;
    let mut source = VaultStore::open_directory(dir.path().join("source"))?;
    source.create_with_parameters(
        pass(),
        incoming,
        crate::storage::KdfParameters {
            memory_kib: 65536,
            iterations: 3,
            lanes: 1,
        },
    )?;
    let cancel = crate::storage::OperationCancellation::default();
    let backup = dir.path().join("public-settings.authpbk");
    crate::backup::export(
        &source,
        crate::backup::Destination::create_new(&backup)?,
        pass(),
        &cancel,
    )?;
    let mut app = app(fixture(1)?, &dir.path().join("destination"))?;
    let (service, fake) = startup::testing::fake();
    app.settings.service = service;
    let size = [600., 800.];
    app.state.navigate(Route::Settings);
    settle(&mut app, size);
    let preview = crate::backup::preview(app.state.repository_mut()?, &backup, pass(), &cancel)?;
    let plan = preview.plan(crate::backup::ImportOptions {
        mode: crate::backup::ImportMode::Replace,
        import_security_preferences: true,
        ..Default::default()
    })?;
    app.backup_import = Some(ImportForm::new(egui::Id::unique("public-settings-import")));
    let watch = app.lifecycle.as_ref().map_err(|e| *e)?.watch();
    app.state.begin_backup(
        super::super::state::BackupRequest::Apply(Box::new(plan)),
        watch,
        || {},
    )?;
    finish(&mut app, size)?;
    assert!(app.backup_import.is_none());
    assert!(!app.state.vault()?.preferences().auto_start());
    assert_eq!(app.state.vault()?.preferences().lock_timeout_seconds(), 60);
    assert_eq!(
        app.state.vault()?.preferences().clipboard_timeout_seconds(),
        15
    );
    assert_eq!(app.state.vault()?.accounts().len(), 2);
    app.state.navigate(Route::Settings);
    let out = settle(&mut app, size);
    assert!(
        labels(&out)
            .iter()
            .any(|s| s == "2FA Codes · 2 active account(s)")
    );
    assert!(
        fake.changes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty()
    );
    assert_eq!(app.settings.actual, Some(Ok(Status::Missing)));
    app.lock();
    app.state.repository_mut()?.unlock(pass())?;
    assert!(!app.state.vault()?.preferences().auto_start());
    Ok(())
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
        let mut code = BackupCode::new(&format!("public-settings-code-{i}-used"), Timestamp(100))?;
        code.set_used(true, Timestamp(100))?;
        v.add_backup_set(
            id,
            vec![
                code,
                BackupCode::new(
                    &format!("public-settings-code-{i}-available"),
                    Timestamp(100),
                )?,
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
            if n.label().or(n.value()) != Some(name)
                || !matches!(
                    n.role(),
                    egui::Role::Button
                        | egui::Role::Tab
                        | egui::Role::CheckBox
                        | egui::Role::ComboBox
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
fn click(app: &mut AuthenticatorApp, size: [f32; 2], name: &str) -> TestResult {
    let mut output = settle(app, size);
    for _ in 0..48 {
        if let Some((_, rect)) = control(&output, name)
            && rect.min.y >= 0.
            && rect.max.y < size[1] - 8.
            && !painted_visible(&output, rect)
        {
            frame(app, size, vec![egui::Event::PointerMoved(rect.center())]);
            output = settle(app, size);
        }
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
    if !painted_visible(out, rect) {
        fn collect(
            shape: &egui::epaint::Shape,
            rect: egui::Rect,
            clip: egui::Rect,
            matches: &mut Vec<(egui::Rect, egui::Rect)>,
        ) {
            match shape {
                egui::epaint::Shape::Vec(shapes) => {
                    for s in shapes {
                        collect(s, rect, clip, matches);
                    }
                }
                egui::epaint::Shape::Rect(s) if s.rect.center().distance(rect.center()) < 2. => {
                    matches.push((s.rect, clip))
                }
                _ => {}
            }
        }
        let mut clips = Vec::new();
        for s in &out.shapes {
            collect(&s.shape, rect, s.clip_rect, &mut clips);
        }
        panic!("focus must be inside its painted clip: {name} {rect:?} candidates {clips:?}");
    }
    Ok(())
}
fn no_secrets(out: &egui::FullOutput) {
    assert!(
        labels(out)
            .iter()
            .all(|s| !s.contains("public-settings-code-") && !s.contains("GEZDGNBV"))
    );
    assert!(
        out.platform_output
            .commands
            .iter()
            .all(|c| !matches!(c, egui::OutputCommand::CopyText(_)))
    );
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

#[test]
fn initial_subpage_focus_keeps_heading_and_first_control_visible() -> TestResult {
    fn text_visible(out: &egui::FullOutput, wanted: &str) -> bool {
        fn find(shape: &egui::epaint::Shape, clip: egui::Rect, wanted: &str) -> bool {
            match shape {
                egui::epaint::Shape::Vec(shapes) => shapes.iter().any(|s| find(s, clip, wanted)),
                egui::epaint::Shape::Text(t) if t.galley.job.text == wanted => {
                    clip.contains_rect(egui::Rect::from_min_size(t.pos, t.galley.size()))
                }
                _ => false,
            }
        }
        out.shapes
            .iter()
            .any(|s| find(&s.shape, s.clip_rect, wanted))
    }
    let dir = tempfile::tempdir()?;
    for (index, width) in [340., 600., 960.].into_iter().enumerate() {
        let mut app = app(fixture(2)?, &dir.path().join(index.to_string()))?;
        for height in [780., 666., 500.] {
            for screen in 0..5 {
                // Fresh Context gives each initial page the same unscrolled viewport.
                // The assertion concerns visible paint, not AX nodes outside a clip.
                app.context = egui::Context::default();
                app.context.enable_accesskit();
                app.folders = None;
                app.trash = None;
                app.backup_codes.reset();
                app.state.navigate(Route::Codes);
                let (heading, first) = match screen {
                    0 => {
                        click(&mut app, [width, height], "Manage folders")?;
                        ("Folders", "Back to 2FA Codes")
                    }
                    1 | 2 => {
                        click(&mut app, [width, height], "Manage folders")?;
                        let (action, title) = if screen == 1 {
                            ("Create folder", "Create folder")
                        } else {
                            ("Edit folder", "Edit folder")
                        };
                        click(&mut app, [width, height], action)?;
                        (title, "Folder name")
                    }
                    3 => {
                        app.state.navigate(Route::BackupCodes);
                        settle(&mut app, [width, height]);
                        // Reach detail through its real scrolled list.
                        click(&mut app, [width, height], "View set")?;
                        ("Backup Codes", "Back")
                    }
                    _ => {
                        click(&mut app, [width, height], "Settings")?;
                        click(&mut app, [width, height], "Trash · 0 item(s)")?;
                        ("Trash", "Back to Settings")
                    }
                };
                let out = settle(&mut app, [width, height]);
                assert!(
                    text_visible(&out, heading),
                    "initial {heading} heading clipped at {width}x{height}"
                );
                assert!(
                    text_visible(&out, first),
                    "initial {heading} first control label clipped at {width}x{height}"
                );
            }
        }
    }
    Ok(())
}
