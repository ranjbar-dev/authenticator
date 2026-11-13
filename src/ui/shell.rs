//! Inventory-derived shell and local vault lifecycle.
use super::backup_codes::{self, Session as BackupSession};
use super::backup_import::{self, Busy as ImportBusy, ImportForm};
use super::codes;
use super::export::{self, ExportForm, Picker};
use super::import_review::ImportReview;
use super::recovery_editor::{self, Editor as RecoveryEditor};
use super::{
    components as c,
    state::{AppState, CaptureEvent, Route, SaveIntent, ScanAlternative, VaultAction, Workflow},
    theme,
};
use crate::{
    domain::{
        clock::{Clock, SystemClock},
        otp,
        records::{
            AccountId, CodesPreferences, DeletionState, FolderFilter, RecordState, ViewMode,
        },
    },
    storage::VaultStore,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
type ImageResult = Result<Option<crate::qr::ScanReview>, crate::qr::QrError>;
enum CodeAction {
    Copy(AccountId),
    Edit(AccountId),
    Delete(AccountId, RecordState, egui::Id),
}
struct ImageJob {
    generation: u64,
    cancelled: Arc<AtomicBool>,
    result: Arc<Mutex<Option<ImageResult>>>,
}
impl Drop for ImageJob {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.result.lock().unwrap_or_else(|e| e.into_inner()).take();
    }
}
use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::time::{Duration, Instant};
use zeroize::{Zeroize, Zeroizing};

pub struct AuthenticatorApp {
    pub state: AppState,
    dialog: Option<c::Dialog>,
    help_open: bool,
    status: Option<&'static str>,
    context: egui::Context,
    lifecycle: Result<crate::platform::lifecycle::Lifecycle, crate::error::AppError>,
    passphrase: Zeroizing<String>,
    confirmation: Zeroizing<String>,
    form_error: Option<String>,
    exists: bool,
    focus_form: bool,
    operation_cancelled: bool,
    owner: usize,
    capture_message: Option<String>,
    capture_failed: bool,
    capture_focus: bool,
    capture_preparing: Option<(u64, bool)>,
    review: Option<ImportReview>,
    export_hub: bool,
    export_form: Option<ExportForm>,
    export_picker: Option<Picker>,
    export_focus: bool,
    backup_import: Option<ImportForm>,
    backup_picker: Option<backup_import::Picker>,
    backup_codes: BackupSession,
    recovery_editor: Option<RecoveryEditor>,
    image: Option<ImageJob>,
    imported: Vec<AccountId>,
    show_imported: bool,
    clipboard: crate::platform::clipboard::Clipboard,
    codes_preferences: Option<CodesPreferences>,
    preferences_dirty: Option<Instant>,
    preferences_error: bool,
    pending_mutation: bool,
    pending_preferences: bool,
    ordered_ids: Option<Vec<AccountId>>,
    delete: Option<(AccountId, RecordState, c::Dialog)>,
    clock: codes::ClockTracker,
    wall_clock: Box<dyn Clock + Send + Sync>,
    clipboard_feedback: Option<(&'static str, Instant)>,
    closing: bool,
    close_locked: bool,
    close_cleanup_deadline: Option<Instant>,
    exit_approved: bool,
    #[cfg(debug_assertions)]
    evidence_code_bounds: Vec<egui::Rect>,
    #[cfg(debug_assertions)]
    evidence_copy_queued: Option<bool>,
}
impl AuthenticatorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::with_repository(cc, VaultStore::open_default())
    }
    /// Explicit repository injection also permits isolated example/test storage.
    pub fn with_repository(
        cc: &eframe::CreationContext<'_>,
        repository: Result<VaultStore, crate::error::AppError>,
    ) -> Self {
        let owner = cc
            .window_handle()
            .ok()
            .and_then(|h| match h.as_raw() {
                RawWindowHandle::Win32(w) => Some(w.hwnd.get() as usize),
                _ => None,
            })
            .unwrap_or(0);
        Self::with_context(cc.egui_ctx.clone(), repository, owner)
    }
    fn with_context(
        context: egui::Context,
        repository: Result<VaultStore, crate::error::AppError>,
        owner: usize,
    ) -> Self {
        theme::apply(&context);
        let state = AppState::new(repository);
        let exists = state.vault_exists().unwrap_or(true);
        let repaint = context.clone();
        let clipboard_repaint = context.clone();
        Self {
            state,
            dialog: None,
            help_open: false,
            status: None,
            lifecycle: crate::platform::lifecycle::Lifecycle::start(move || {
                repaint.request_repaint()
            }),
            context,
            passphrase: Zeroizing::new(String::new()),
            confirmation: Zeroizing::new(String::new()),
            form_error: None,
            exists,
            focus_form: true,
            operation_cancelled: false,
            owner,
            capture_message: None,
            capture_failed: false,
            capture_focus: false,
            capture_preparing: None,
            review: None,
            export_hub: false,
            export_form: None,
            export_picker: None,
            export_focus: false,
            backup_import: None,
            backup_picker: None,
            backup_codes: BackupSession::default(),
            recovery_editor: None,
            image: None,
            imported: Vec::new(),
            show_imported: false,
            clipboard: crate::platform::clipboard::Clipboard::new(owner, move || {
                clipboard_repaint.request_repaint()
            }),
            codes_preferences: None,
            preferences_dirty: None,
            preferences_error: false,
            pending_mutation: false,
            pending_preferences: false,
            ordered_ids: None,
            delete: None,
            clock: codes::ClockTracker::default(),
            wall_clock: Box::new(SystemClock),
            clipboard_feedback: None,
            closing: false,
            close_locked: false,
            close_cleanup_deadline: None,
            exit_approved: false,
            #[cfg(debug_assertions)]
            evidence_code_bounds: Vec::new(),
            #[cfg(debug_assertions)]
            evidence_copy_queued: None,
        }
    }
    pub fn lock(&mut self) {
        self.recovery_editor = None;
        self.backup_codes.reset();
        #[cfg(debug_assertions)]
        {
            self.evidence_copy_queued = None;
        }
        if self.closing {
            self.close_locked = true;
            self.close_cleanup_deadline
                .get_or_insert(Instant::now() + Duration::from_millis(300));
        }
        #[cfg(debug_assertions)]
        self.evidence_code_bounds.clear();
        let restore_capture = self.capture_preparing.is_some() || self.state.capture_busy();
        self.clipboard.clear();
        self.clipboard_feedback = None;
        self.codes_preferences = None;
        self.preferences_dirty = None;
        self.preferences_error = false;
        self.pending_mutation = false;
        self.pending_preferences = false;
        self.ordered_ids = None;
        self.delete = None;
        self.clock.clear();
        egui::Popup::close_all(&self.context);
        self.review = None;
        self.export_form = None;
        self.export_picker = None;
        self.export_hub = false;
        self.export_focus = false;
        self.backup_import = None;
        self.backup_picker = None;
        self.image = None;
        self.state.finish_image();
        self.imported.clear();
        self.show_imported = false;
        self.dialog = None;
        self.help_open = false;
        self.status = None;
        self.clear_form();
        self.form_error = None;
        self.focus_form = true;
        self.capture_message = None;
        self.capture_failed = false;
        self.capture_preparing = None;
        self.state.lock();
        self.operation_cancelled = self.state.busy();
        // ROOT will render the locked state before the visibility command is
        // applied. Native overlay never restores an old unlocked framebuffer.
        if restore_capture {
            self.context
                .send_viewport_cmd(egui::ViewportCommand::Visible(true));
        }
        self.context.request_repaint();
        // One public one-shot hidden pass removes old AccessKit nodes and the
        // cached unlocked framebuffer without showing/unminimizing the window.
        self.context
            .send_viewport_cmd(egui::ViewportCommand::RequestPaintWhileHidden);
    }
    fn clear_form(&mut self) {
        self.passphrase.zeroize();
        self.confirmation.zeroize();
        // Includes native TextEdit undo/cursor memory for search and dialogs.
        // Secret editors additionally drop undo history after every edit pass.
        c::clear_sensitive_widget_memory(&self.context);
    }
    pub fn show(&mut self, root: &mut egui::Ui) {
        #[cfg(debug_assertions)]
        self.evidence_code_bounds.clear();
        let ctx = root.ctx().clone();
        if self.state.route != Route::BackupCodes || !self.state.session_active() {
            self.backup_codes.reset();
            self.recovery_editor = None;
        }
        let now = Instant::now();
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        let was_closing = self.closing;
        if close_requested && !self.closing && !self.exit_approved {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.closing = true;
            self.clipboard.clear();
            self.clipboard_feedback = None;
            self.clear_form();
            let can_flush = (self.preferences_dirty.is_some() || self.pending_preferences)
                && self.review.is_none()
                && self.export_form.is_none()
                && self.export_picker.is_none()
                && self.backup_import.is_none()
                && self.backup_picker.is_none()
                && self.image.is_none()
                && self.capture_preparing.is_none()
                && !self.state.capture_busy()
                && (!self.state.busy() || self.pending_preferences);
            if !can_flush {
                self.lock();
            }
        }
        // Match the key event's modifiers. A complete physical chord can be
        // pressed/released in one frame, leaving the final modifiers empty.
        let lock_shortcut = ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::L,
            ))
        });
        if self.lifecycle.as_ref().is_ok_and(|l| l.take_lock())
            || self.state.idle_due(now)
            || ctx.input(|i| {
                (close_requested && (!self.closing || was_closing))
                    || lock_shortcut
                    || (i.key_pressed(egui::Key::Escape) && self.state.busy())
            })
        {
            self.lock();
            if self.closing {
                self.closing = false;
                self.exit_approved = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        let was_importing = self.state.importing();
        let save_mode = self.review.as_ref().map(|r| r.mode);
        if let Some(outcome) = self.state.poll(now) {
            self.ordered_ids = None;
            self.backup_codes.invalidate();
            if let Some(imported) = self.state.take_backup_result() {
                self.complete_backup_import(imported);
            } else if let Some(exported) = self.state.take_exported() {
                if let Some(form) = &mut self.export_form {
                    form.complete(exported, &ctx);
                }
            } else if self.recovery_editor.as_ref().is_some_and(|e| e.saving) {
                if outcome.is_ok() {
                    let editor = self.recovery_editor.take();
                    if let Some(editor) = editor {
                        self.backup_codes.screen = if editor.deleting {
                            backup_codes::Screen::List
                        } else {
                            match self.state.vault().ok().and_then(|v| {
                                v.backup_sets()
                                    .iter()
                                    .find(|s| s.account_id() == editor.target.account_id())
                                    .map(|s| s.id())
                            }) {
                                Some(set) => backup_codes::Screen::Detail {
                                    account: editor.target.account_id(),
                                    set,
                                    from_selection: false,
                                },
                                None => backup_codes::Screen::List,
                            }
                        };
                    }
                    self.backup_codes.focus = true;
                    c::clear_sensitive_widget_memory(&ctx);
                    self.status = Some("Backup-code changes saved in your encrypted vault.");
                } else if let Some(editor) = &mut self.recovery_editor {
                    editor.saving = false;
                    editor.deleting = false;
                    editor.error = Some("Changes could not be saved. Your input and previous set are kept. Retry Save or cancel.".into());
                    editor.focus = true;
                }
            } else if self.pending_mutation {
                self.pending_mutation = false;
                self.preferences_error = self.pending_preferences && outcome.is_err();
                self.status = Some(if outcome.is_ok() {
                    "Changes saved in your encrypted vault."
                } else {
                    "Changes could not be saved. Your previous vault is unchanged. Retry the action."
                });
                if self.preferences_error {
                    self.preferences_dirty = Some(now);
                }
                self.pending_preferences = false;
            } else if was_importing {
                match outcome {
                    Ok(()) => {
                        self.imported = self.state.take_imported().unwrap_or_default();
                        self.review = None;
                        c::clear_sensitive_widget_memory(&ctx);
                        self.state.navigate(Route::Codes);
                        self.capture_message = Some(
                            match save_mode {
                                Some(super::import_review::ReviewMode::Edit { .. }) => {
                                    "Account changes saved in your encrypted vault."
                                }
                                Some(super::import_review::ReviewMode::Create) => {
                                    "Account saved in your encrypted vault."
                                }
                                _ => "Account import saved in your encrypted vault.",
                            }
                            .into(),
                        );
                        self.capture_failed = false;
                        self.capture_focus = false;
                        self.show_imported = true;
                    }
                    Err(error) => {
                        if let Some(review) = &mut self.review {
                            review.error = Some(format!(
                                "{error} Your reviewed edits are kept. Retry Save or cancel."
                            ));
                            review.show_error = true;
                        }
                    }
                }
            } else {
                self.clear_form();
                self.focus_form = true;
                self.form_error = outcome.err().map(|e| e.to_string());
                self.operation_cancelled = false;
            }
        }
        if (self.export_hub || self.export_form.is_some() || self.backup_import.is_some())
            && !self.state.session_active()
        {
            self.lock();
        }
        if let Some(editor) = &mut self.recovery_editor {
            let had_job = editor.job.is_some();
            editor.poll(self.state.generation(), self.state.is_unlocked(), &ctx);
            if had_job && editor.job.is_none() {
                self.state.finish_image();
            }
        }
        if let Some(picker) = &self.backup_picker
            && let Some(selection) = picker.poll()
        {
            let current = picker.generation == self.state.generation() && self.state.is_unlocked();
            self.backup_picker = None;
            self.state.finish_image();
            if current && let Some(form) = &mut self.backup_import {
                match selection {
                    Ok(Some(path)) => {
                        form.reset_selection();
                        form.path = Some(path);
                    }
                    Ok(None) => {}
                    Err(error) => form.error = Some(backup_import::error_message(error)),
                }
                form.focus = true;
            }
        }
        if let Some(picker) = &self.export_picker
            && let Some(selection) = picker.poll()
        {
            let current = picker.generation == self.state.generation() && self.state.is_unlocked();
            self.export_picker = None;
            self.state.finish_image();
            if current && let Some(form) = &mut self.export_form {
                match selection {
                    Ok(Some((path, destination))) => {
                        form.path = Some(path);
                        form.destination = Some(destination);
                        form.error = None;
                        form.success = false;
                    }
                    Ok(None) => {}
                    Err(error) => {
                        form.set_error(export::error_message(error));
                    }
                }
                form.focus = form.error.is_none();
            }
        }
        if self.codes_preferences.is_none()
            && let Ok(vault) = self.state.vault()
        {
            let preferences = vault.preferences().codes();
            self.state.query = preferences.query.to_string();
            self.codes_preferences = Some(preferences);
        }
        if self.state.session_active()
            && let Some(result) = self.clipboard.poll()
        {
            self.clipboard_feedback = Some((
                match result {
                    Ok(()) => "Copied. The app-owned clipboard copy will expire automatically.",
                    Err(crate::platform::clipboard::ClipboardError::Busy) => {
                        "Clipboard is busy. Copy again when the other application releases it."
                    }
                    Err(crate::platform::clipboard::ClipboardError::Expired) => {
                        "The code changed before Windows could copy it. Copy the current code again."
                    }
                    Err(crate::platform::clipboard::ClipboardError::Changed) => {
                        "The clipboard changed before copying finished. Try Copy again; newer clipboard data is preserved."
                    }
                    Err(_) => {
                        "Windows could not update the clipboard. Copy or clear may be unavailable."
                    }
                },
                now + Duration::from_secs(5),
            ));
        }
        if self.closing {
            self.advance_close();
            egui::CentralPanel::default().show(root, |ui| {
                c::heading(ui, "Closing Authenticator", 1);
                if self.close_locked {
                    c::progress(ui, "Clearing the app-owned clipboard…", None);
                } else if self.preferences_error {
                    c::status(
                        ui,
                        "close-save-error",
                        "Display changes could not be saved. Your previous vault is unchanged.",
                        c::StatusKind::Error,
                    );
                    if c::button(ui, "Retry saving display changes", c::ButtonStyle::Primary)
                        .clicked()
                    {
                        self.preferences_changed();
                    }
                } else {
                    c::progress(ui, "Saving encrypted display preferences…", None);
                }
                c::body(ui, "Account codes and metadata are hidden.");
                if !self.close_locked
                    && c::button(
                        ui,
                        "Close without saving display changes",
                        c::ButtonStyle::Secondary,
                    )
                    .clicked()
                {
                    self.lock();
                    self.advance_close();
                }
            });
            ctx.output_mut(|output| {
                output.commands.retain_mut(|command| {
                    if let egui::OutputCommand::CopyText(value) = command {
                        value.zeroize();
                        false
                    } else {
                        true
                    }
                })
            });
            ctx.request_repaint_after(Duration::from_millis(100));
            return;
        }
        if !self.state.session_active() && !self.state.busy() {
            if self.export_form.is_some() {
                // An unexpected worker failure may have locked its repository.
                // Drop queued export secrets/metadata just as for explicit Lock.
                self.lock();
            }
            self.review = None;
            self.image = None;
        }
        self.poll_image();
        if self.state.route == Route::BackupCodes
            && self.backup_codes.screen != backup_codes::Screen::List
            && ctx.input(|i| i.key_pressed(egui::Key::Escape))
            && !self.state.busy()
            && ctx.memory(|m| m.top_modal_layer().is_none())
            && !egui::Popup::is_any_open(&ctx)
        {
            if let Some(editor) = &mut self.recovery_editor {
                editor.request_cancel();
            } else {
                self.backup_codes.back();
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape))
            && (self.review.is_some() || self.image.is_some())
            && !self.state.busy()
            && ctx.memory(|m| m.top_modal_layer().is_none())
            && !egui::Popup::is_any_open(&ctx)
        {
            if self.review.as_ref().is_some_and(|r| !r.mode.is_import()) {
                if let Some(review) = &mut self.review {
                    review.discard_requested = true;
                    review.focus = true;
                }
            } else {
                self.cancel_review();
            }
        }
        if let Some(event) = self.state.poll_capture() {
            if event == CaptureEvent::Locked {
                self.lock();
            } else if self.state.is_unlocked() {
                self.capture_failed = matches!(
                    event,
                    CaptureEvent::CaptureFailure(_) | CaptureEvent::DecodeFailure(_)
                );
                self.capture_message = Some(match event {
                    CaptureEvent::ReviewReady => "QR scanned locally. Review before saving.".into(),
                    CaptureEvent::Cancelled => "Selection cancelled. Nothing was added.".into(),
                    CaptureEvent::CaptureFailure(e) => e.to_string(),
                    CaptureEvent::DecodeFailure(e) => e.to_string(),
                    CaptureEvent::Locked => unreachable!(),
                });
                self.capture_focus = true;
                if event == CaptureEvent::ReviewReady
                    && let Some(scan) = self.state.take_scan_review()
                {
                    self.review = Some(ImportReview::from_scan(scan));
                }
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            ctx.request_repaint();
        }
        if self.state.capture_decoding() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        }
        if let Some((generation, frame_ready)) = self.capture_preparing {
            if generation != self.state.generation() || !self.state.is_unlocked() {
                self.capture_preparing = None;
            } else if !frame_ready {
                // A complete nonsensitive ROOT frame + AccessKit update must
                // precede hiding the window. No new native window exists yet.
                self.capture_preparing = Some((generation, true));
                ctx.request_repaint();
            } else {
                self.capture_preparing = None;
                let repaint = self.context.clone();
                if let Ok(lifecycle) = &self.lifecycle {
                    match self
                        .state
                        .begin_capture(self.owner, lifecycle.watch(), move || {
                            repaint.request_repaint()
                        }) {
                        Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false)),
                        Err(e) => self.capture_message = Some(e.to_string()),
                    }
                }
            }
        }
        if !self.state.busy() {
            self.exists = self.state.vault_exists().unwrap_or(true);
        }
        if ctx.input(|i| !i.events.is_empty()) {
            self.state.activity(now);
        }
        // Real monotonic expiry continues even with no pointer/keyboard events.
        if self.state.busy()
            || self.export_picker.is_some()
            || self.backup_picker.is_some()
            || self.image.is_some()
            || self.capture_preparing.is_some()
            || self.state.capture_busy()
        {
            ctx.request_repaint_after(Duration::from_millis(100));
        } else if self.state.session_active() {
            // Countdown/clock reads on UTC seconds, including the exact OTP boundary.
            let until_second = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| Duration::from_secs(1) - Duration::from_nanos(u64::from(d.subsec_nanos())))
                .unwrap_or(Duration::from_secs(1));
            ctx.request_repaint_after(until_second);
        }
        c::restore_dialog_focus(&ctx);
        if self
            .clipboard_feedback
            .is_some_and(|(_, until)| now >= until)
        {
            self.clipboard_feedback = None;
        }
        // Keep this holder in the tree even when empty: conditionally inserting
        // a Root panel changes later auto-IDs and would drop Copy's focus.
        egui::Panel::bottom("clipboard-feedback")
            .frame(egui::Frame::NONE)
            .exact_size(if self.clipboard_feedback.is_some() {
                theme::CLIPBOARD_FEEDBACK_HEIGHT
            } else {
                0.
            })
            .show(root, |ui| {
                if let Some((text, until)) = self.clipboard_feedback {
                    c::status(ui, "clipboard-result", text, c::StatusKind::Info);
                    ctx.request_repaint_after(until.saturating_duration_since(now));
                }
            });
        // This single scroll holder keeps all controls reachable at high app zoom.
        egui::CentralPanel::default().frame(egui::Frame::new().fill(theme::BACKGROUND).inner_margin(theme::PADDING)).show(root,|ui| {
            let holder = egui::ScrollArea::vertical().id_salt("screen-holder").scroll_source(egui::scroll_area::ScrollSource { drag: egui::scroll_area::DragScroll::Never, ..Default::default() });
            let holder = if self.review.as_ref().is_some_and(|r|r.focus) { holder.vertical_scroll_offset(0.) } else { holder };
            holder.show(ui,|ui| {
                    ui.set_max_width(theme::CONTENT_MAX_WIDTH.min(ui.available_width()));
                let codes_home=self.state.session_active() && self.state.route==Route::Codes && self.review.is_none() && self.backup_import.is_none() && self.capture_preparing.is_none() && !self.state.capture_busy();
                if !codes_home { c::secondary(ui,"AUTHENTICATOR · LOCAL VAULT"); }
                c::toolbar(ui,|ui| {
                    c::heading(ui,if self.backup_import.is_some() {"Import backup"} else if self.export_form.is_some() {"Export encrypted backup"} else if self.export_hub {"Export & Import"} else if self.recovery_editor.is_some() {"Backup Codes"} else if self.pending_mutation {self.state.route.title()} else if self.state.importing() {"Saving accounts"} else if self.review.as_ref().is_some_and(|r| !r.mode.is_import()) {"Account"} else if self.state.is_unlocked() && (self.capture_preparing.is_some() || self.state.capture_busy()) {"Screenshot to Scan"} else if self.state.session_active() {self.state.route.title()} else if self.exists {"Unlock vault"} else {"Create vault"},1);
                    let help=c::icon_button(ui,c::Icon::Help,"Help for this screen");
                    if help.clicked() {self.dialog=Some(c::Dialog::new("Offline help",help.id)); self.help_open=true;}
                    if self.state.session_active() && c::icon_button(ui,c::Icon::Lock,"Lock vault").clicked() {self.lock();}
                });
                if !codes_home { c::secondary(ui,if self.state.session_active() {"Vault unlocked on this device"} else {"Vault locked · credentials hidden"}); }
                ui.add_space(theme::SMALL_GAP);
                if self.recovery_editor.is_none() && !self.export_hub && self.backup_import.is_none() && self.export_form.is_none() && self.review.is_none() && self.image.is_none() && self.capture_preparing.is_none() && !self.state.capture_busy() && let Some(route)=c::tabs(ui,self.state.route) { if route != self.state.route { self.backup_codes.reset(); self.focus_form=true; } self.state.navigate(route); self.status=None; }
                ui.add_space(theme::SMALL_GAP);
                if let Some(error)=self.state.storage_error() {
                    c::status(ui,"storage-error",&error.to_string(),c::StatusKind::Error);
                    c::secondary(ui,"Close other Authenticator instances and restart. No replacement vault is created here.");
                } else if self.backup_import.is_some() && self.state.session_active() { self.backup_import_content(ui);
                } else if self.export_form.is_some() && self.state.session_active() { self.export_content(ui);
                } else if self.pending_mutation { self.unlocked_content(ui);
                } else if self.recovery_editor.is_some() && self.state.session_active() { self.backup_codes_content(ui);
                } else if self.state.importing() { self.review_content(ui);
                } else if !self.state.is_unlocked() {
                    self.locked_content(ui);
                } else { self.unlocked_content(ui); }
                if let Some(status)=self.status {c::status(ui,"shell-feedback",status,c::StatusKind::Info);}
            });
        });
        self.show_dialog(&ctx);
        self.delete_dialog(&ctx);
        self.persist_codes_preferences();
        // Guard framework Copy/Cut/selection routes too; never let eframe's
        // unowned clipboard adapter publish text behind the lease actor.
        let mut copies = Vec::new();
        ctx.output_mut(|output| {
            output.commands.retain_mut(|command| {
                if let egui::OutputCommand::CopyText(value) = command {
                    copies.push(Zeroizing::new(std::mem::take(value)));
                    false
                } else {
                    true
                }
            })
        });
        for value in copies {
            self.copy_text(value);
        }
    }
    fn locked_content(&mut self, ui: &mut egui::Ui) {
        if !self.lifecycle.as_ref().is_ok_and(|l| l.healthy()) {
            c::status(
                ui,
                "lifecycle-error",
                "Windows lock notifications are unavailable. Close and reopen Authenticator to retry. Your vault stays locked.",
                c::StatusKind::Error,
            );
            return;
        }
        if self.state.busy() {
            c::progress(
                ui,
                if self.operation_cancelled {
                    "Finishing cancellation · vault stays locked"
                } else if self.exists {
                    "Unlocking vault…"
                } else {
                    "Creating encrypted vault…"
                },
                None,
            );
            c::secondary(
                ui,
                "Passphrase work stays on this device. You can cancel or close this window.",
            );
            if !self.operation_cancelled {
                let cancel = c::button(ui, "Cancel", c::ButtonStyle::Secondary);
                if self.focus_form {
                    cancel.request_focus();
                    self.focus_form = false;
                }
                if cancel.clicked() {
                    self.lock();
                }
            }
            return;
        }
        c::card(ui, |ui| {
            c::heading(
                ui,
                if self.exists {
                    "Your local vault"
                } else {
                    "Protect your local vault"
                },
                2,
            );
            c::body(
                ui,
                if self.exists {
                    "Unlock to see account names, verification codes and recovery codes."
                } else {
                    "Encrypt this device’s vault with a passphrase. No account or internet connection needed."
                },
            );
            ui.add_space(theme::SMALL_GAP);
            let field = c::labeled_secret_field(
                ui,
                "vault-passphrase",
                "Vault passphrase",
                &mut self.passphrase,
                "Use a long, unique passphrase. Spaces are kept exactly.",
                None,
            );
            if self.focus_form && ui.ctx().memory(|m| m.top_modal_layer().is_none()) {
                field.request_focus();
                self.focus_form = false;
            }
            let mut submit = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if !self.exists {
                let confirm = c::labeled_secret_field(
                    ui,
                    "vault-confirmation",
                    "Confirm passphrase",
                    &mut self.confirmation,
                    "Enter exactly the same passphrase.",
                    None,
                );
                submit |= confirm.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            }
            c::secondary(
                ui,
                "A lost passphrase cannot be reset. Recovery needs a separate encrypted backup and its passphrase. Keep it independently; creating a vault does not create a backup.",
            );
            if let Some(error) = &self.form_error {
                c::status(ui, "vault-form-error", error, c::StatusKind::Error);
            }
            let action = c::button(
                ui,
                if self.exists {
                    "Unlock vault"
                } else {
                    "Create vault"
                },
                c::ButtonStyle::Primary,
            );
            if action.clicked() || submit {
                self.submit();
            }
            if c::button(ui, "Clear fields", c::ButtonStyle::Secondary).clicked() {
                self.clear_form();
                self.form_error = None;
                self.focus_form = true;
            }
        });
    }
    fn submit(&mut self) {
        if self.state.busy() {
            return;
        }
        if self.passphrase.len() < 12 {
            self.form_error =
                Some("Choose a longer passphrase. Try several unrelated words.".into());
            return;
        }
        if self.passphrase.len() > 1024 {
            self.form_error = Some("This passphrase is too long. Shorten it and try again.".into());
            return;
        }
        if !self.exists && *self.passphrase != *self.confirmation {
            self.form_error = Some("The passphrases do not match. Enter them again.".into());
            return;
        }
        let passphrase = std::mem::replace(&mut self.passphrase, Zeroizing::new(String::new()));
        self.clear_form();
        self.form_error = None;
        self.operation_cancelled = false;
        self.focus_form = true;
        let repaint = self.context.clone();
        if let Err(error) = self.state.begin(
            if self.exists {
                VaultAction::Unlock
            } else {
                VaultAction::Create
            },
            passphrase,
            move || repaint.request_repaint(),
        ) {
            self.form_error = Some(error.to_string());
        }
    }
    fn unlocked_content(&mut self, ui: &mut egui::Ui) {
        if self.export_hub {
            self.export_hub_content(ui);
            return;
        }
        if self.pending_mutation && self.state.route == Route::Settings {
            c::progress(ui, "Saving encrypted settings…", None);
            return;
        }
        if self.review.is_some() {
            self.review_content(ui);
            return;
        }
        if self.image.is_some() {
            c::progress(ui, "Choose or scan a local QR image…", None);
            if c::button(ui, "Cancel image import", c::ButtonStyle::Secondary).clicked() {
                self.cancel_review();
            }
            return;
        }
        if self.capture_preparing.is_some() || self.state.capture_busy() {
            self.capture_controls(ui);
            return;
        }
        match self.state.route {
            Route::BackupCodes => self.backup_codes_content(ui),
            Route::Codes => {
                self.capture_controls(ui);
                if self.capture_preparing.is_some() {
                    return;
                }
                let search_error = (self.state.query.len() > CodesPreferences::MAX_QUERY_BYTES
                    || self.state.query.chars().any(char::is_control))
                .then_some("Use at most 1024 UTF-8 bytes without control characters.");
                let search =
                    c::labeled_search_field(ui, "search", &mut self.state.query, search_error);
                if self.focus_form {
                    search.request_focus();
                    self.focus_form = false;
                }
                if search.changed()
                    && self.state.query.len() <= CodesPreferences::MAX_QUERY_BYTES
                    && !self.state.query.chars().any(char::is_control)
                {
                    if let Some(preferences) = &mut self.codes_preferences {
                        preferences.query = Zeroizing::new(self.state.query.clone());
                    }
                    self.preferences_changed();
                }
                self.codes_toolbar(ui);
                if self.state.query.len() > CodesPreferences::MAX_QUERY_BYTES
                    || self.state.query.chars().any(char::is_control)
                {
                    return;
                }
                if self.pending_mutation {
                    c::progress(ui, "Saving encrypted changes…", None);
                    return;
                }
                let count = self
                    .state
                    .vault()
                    .map(|v| {
                        v.accounts()
                            .iter()
                            .filter(|a| a.state().deletion == DeletionState::Active)
                            .count()
                    })
                    .unwrap_or(0);
                if count == 0 {
                    c::empty_state(
                        ui,
                        c::Icon::Key,
                        "No accounts yet",
                        "Scan a QR code from another window, review the account, then save it securely.",
                    );
                } else {
                    self.code_cards(ui);
                }
            }
            Route::Settings => {
                let export = c::button(ui, "Export & Import", c::ButtonStyle::Primary);
                if self.export_focus {
                    export.request_focus();
                    self.export_focus = false;
                }
                if export.clicked() {
                    self.export_hub = true;
                    self.export_focus = true;
                    self.status = None;
                }
                c::card(ui, |ui| {
                    c::icon_tile(ui, c::Icon::Settings);
                    c::heading(ui, "Local vault", 2);
                    c::body(
                        ui,
                        "Lock after inactivity. Windows session lock, sleep and closing the app also require unlocking again.",
                    );
                    let current = self
                        .state
                        .vault()
                        .map(|v| v.preferences().lock_timeout_seconds())
                        .unwrap_or(300);
                    for (label, seconds) in [
                        ("1 minute", 60),
                        ("5 minutes (default)", 300),
                        ("15 minutes", 900),
                        ("30 minutes", 1800),
                    ] {
                        let label = if current == seconds {
                            format!("Selected · {label}")
                        } else {
                            label.into()
                        };
                        if c::button(ui, &label, c::ButtonStyle::Secondary).clicked() {
                            self.begin_mutation(SaveIntent::IdleTimeout(seconds));
                        }
                    }
                    c::heading(ui, "Clipboard expiry", 2);
                    c::body(
                        ui,
                        "Clear only the copy still owned by Authenticator. Windows or other clipboard tools may retain copies.",
                    );
                    let current = self
                        .state
                        .vault()
                        .map(|v| v.preferences().clipboard_timeout_seconds())
                        .unwrap_or(30);
                    for seconds in [15, 30, 60, 120] {
                        let label = format!(
                            "{}{} seconds",
                            if current == seconds {
                                "Selected · "
                            } else {
                                ""
                            },
                            seconds
                        );
                        if c::button(ui, &label, c::ButtonStyle::Secondary).clicked() {
                            self.begin_mutation(SaveIntent::ClipboardTimeout(seconds));
                        }
                    }
                });
                if c::button(ui, "Lock vault", c::ButtonStyle::Primary).clicked() {
                    self.lock();
                }
            }
        }
    }
    fn backup_codes_content(&mut self, ui: &mut egui::Ui) {
        if let Some(mut editor) = self.recovery_editor.take() {
            let had_job = editor.job.is_some();
            let action = if editor.saving {
                c::progress(ui, "Saving encrypted backup-code changes…", None);
                if c::button(ui, "Cancel and lock", c::ButtonStyle::Secondary).clicked() {
                    recovery_editor::Action::Lock
                } else {
                    recovery_editor::Action::None
                }
            } else if let Ok(vault) = self.state.vault() {
                editor.content(ui, vault)
            } else {
                recovery_editor::Action::Lock
            };
            #[cfg(debug_assertions)]
            if !editor.saving
                && let Some(rect) = editor.sensitive_rect
            {
                self.evidence_code_bounds.push(rect);
            }
            if had_job && editor.job.is_none() {
                self.state.finish_image();
            }
            let mut keep = true;
            match action {
                recovery_editor::Action::None => {}
                recovery_editor::Action::Lock => {
                    self.lock();
                    keep = false;
                }
                recovery_editor::Action::Cancel => {
                    if editor.job.is_some() {
                        self.state.finish_image();
                    }
                    keep = false;
                    if matches!(self.backup_codes.screen, backup_codes::Screen::Editor(_)) {
                        self.backup_codes.back();
                    }
                    self.backup_codes.focus = true;
                    c::clear_sensitive_widget_memory(&self.context);
                }
                recovery_editor::Action::Preview | recovery_editor::Action::ReadFile => {
                    if let (Ok(()), Some(watch), Ok(deadline)) = (
                        self.state.begin_image(),
                        self.lifecycle.as_ref().ok().map(|l| l.watch()),
                        self.state.idle_deadline(),
                    ) {
                        editor.job = Some(recovery_editor::Job::start(
                            self.state.generation(),
                            watch,
                            deadline,
                            self.context.clone(),
                            editor.target.clone(),
                            if matches!(action, recovery_editor::Action::Preview) {
                                Some(editor.text.clone())
                            } else {
                                None
                            },
                        ));
                    } else {
                        self.state.finish_image();
                        editor.error = Some(
                            "Finish the active operation before reading or previewing codes."
                                .into(),
                        );
                    }
                }
                recovery_editor::Action::Save(intent) => {
                    if let Some(watch) = self.lifecycle.as_ref().ok().map(|l| l.watch()) {
                        let repaint = self.context.clone();
                        match self
                            .state
                            .begin_save(*intent, watch, move || repaint.request_repaint())
                        {
                            Ok(()) => {
                                editor.saving = true;
                                editor.reveal = false;
                                editor.error = None;
                                c::clear_sensitive_widget_memory(&self.context);
                            }
                            Err(error) => {
                                editor.deleting = false;
                                editor.error = Some(error.to_string());
                            }
                        }
                    } else {
                        editor.error = Some(
                            "Windows lifecycle protection is unavailable. Restart before saving."
                                .into(),
                        );
                    }
                }
            }
            if keep {
                self.recovery_editor = Some(editor);
            }
            return;
        }
        if self.pending_mutation {
            c::progress(ui, "Saving encrypted changes…", None);
            return;
        }
        if self.focus_form {
            self.backup_codes.focus = true;
            self.focus_form = false;
        }
        let mut query = std::mem::take(&mut self.state.backup_query);
        let action = match self.state.vault() {
            Ok(vault) => backup_codes::content(ui, vault, &mut self.backup_codes, &mut query),
            Err(_) => {
                c::status(
                    ui,
                    "backup-vault-unavailable",
                    "Unlock the vault to view backup-code sets.",
                    c::StatusKind::Info,
                );
                backup_codes::Action::None
            }
        };
        self.state.backup_query = query;
        match action {
            backup_codes::Action::Help(opener) => {
                self.dialog = Some(c::Dialog::new("About backup codes", opener));
                self.help_open = true;
            }
            backup_codes::Action::AddAccount => {
                self.backup_codes.reset();
                self.state.navigate(Route::Codes);
                self.start_alternative(ScanAlternative::Manual);
            }
            backup_codes::Action::None => {}
            backup_codes::Action::Edit(id) | backup_codes::Action::Delete(id) => {
                match self.state.vault().and_then(|v| RecoveryEditor::new(v, id)) {
                    Ok(mut editor) => {
                        if let backup_codes::Action::Delete(_) = action {
                            editor.request_delete_on_open();
                        }
                        self.recovery_editor = Some(editor);
                        self.status = None;
                        c::clear_sensitive_widget_memory(&self.context);
                    }
                    Err(_) => {
                        self.status =
                            Some("This account changed. Reopen it before editing backup codes.")
                    }
                }
            }
        }
    }
    fn export_hub_content(&mut self, ui: &mut egui::Ui) {
        if c::button(ui, "Back to Settings", c::ButtonStyle::Secondary).clicked() {
            self.export_hub = false;
            self.export_focus = true;
            return;
        }
        c::card(ui, |ui| {
            c::icon_tile(ui, c::Icon::Lock);
            c::heading(ui, "A backup you control", 2);
            c::body(
                ui,
                "Save a portable encrypted copy of your full vault to a local file. Keep a separate backup passphrase to restore it later.",
            );
            c::secondary(
                ui,
                "No registration or internet connection is needed. Store the file and its passphrase separately.",
            );
            let export = c::button(ui, "Export encrypted backup", c::ButtonStyle::Primary);
            if self.export_focus {
                export.request_focus();
                self.export_focus = false;
            }
            if export.clicked() {
                self.export_form = Some(ExportForm::new(export.id));
                c::clear_sensitive_widget_memory(&self.context);
            }
            let import = c::button(ui, "Import backup", c::ButtonStyle::Secondary);
            if import.clicked() {
                self.backup_import = Some(ImportForm::new(import.id));
                c::clear_sensitive_widget_memory(&self.context);
            }
        });
    }
    fn start_export(&mut self, form: &mut ExportForm) {
        if form.exporting {
            return;
        }
        if self.state.busy() {
            if self.pending_preferences {
                form.saving_preferences = true;
                form.focus = true;
            } else {
                form.set_error("Finish the active operation before exporting the backup.");
            }
            return;
        }
        if let Err(error) = form.validate() {
            form.set_error(error);
            return;
        }
        if self.preferences_dirty.is_some() {
            let Some(preferences) = self.codes_preferences.clone() else {
                form.clear_secrets(&self.context);
                form.set_error("Current display settings are unavailable. Return to Settings and reopen export before retrying.");
                return;
            };
            self.preferences_error = false;
            if self.begin_mutation(SaveIntent::CodesPreferences(preferences)) {
                self.preferences_dirty = None;
                form.saving_preferences = true;
                form.error = None;
                form.focus = true;
            } else {
                form.clear_secrets(&self.context);
                form.set_error("Display settings could not be saved. Finish other operations or restart Authenticator, then retry export.");
            }
            return;
        }
        let Some(destination) = form.destination.take() else {
            return;
        };
        let passphrase = std::mem::take(&mut form.passphrase);
        form.clear_secrets(&self.context);
        let repaint = self.context.clone();
        let result = match &self.lifecycle {
            Ok(lifecycle) if lifecycle.healthy() => self.state.begin_export(
                destination.approve(),
                passphrase,
                lifecycle.watch(),
                move || repaint.request_repaint(),
            ),
            _ => Err(crate::error::AppError::StorageUnavailable),
        };
        match result {
            Ok(()) => {
                form.exporting = true;
                form.focus = true;
                form.error = None;
            }
            Err(_) => {
                form.set_error("Cannot start export. Wait for other operations, or restart Authenticator if lifecycle protection is unavailable. Choose a destination and retry.");
            }
        }
    }
    fn export_content(&mut self, ui: &mut egui::Ui) {
        let Some(mut form) = self.export_form.take() else {
            return;
        };
        if form.saving_preferences && !self.state.busy() {
            form.saving_preferences = false;
            if self.preferences_error {
                form.clear_secrets(&self.context);
                form.set_error("Your display changes could not be saved, so export did not start. Check access to the local vault, enter the backup passphrase and retry. Export did not write to the selected file.");
            } else {
                self.start_export(&mut form);
            }
        }
        let action = export::form(ui, &mut form, self.export_picker.is_some());
        let mut keep = true;
        match action {
            export::Action::None => {}
            export::Action::Back => {
                form.clear_secrets(&self.context);
                self.export_picker = None;
                self.state.finish_image();
                self.export_focus = true;
                self.context
                    .memory_mut(|memory| memory.request_focus(form.opener));
                keep = false;
            }
            export::Action::CancelAndLock => {
                self.lock();
                keep = false;
            }
            export::Action::Pick => {
                let launch = self.state.idle_deadline().and_then(|deadline| {
                    let watch = self.lifecycle.as_ref().map_err(|e| *e)?.watch();
                    self.state.begin_image()?;
                    self.export_picker = Some(Picker::start(
                        self.state.generation(),
                        watch,
                        deadline,
                        self.context.clone(),
                    ));
                    Ok(())
                });
                if launch.is_err() {
                    form.set_error("Cannot open the destination picker. Finish other operations or restart Authenticator, then retry.");
                }
            }
            export::Action::Submit(opener) => match form.validate() {
                Err(error) => form.set_error(error),
                Ok(())
                    if form
                        .destination
                        .as_ref()
                        .is_some_and(|d| d.needs_overwrite()) =>
                {
                    form.overwrite = Some(c::Dialog::new("Replace existing backup file?", opener));
                }
                Ok(()) => self.start_export(&mut form),
            },
        }
        if let Some(mut dialog) = form.overwrite.take() {
            let mut consent = false;
            let close = dialog.show(&self.context, |ui, first| {
                c::body(ui, "Export will replace this existing file with the encrypted vault backup:");
                if let Some(path) = &form.path { c::body(ui, &path.to_string_lossy()); }
                c::secondary(ui, "Keep the existing file if it is your only working backup. A changed file must be selected and reviewed again.");
                let cancel = c::button(ui, "Keep existing file", c::ButtonStyle::Secondary);
                if first { cancel.request_focus(); }
                if c::button(ui, "Replace file and export", c::ButtonStyle::Primary).clicked() { consent = true; }
                cancel.clicked() || consent
            });
            if consent {
                self.start_export(&mut form);
            } else if !close {
                form.overwrite = Some(dialog);
            }
        }
        if keep {
            self.export_form = Some(form);
        }
    }
    fn complete_backup_import(&mut self, result: super::state::BackupResult) {
        use super::state::BackupResult;
        let Some(form) = &mut self.backup_import else {
            return;
        };
        form.busy = None;
        form.focus = true;
        form.clear_secret(&self.context);
        match result {
            BackupResult::Preview(result) => match result {
                Ok(preview) => {
                    form.preview = Some(preview);
                    form.error = None;
                }
                Err(error) => {
                    form.reset_selection();
                    form.error = Some(backup_import::error_message(error));
                }
            },
            BackupResult::Plan(result) => match result {
                Ok(plan) => {
                    form.plan = Some(*plan);
                    form.error = None;
                }
                Err(error) => form.error = Some(backup_import::error_message(error)),
            },
            BackupResult::Applied(result) => match result {
                Ok(_) => {
                    // Only an adopted, current approved success invalidates the
                    // projection. Failures keep the old session data intact.
                    self.backup_import = None;
                    self.backup_picker = None;
                    self.export_form = None;
                    self.export_picker = None;
                    self.export_hub = false;
                    self.codes_preferences = None;
                    self.preferences_dirty = None;
                    self.preferences_error = false;
                    self.ordered_ids = None;
                    self.state.query.zeroize();
                    self.state.backup_query.zeroize();
                    self.backup_codes.reset();
                    self.recovery_editor = None;
                    self.imported.clear();
                    self.show_imported = false;
                    self.review = None;
                    self.clock.clear();
                    self.clipboard.clear();
                    self.clipboard_feedback = None;
                    self.state.navigate(Route::Codes);
                    self.capture_message = Some("Backup import saved in your encrypted vault. The current vault data is shown below.".into());
                    self.capture_failed = false;
                    self.capture_focus = true;
                    c::clear_sensitive_widget_memory(&self.context);
                }
                Err(error) => {
                    form.reset_selection();
                    form.error = Some(backup_import::error_message(error));
                }
            },
        }
    }
    fn start_backup_preview(&mut self, form: &mut ImportForm) {
        if form.busy.is_some() {
            return;
        }
        if let Err(error) = form.validate() {
            form.error = Some(error);
            return;
        }
        if self.state.busy() {
            if self.pending_preferences {
                form.saving_preferences = true;
                form.focus = true;
            } else {
                form.error = Some("Finish the active operation before previewing this backup.");
            }
            return;
        }
        // Save the latest projection before the engine captures its exact
        // committed destination revision. Cancellation later cannot roll back
        // this independently requested preferences transaction.
        if self.preferences_dirty.is_some() {
            let Some(preferences) = self.codes_preferences.clone() else {
                form.clear_secret(&self.context);
                form.error = Some(
                    "Current display settings are unavailable. Return to Settings and reopen import.",
                );
                return;
            };
            self.preferences_error = false;
            if self.begin_mutation(SaveIntent::CodesPreferences(preferences)) {
                self.preferences_dirty = None;
                form.saving_preferences = true;
                form.focus = true;
                form.error = None;
            } else {
                form.clear_secret(&self.context);
                form.error = Some(
                    "Display settings could not be saved. Check local vault access, enter the backup passphrase and retry.",
                );
            }
            return;
        }
        let Some(path) = form.path.clone() else {
            return;
        };
        let passphrase = std::mem::take(&mut form.passphrase);
        form.clear_secret(&self.context);
        self.start_backup_job(
            form,
            super::state::BackupRequest::Preview(path, passphrase),
            ImportBusy::Preview,
        );
    }
    fn start_backup_job(
        &mut self,
        form: &mut ImportForm,
        request: super::state::BackupRequest,
        busy: ImportBusy,
    ) {
        let repaint = self.context.clone();
        let result = match &self.lifecycle {
            Ok(lifecycle) if lifecycle.healthy() => {
                self.state
                    .begin_backup(request, lifecycle.watch(), move || {
                        repaint.request_repaint()
                    })
            }
            _ => Err(crate::error::AppError::StorageUnavailable),
        };
        match result {
            Ok(()) => {
                form.busy = Some(busy);
                form.focus = true;
                form.error = None;
            }
            Err(_) => {
                form.error = Some(
                    "Cannot start import. Finish other operations or restart Authenticator if lifecycle protection is unavailable, then preview again.",
                );
                form.focus = true;
            }
        }
    }
    fn apply_backup_plan(&mut self, form: &mut ImportForm) {
        let Some(plan) = form.plan.take() else {
            return;
        };
        // Final consent has been obtained. UI no longer retains incoming/local
        // snapshot metadata while a worker owns the approved candidate.
        form.preview = None;
        form.replace = None;
        form.options = crate::backup::ImportOptions::default();
        form.clear_secret(&self.context);
        self.start_backup_job(
            form,
            super::state::BackupRequest::Apply(Box::new(plan)),
            ImportBusy::Apply,
        );
    }
    fn backup_import_content(&mut self, ui: &mut egui::Ui) {
        let Some(mut form) = self.backup_import.take() else {
            return;
        };
        if form.saving_preferences && !self.state.busy() {
            form.saving_preferences = false;
            if self.preferences_error {
                form.clear_secret(&self.context);
                form.error = Some(
                    "Your display changes could not be saved, so preview did not start. Check local vault access, enter the backup passphrase and retry. No backup data was imported.",
                );
            } else {
                self.start_backup_preview(&mut form);
            }
        }
        let mut keep = true;
        match backup_import::form(ui, &mut form, self.backup_picker.is_some()) {
            backup_import::Action::None => {}
            backup_import::Action::Back => {
                if form.plan.take().is_some() {
                    form.focus = true;
                    form.error = None;
                } else if form.preview.is_some() {
                    form.reset_selection();
                    form.clear_secret(&self.context);
                } else {
                    form.clear_secret(&self.context);
                    self.export_focus = false;
                    self.context.memory_mut(|m| m.request_focus(form.opener));
                    keep = false;
                }
            }
            backup_import::Action::CancelAndLock => {
                self.lock();
                keep = false;
            }
            backup_import::Action::Pick => {
                form.clear_secret(&self.context);
                let launch = self.state.idle_deadline().and_then(|deadline| {
                    let watch = self.lifecycle.as_ref().map_err(|e| *e)?.watch();
                    self.state.begin_image()?;
                    self.backup_picker = Some(backup_import::Picker::start(
                        self.state.generation(),
                        watch,
                        deadline,
                        self.context.clone(),
                    ));
                    Ok(())
                });
                if launch.is_err() {
                    form.error = Some(
                        "Cannot open the backup picker. Finish other operations or restart Authenticator, then retry.",
                    );
                }
            }
            backup_import::Action::Preview => self.start_backup_preview(&mut form),
            backup_import::Action::Plan => {
                if let Some(preview) = form.preview.clone() {
                    let options = form.plan_options();
                    self.start_backup_job(
                        &mut form,
                        super::state::BackupRequest::Plan(preview, options),
                        ImportBusy::Plan,
                    );
                }
            }
            backup_import::Action::Apply(opener) => {
                if form
                    .plan
                    .as_ref()
                    .is_some_and(|p| p.summary().mode == crate::backup::ImportMode::Replace)
                {
                    form.replace =
                        Some(c::Dialog::new("Replace current vault collections?", opener));
                } else {
                    self.apply_backup_plan(&mut form);
                }
            }
        }
        if let Some(mut dialog) = form.replace.take() {
            let mut consent = false;
            let close = dialog.show(&self.context, |ui, first| {
                if let Some(plan) = &form.plan {
                    backup_import::counts(ui, "Current records that will be removed", plan.summary().removed_local_records);
                    backup_import::counts(ui, "Resulting vault", plan.summary().after);
                }
                c::body(ui, "This replaces accounts, folders, memberships and recovery records, including Trash and used states. Keep a current encrypted backup if you may need these records again.");
                let cancel = c::button(ui, "Keep current vault · go back", c::ButtonStyle::Secondary);
                if first { cancel.request_focus(); }
                let confirm = c::button(ui, "Replace collections and import", c::ButtonStyle::Primary);
                if confirm.clicked() { consent = true; }
                cancel.clicked() || consent
            });
            if consent {
                self.apply_backup_plan(&mut form);
            } else if !close {
                form.replace = Some(dialog);
            }
        }
        if keep {
            self.backup_import = Some(form);
        }
    }
    fn capture_controls(&mut self, ui: &mut egui::Ui) {
        if self.capture_preparing.is_some() || self.state.capture_busy() {
            c::progress(
                ui,
                if self.capture_preparing.is_some() {
                    "Preparing private screen selection…"
                } else {
                    "Selecting or scanning QR locally…"
                },
                None,
            );
            if c::button(ui, "Cancel scan", c::ButtonStyle::Secondary).clicked() {
                self.capture_preparing = None;
                self.state.cancel_capture();
                self.capture_message = Some("Selection cancelled. Nothing was added.".into());
            }
            return;
        }
        let mut manual = false;
        let action_height = if ui.available_width() < theme::CODES_TOOLBAR_WIDE {
            theme::WRAPPED_TAB_HEIGHT
        } else {
            theme::CONTROL_HEIGHT
        };
        let start = ui.columns(2, |columns| {
            let start = c::button_with_height(
                &mut columns[0],
                if self.capture_failed {
                    "Retry Screenshot to Scan"
                } else {
                    "Screenshot to Scan"
                },
                c::ButtonStyle::Primary,
                action_height,
            );
            manual = c::button_with_height(
                &mut columns[1],
                "Add Account",
                c::ButtonStyle::Primary,
                action_height,
            )
            .clicked();
            start
        });
        if manual {
            self.start_alternative(ScanAlternative::Manual);
        }
        if self.capture_focus {
            start.request_focus();
            self.capture_focus = false;
        }
        c::secondary(
            ui,
            "Scan a QR in another window, or add an account manually.",
        );
        if start.clicked() {
            self.capture_message = None;
            self.capture_failed = false;
            self.dialog = None;
            self.state.cancel_capture();
            c::clear_sensitive_widget_memory(&self.context);
            self.capture_preparing = Some((self.state.generation(), false));
            ui.ctx().request_repaint();
        }
        if let Some(message) = &self.capture_message {
            c::status(
                ui,
                "capture-feedback",
                message,
                if self.capture_failed {
                    c::StatusKind::Error
                } else {
                    c::StatusKind::Info
                },
            );
        }
        if c::button(ui, "Import from Photos", c::ButtonStyle::Secondary).clicked() {
            self.start_alternative(ScanAlternative::Image);
        }
        ui.add_space(theme::SMALL_GAP);
    }
    fn cancel_review(&mut self) {
        let edit = self.review.as_ref().is_some_and(|r| r.mode.is_edit());
        let manual = self.review.as_ref().is_some_and(|r| !r.mode.is_import());
        self.review = None;
        self.image = None;
        self.state.finish_image();
        self.state.cancel_capture();
        self.capture_message = Some(
            if edit {
                "Edit cancelled. The original account is unchanged."
            } else if manual {
                "Account creation cancelled. Nothing was added."
            } else {
                "Import cancelled. Nothing was added."
            }
            .into(),
        );
        self.capture_failed = false;
        self.capture_focus = true;
        c::clear_sensitive_widget_memory(&self.context);
    }
    fn start_alternative(&mut self, alternative: ScanAlternative) {
        if !self.state.is_unlocked()
            || self.state.busy()
            || self.state.capture_busy()
            || self.image.is_some()
        {
            return;
        }
        self.review = None;
        self.dialog = None;
        self.capture_failed = false;
        self.capture_message = None;
        c::clear_sensitive_widget_memory(&self.context);
        if alternative == ScanAlternative::Manual {
            self.review = Some(ImportReview::manual());
            return;
        }
        let Ok(lifecycle) = &self.lifecycle else {
            return;
        };
        let watch = lifecycle.watch();
        if self.state.begin_image().is_err() {
            return;
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        let result = Arc::new(Mutex::new(None));
        self.image = Some(ImageJob {
            generation: self.state.generation(),
            cancelled: cancelled.clone(),
            result: result.clone(),
        });
        let repaint = self.context.clone();
        std::thread::spawn(move || {
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let path = crate::platform::image_picker::choose(&|| {
                    cancelled.load(Ordering::SeqCst) || watch.changed()
                })?;
                if cancelled.load(Ordering::SeqCst) || watch.changed() {
                    return Ok(None);
                }
                path.map(|p| crate::qr::decode_file(&p)).transpose()
            }))
            .unwrap_or(Err(crate::qr::QrError::DecodeFailure));
            let mut slot = result.lock().unwrap_or_else(|e| e.into_inner());
            if !cancelled.load(Ordering::SeqCst) && !watch.changed() {
                *slot = Some(outcome);
            }
            drop(slot);
            repaint.request_repaint();
        });
    }
    fn poll_image(&mut self) {
        let Some(job) = &self.image else {
            return;
        };
        if job.generation != self.state.generation() || !self.state.is_unlocked() {
            self.image = None;
            return;
        }
        let outcome = job.result.lock().unwrap_or_else(|e| e.into_inner()).take();
        if let Some(outcome) = outcome {
            self.image = None;
            self.state.finish_image();
            match outcome {
                Ok(Some(scan)) => self.review = Some(ImportReview::from_scan(scan)),
                Ok(None) => {
                    self.capture_message =
                        Some("Image selection cancelled. Nothing was added.".into());
                    self.capture_failed = false;
                    self.capture_focus = true;
                }
                Err(error) => {
                    self.capture_message = Some(error.to_string());
                    self.capture_failed = true;
                    self.capture_focus = true;
                }
            }
        }
    }
    fn review_content(&mut self, ui: &mut egui::Ui) {
        if self.state.importing() {
            let mode = self.review.as_ref().map(|r| r.mode);
            c::heading(
                ui,
                if mode.is_some_and(|m| m.is_edit()) {
                    "Saving account changes"
                } else {
                    "Saving reviewed accounts"
                },
                2,
            );
            c::progress(ui, "Encrypting and saving the complete operation…", None);
            c::secondary(
                ui,
                "Lock or Escape cancels pending work. If the atomic save has already completed, the whole batch remains encrypted; unlock to check it.",
            );
            if c::button(ui, "Cancel and lock", c::ButtonStyle::Secondary).clicked() {
                self.lock();
            }
            return;
        }
        let Some(review) = &mut self.review else {
            return;
        };
        if review.discard_requested {
            c::heading(ui, "Discard unsaved changes?", 2);
            c::body(
                ui,
                "Nothing is saved until you choose Save. Discard clears this form and keeps all existing records unchanged.",
            );
            let keep = c::button(ui, "Keep editing", c::ButtonStyle::Primary);
            if review.focus {
                keep.request_focus();
                review.focus = false;
            }
            if keep.clicked() {
                review.discard_requested = false;
                review.focus = true;
            }
            if c::button(ui, "Discard changes", c::ButtonStyle::Secondary).clicked() {
                self.cancel_review();
            }
            return;
        }
        if !review.mode.is_import()
            && c::button(ui, "Back to accounts", c::ButtonStyle::Secondary).clicked()
        {
            review.discard_requested = true;
            review.focus = true;
            return;
        }
        c::heading(
            ui,
            if review.choosing {
                "Choose accounts to import"
            } else if review.mode.is_edit() {
                "Edit account"
            } else if !review.mode.is_import() {
                "New account"
            } else {
                "Review account"
            },
            2,
        );
        if review.rejected > 0 && !review.editors.is_empty() {
            c::status(
                ui,
                "unsupported-candidates",
                &format!(
                    "{} QR result(s) are unsupported or invalid and cannot be imported. Retry a clearer capture, image or manual entry if an account is missing.",
                    review.rejected
                ),
                c::StatusKind::Info,
            );
        }
        if review.editors.is_empty() {
            c::status(
                ui,
                "no-supported",
                "No supported TOTP credentials were found. Use Screenshot to Scan, a PNG/JPEG image or manual entry.",
                c::StatusKind::Error,
            );
            let back = c::button(ui, "Back to import options", c::ButtonStyle::Primary);
            if review.focus {
                back.request_focus();
                review.focus = false;
            }
            if back.clicked() {
                self.cancel_review();
                self.capture_failed = true;
                self.capture_message = Some("No supported account was imported. Retry Screenshot to Scan, choose an image or use manual entry.".into());
            }
            return;
        }
        if review.choosing {
            c::secondary(
                ui,
                "Select each account you want. You can edit every selected account before the whole batch is saved.",
            );
            for (index, editor) in review.editors.iter_mut().enumerate() {
                let label = Zeroizing::new(format!(
                    "{} · {} · {}",
                    if editor.selected {
                        "Selected"
                    } else {
                        "Select"
                    },
                    *editor.service,
                    if editor.account.is_empty() {
                        "No account identifier"
                    } else {
                        &editor.account
                    }
                ));
                let button = c::button(ui, &label, c::ButtonStyle::Secondary);
                if review.focus && index == 0 {
                    button.request_focus();
                    review.focus = false;
                }
                if button.clicked() {
                    editor.selected = !editor.selected;
                }
            }
            let selected = review.editors.iter().position(|e| e.selected);
            ui.add_enabled_ui(selected.is_some(), |ui| {
                if c::button(ui, "Review selected accounts", c::ButtonStyle::Primary).clicked()
                    && let Some(index) = selected
                {
                    review.current = index;
                    review.choosing = false;
                    review.focus = true;
                    c::clear_sensitive_widget_memory(&self.context);
                }
            });
        } else {
            let selected_count = review.editors.iter().filter(|e| e.selected).count();
            let batch_description = format!(
                "{selected_count} account(s) selected. Save always creates separate new entries; existing accounts are never replaced."
            );
            c::secondary(
                ui,
                if review.mode.is_edit() {
                    "Save changes updates only this selected account. Its folders and existing recovery codes are kept."
                } else if !review.mode.is_import() {
                    "Save creates a new account. Existing accounts are kept."
                } else {
                    &batch_description
                },
            );
            if review.editors.len() > 1 {
                for (index, editor) in review
                    .editors
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.selected)
                {
                    if c::button(
                        ui,
                        &Zeroizing::new(format!("Edit {} · {}", index + 1, *editor.service)),
                        c::ButtonStyle::Secondary,
                    )
                    .clicked()
                    {
                        review.current = index;
                        review.focus = true;
                    }
                }
            }
            let editor = &mut review.editors[review.current];
            ui.push_id(review.current, |ui| {
                c::card(ui, |ui| {
                    if editor.show(
                        ui,
                        &mut review.focus,
                        review.mode.is_edit(),
                        review.error.is_some(),
                    ) {
                        review.error = None;
                    }
                });
            });
            if matches!(review.mode, super::import_review::ReviewMode::Create) {
                if c::button(
                    ui,
                    if review.backup_enabled {
                        "Backup codes · on"
                    } else {
                        "Backup codes · off"
                    },
                    c::ButtonStyle::Secondary,
                )
                .clicked()
                {
                    review.set_backup_enabled(!review.backup_enabled);
                    c::clear_sensitive_widget_memory(&self.context);
                }
                if review.backup_enabled {
                    let response = c::labeled_sensitive_multiline(
                        ui,
                        "account-backup-codes",
                        "Backup codes · one per line",
                        &mut review.backup_text,
                        review.backup_reveal,
                    );
                    if response.changed() || review.backup_review.is_none() {
                        if response.changed() {
                            review.error = None;
                        }
                        review.acknowledge_backup_duplicates(false);
                        review.backup_review = Some(
                            crate::domain::recovery_input::RecoveryInput::parse(
                                &review.backup_text,
                            )
                            .map(|p| (p.values.len(), p.duplicates)),
                        );
                    }
                    c::secondary(
                        ui,
                        "Optional service-issued codes. Blank lines and surrounding whitespace are removed. Up to 1 MiB; 4096 bytes per code.",
                    );
                    if c::button(
                        ui,
                        if review.backup_reveal {
                            "Conceal backup codes"
                        } else {
                            "Reveal backup codes"
                        },
                        c::ButtonStyle::Secondary,
                    )
                    .clicked()
                    {
                        review.backup_reveal = !review.backup_reveal;
                        c::clear_sensitive_widget_memory(&self.context);
                    }
                    match review.backup_review {
                        Some(Ok((unique, duplicates))) => {
                            c::secondary(
                                ui,
                                &format!(
                                    "{unique} unique code(s) · {duplicates} exact duplicate line(s)"
                                ),
                            );
                            if duplicates > 0
                                && c::button(
                                    ui,
                                    if review.backup_duplicates_acknowledged() {
                                        "Duplicate removal · acknowledged"
                                    } else {
                                        "Remove exact duplicates when saving"
                                    },
                                    c::ButtonStyle::Secondary,
                                )
                                .clicked()
                            {
                                review.acknowledge_backup_duplicates(
                                    !review.backup_duplicates_acknowledged(),
                                );
                            }
                        }
                        Some(Err(error))
                            if !review.backup_text.is_empty() || review.error.is_some() =>
                        {
                            c::status(
                                ui,
                                "backup-validation",
                                &error.to_string(),
                                c::StatusKind::Error,
                            )
                        }
                        _ => {}
                    }
                }
            }
            if let Ok(drafts) = self.state.vault().and_then(|v| review.drafts(v)) {
                let mut duplicates = 0;
                if let Ok(vault) = self.state.vault() {
                    for (index, draft) in drafts.iter().enumerate() {
                        if !vault.duplicates(draft, review.mode.exclude()).is_empty()
                            || drafts[..index].iter().any(|previous| {
                                let (same_seed, same_identity) = previous.duplicate_criteria(draft);
                                same_seed || same_identity
                            })
                        {
                            duplicates += 1;
                        }
                    }
                }
                if duplicates > 0 {
                    c::status(
                        ui,
                        "duplicate-warning",
                        if review.mode.is_edit() {
                            "Possible duplicates exist in this vault or trash. Save changes updates only the selected account and keeps every other entry."
                        } else {
                            "Possible duplicates exist in this vault, trash or selected batch. Save creates separate new entries and keeps every existing account. Cancel or change the selection if these are unwanted."
                        },
                        c::StatusKind::Info,
                    );
                }
            }
            if let Some(error) = &review.error {
                let status = ui
                    .scope(|ui| c::status(ui, "review-error", error, c::StatusKind::Error))
                    .response;
                if review.show_error {
                    status.scroll_to_me(Some(egui::Align::Center));
                    review.show_error = false;
                }
            }
            if c::button(
                ui,
                if review.mode.is_edit() {
                    "Save changes to this account"
                } else if review.mode.is_import() {
                    "Save selected as new accounts"
                } else {
                    "Save new account"
                },
                c::ButtonStyle::Primary,
            )
            .clicked()
            {
                match self.state.vault().and_then(|v| review.save_intent(v)) {
                    Ok(intent) => {
                        let repaint = self.context.clone();
                        let result = match &self.lifecycle {
                            Ok(lifecycle) => {
                                self.state.begin_save(intent, lifecycle.watch(), move || {
                                    repaint.request_repaint()
                                })
                            }
                            Err(error) => Err(*error),
                        };
                        if let Err(error) = result {
                            review.error = Some(error.to_string());
                            review.show_error = true;
                        } else {
                            for editor in &mut review.editors {
                                editor.reveal = false;
                            }
                            review.backup_reveal = false;
                            c::clear_sensitive_widget_memory(&self.context);
                            ui.ctx().request_repaint();
                        }
                    }
                    Err(error) => {
                        review.error = Some(format!(
                            "{error} Check every selected account; your edits are kept."
                        ));
                        review.show_error = true;
                    }
                }
            }
            if review.editors.len() > 1
                && c::button(ui, "Back to account selection", c::ButtonStyle::Secondary).clicked()
            {
                review.choosing = true;
                review.focus = true;
                for editor in &mut review.editors {
                    editor.reveal = false;
                }
                c::clear_sensitive_widget_memory(&self.context);
            }
        }
        if c::button(
            ui,
            if review.mode.is_import() {
                "Cancel import"
            } else {
                "Cancel"
            },
            c::ButtonStyle::Secondary,
        )
        .clicked()
        {
            if review.mode.is_import() {
                self.cancel_review();
            } else {
                review.discard_requested = true;
                review.focus = true;
            }
        }
    }
    fn preferences_changed(&mut self) {
        self.preferences_dirty = Some(Instant::now());
        self.preferences_error = false;
        self.ordered_ids = None;
        self.context
            .request_repaint_after(Duration::from_millis(500));
    }
    pub fn set_idle_timeout(&mut self, seconds: u32) {
        self.begin_mutation(SaveIntent::IdleTimeout(seconds));
    }
    fn begin_mutation(&mut self, intent: SaveIntent) -> bool {
        let preferences = matches!(&intent, SaveIntent::CodesPreferences(_));
        let Some(watch) = self.lifecycle.as_ref().ok().map(|l| l.watch()) else {
            self.status =
                Some("Windows lifecycle protection is unavailable. Restart before saving.");
            return false;
        };
        let repaint = self.context.clone();
        match self
            .state
            .begin_save(intent, watch, move || repaint.request_repaint())
        {
            Ok(()) => {
                self.pending_mutation = true;
                self.pending_preferences = preferences;
                true
            }
            Err(_) => {
                self.status = Some("Another operation is active. Finish it before saving changes.");
                false
            }
        }
    }
    fn persist_codes_preferences(&mut self) {
        if self.preferences_error
            || self.state.busy()
            || !self.state.is_unlocked()
            || self.review.is_some()
            || self.recovery_editor.is_some()
            || self.export_form.is_some()
            || self.export_picker.is_some()
            || self.backup_import.is_some()
            || self.backup_picker.is_some()
            || self.delete.is_some()
            || self.state.capture_busy()
            || self.image.is_some()
            || self.capture_preparing.is_some()
        {
            return;
        }
        if self
            .preferences_dirty
            .is_some_and(|at| at.elapsed() >= Duration::from_millis(500))
            && let Some(preferences) = self.codes_preferences.clone()
            && self.begin_mutation(SaveIntent::CodesPreferences(preferences))
        {
            self.preferences_dirty = None;
        }
    }
    fn advance_close(&mut self) {
        if !self.closing {
            return;
        }
        if let Some(outcome) = self.state.poll(Instant::now()) {
            self.pending_mutation = false;
            self.pending_preferences = false;
            self.preferences_error = outcome.is_err();
        }
        if self.preferences_error && !self.close_locked {
            return;
        }
        if self.state.is_unlocked()
            && !self.state.busy()
            && self.preferences_dirty.is_some()
            && !self.preferences_error
        {
            self.preferences_dirty = Instant::now().checked_sub(Duration::from_secs(1));
            self.persist_codes_preferences();
        }
        if self.close_locked
            || (!self.state.busy()
                && (self.preferences_dirty.is_none()
                    || self.preferences_error
                    || !self.state.is_unlocked()))
        {
            if !self.close_locked {
                self.lock();
            }
            if !self.clipboard.cleanup_pending()
                || self
                    .close_cleanup_deadline
                    .is_some_and(|deadline| Instant::now() >= deadline)
            {
                self.closing = false;
                self.exit_approved = true;
                self.context.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
    fn codes_toolbar(&mut self, ui: &mut egui::Ui) {
        let Some(preferences) = &mut self.codes_preferences else {
            return;
        };
        let folders = self
            .state
            .vault()
            .map(|vault| {
                vault
                    .folders()
                    .iter()
                    .filter(|folder| folder.state().deletion == DeletionState::Active)
                    .map(|folder| (folder.id(), folder.name().to_owned()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut choices = vec![
            (FolderFilter::All, "All accounts"),
            (FolderFilter::Unfiled, "Unfiled"),
        ];
        choices.extend(
            folders
                .iter()
                .map(|(id, name)| (FolderFilter::Folder(*id), name.as_str())),
        );
        let mut changed = false;
        let columns = if ui.available_width() >= theme::CODES_TOOLBAR_WIDE {
            3
        } else {
            2
        };
        let mut label = codes::sort_label(preferences.sort).to_owned();
        let mut view = if preferences.view == ViewMode::List {
            "List".to_owned()
        } else {
            "Grid".to_owned()
        };
        ui.columns(columns, |uis| {
            if c::labeled_choice(
                &mut uis[0],
                "codes-sort",
                "Sort accounts",
                &mut label,
                &codes::SORT_LABELS,
            ) && let Some(index) = codes::SORT_LABELS.iter().position(|value| *value == label)
            {
                preferences.sort = codes::SORTS[index];
                changed = true;
            }
            // Stable IDs, not possibly duplicate names, identify folders.
            if c::labeled_selection(
                &mut uis[1],
                "codes-folder",
                "Folder",
                &mut preferences.folder,
                &choices,
            ) {
                changed = true;
            }
            if columns == 3
                && c::labeled_choice(
                    &mut uis[2],
                    "codes-view",
                    "View",
                    &mut view,
                    &["List", "Grid"],
                )
            {
                preferences.view = if view == "List" {
                    ViewMode::List
                } else {
                    ViewMode::Grid
                };
                changed = true;
            }
        });
        if columns == 2 && c::labeled_choice(ui, "codes-view", "View", &mut view, &["List", "Grid"])
        {
            preferences.view = if view == "List" {
                ViewMode::List
            } else {
                ViewMode::Grid
            };
            changed = true;
        }
        if changed {
            self.preferences_changed();
        }
        if self.preferences_dirty.is_some() {
            c::secondary(ui, "Saving display preferences locally…");
        }
        if self.preferences_error
            && c::button(
                ui,
                "Retry saving display preferences",
                c::ButtonStyle::Secondary,
            )
            .clicked()
        {
            self.preferences_changed();
        }
    }
    fn copy_text(&mut self, text: Zeroizing<String>) {
        self.copy_with_window(text, None);
    }
    fn copy_with_window(
        &mut self,
        text: Zeroizing<String>,
        window: Option<crate::platform::clipboard::CodeWindow>,
    ) {
        #[cfg(debug_assertions)]
        {
            self.evidence_copy_queued = None;
        }
        if !self.state.is_unlocked() {
            return;
        }
        if let (Ok(vault), Ok(lifecycle)) = (self.state.vault(), &self.lifecycle)
            && lifecycle.healthy()
        {
            let seconds = vault.preferences().clipboard_timeout_seconds();
            let queued = if let Some(window) = window {
                self.clipboard
                    .copy_code(text, seconds, lifecycle.watch(), window)
            } else {
                self.clipboard.copy(text, seconds, lifecycle.watch())
            };
            #[cfg(debug_assertions)]
            {
                self.evidence_copy_queued = Some(queued);
            }
            if !queued {
                self.clipboard_feedback = Some((
                    "Clipboard queue is busy. Try Copy again.",
                    Instant::now() + Duration::from_secs(5),
                ));
            }
        } else {
            self.clipboard_feedback = Some((
                "Windows lifecycle protection is unavailable. Restart Authenticator before copying.",
                Instant::now() + Duration::from_secs(5),
            ));
        }
    }
    fn delete_dialog(&mut self, ctx: &egui::Context) {
        let Some((id, expected, mut dialog)) = self.delete.take() else {
            return;
        };
        let mut confirm = false;
        let close = dialog.show(ctx, |ui, first| {
            c::body(ui, "Move this account to Trash? Its recovery codes and folder memberships are retained. You can restore it from Trash when the Trash screen is available.");
            let cancel = c::button(ui, "Keep account", c::ButtonStyle::Secondary);
            if first { cancel.request_focus(); }
            if c::button(ui, "Move to Trash", c::ButtonStyle::Primary).clicked() { confirm = true; }
            cancel.clicked() || confirm
        });
        if confirm {
            self.begin_mutation(SaveIntent::Trash { id, expected });
        } else if !close {
            self.delete = Some((id, expected, dialog));
        }
    }
    fn code_cards(&mut self, ui: &mut egui::Ui) {
        let frame_clock = codes::FrameClock(self.wall_clock.unix_seconds());
        self.clock.observe(frame_clock.0, Instant::now());
        if frame_clock.0.is_err() {
            c::status(
                ui,
                "clock-unavailable",
                "Codes are unavailable. Check the Windows date, time and time zone, then retry. No remote clock check is performed.",
                c::StatusKind::Error,
            );
        } else if self.clock.shifted {
            c::body(
                ui,
                "The system clock changed unexpectedly. Codes use the current local clock. Check Windows date/time if a service rejects them; no remote accuracy check is performed.",
            );
            if c::button(ui, "Dismiss clock guidance", c::ButtonStyle::Secondary).clicked() {
                self.clock.shifted = false;
            }
        }
        let scroll_to = self
            .show_imported
            .then(|| self.imported.first().copied())
            .flatten();
        self.show_imported = false;
        let Ok(vault) = self.state.vault() else {
            return;
        };
        let Some(preferences) = &self.codes_preferences else {
            return;
        };
        if self.ordered_ids.is_none() {
            self.ordered_ids = Some(codes::ordered(vault, preferences));
        }
        let ids = self.ordered_ids.as_deref().unwrap_or_default();
        if ids.is_empty() {
            c::heading(ui, "No matching accounts", 2);
            c::body(
                ui,
                "Try another service or account name, or clear the search and folder filter.",
            );
            if c::button(ui, "Show all accounts", c::ButtonStyle::Secondary).clicked() {
                self.state.query.zeroize();
                if let Some(preferences) = &mut self.codes_preferences {
                    preferences.query.zeroize();
                    preferences.folder = FolderFilter::All;
                }
                self.preferences_changed();
            }
            return;
        }
        c::secondary(ui, &format!("{} matching account(s)", ids.len()));
        let columns = if preferences.view == ViewMode::Grid
            && ui.available_width() >= theme::CODES_GRID_MIN_WIDTH
        {
            2
        } else {
            1
        };
        let records = vault
            .accounts()
            .iter()
            .map(|account| (account.id(), account))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut action = None;
        for row in ids.chunks(columns) {
            ui.columns(columns, |uis| {
                for (index, id) in row.iter().enumerate() {
                    let Some(account) = records.get(id) else {
                        continue;
                    };
                    let ui = &mut uis[index];
                    ui.push_id(id, |ui| {
                        let card = c::card(ui, |ui| {
                            ui.horizontal(|ui| {
                                c::icon_tile(ui, c::Icon::Key);
                                ui.vertical(|ui| {
                                    c::heading(ui, account.credential().service(), 2);
                                    if let Some(identifier) =
                                        account.credential().account_identifier()
                                    {
                                        c::body(ui, identifier);
                                    }
                                });
                            });
                            if self.imported.contains(id) {
                                c::secondary(ui, "Saved · encrypted locally");
                            }
                            let visible = ui.clip_rect().intersects(egui::Rect::from_min_size(
                                ui.cursor().min,
                                egui::vec2(ui.available_width(), theme::CODE_CARD_CONTENT_HEIGHT),
                            ));
                            let draft = account.credential();
                            let code = if visible {
                                otp::current_code(draft.secret(), draft.settings(), &frame_clock)
                                    .ok()
                            } else {
                                None
                            };
                            if let Some(code) = &code {
                                // Nonselectable ordinary text; never a live region or an
                                // unguarded framework selection-copy route.
                                let _code_label = ui.add(
                                    egui::Label::new(
                                        theme::CODE.text(&*codes::formatted(code.expose_value())),
                                    )
                                    .selectable(false),
                                );
                                #[cfg(debug_assertions)]
                                self.evidence_code_bounds.push(_code_label.rect);
                                c::secondary(
                                    ui,
                                    &format!("{} seconds remaining", code.remaining_seconds),
                                );
                                ui.add(
                                    egui::ProgressBar::new(
                                        code.remaining_seconds as f32
                                            / draft.settings().period.seconds() as f32,
                                    )
                                    .desired_height(theme::PROGRESS_HEIGHT)
                                    .fill(theme::BLUE),
                                );
                            } else {
                                ui.add(egui::Label::new(theme::CODE.text("— —")).selectable(false));
                                c::secondary(
                                    ui,
                                    if frame_clock.0.is_err() {
                                        "Check Windows date and time"
                                    } else {
                                        "Code refreshes when visible"
                                    },
                                );
                                ui.add(
                                    egui::ProgressBar::new(0.)
                                        .desired_height(theme::PROGRESS_HEIGHT),
                                );
                            }
                            ui.add_enabled_ui(frame_clock.0.is_ok(), |ui| {
                                if c::button(ui, "Copy code", c::ButtonStyle::Primary).clicked() {
                                    action = Some(CodeAction::Copy(*id));
                                }
                            });
                            if c::button(ui, "Edit account", c::ButtonStyle::Secondary).clicked() {
                                action = Some(CodeAction::Edit(*id));
                            }
                            let delete = c::button(ui, "Delete account", c::ButtonStyle::Secondary);
                            if delete.clicked() {
                                action = Some(CodeAction::Delete(*id, account.state(), delete.id));
                            }
                        });
                        if scroll_to == Some(*id) {
                            card.response.scroll_to_me(Some(egui::Align::Center));
                        }
                    });
                }
            });
            ui.add_space(theme::SMALL_GAP);
        }
        match action {
            Some(CodeAction::Copy(id)) => {
                // A click on a rendered old frame must never copy its old value.
                let value = records.get(&id).and_then(|account| {
                    let now = self.wall_clock.unix_seconds().ok()?;
                    let value = codes::copy_value(account, &codes::FrameClock(Ok(now))).ok()?;
                    let period = account.credential().settings().period.seconds();
                    Some((
                        value,
                        crate::platform::clipboard::CodeWindow {
                            counter: now / period,
                            period,
                        },
                    ))
                });
                if let Some((value, window)) = value {
                    self.copy_with_window(value, Some(window));
                } else {
                    self.status = Some("Cannot copy a code while the system clock is unavailable.");
                }
            }
            Some(CodeAction::Edit(id)) => {
                self.review = records
                    .get(&id)
                    .and_then(|account| ImportReview::edit(account).ok());
                c::clear_sensitive_widget_memory(&self.context);
            }
            Some(CodeAction::Delete(id, expected, opener)) => {
                self.delete = Some((
                    id,
                    expected,
                    c::Dialog::new("Move account to Trash?", opener),
                ));
            }
            None => {}
        }
    }
    /// Geometry only, for the isolated debug fixture to redact real framebuffer
    /// screenshots before any pixels reach disk. No values/keys or AX IDs.
    #[cfg(debug_assertions)]
    pub fn code_bounds_for_evidence(&self) -> Vec<egui::Rect> {
        self.evidence_code_bounds.clone()
    }
    #[cfg(debug_assertions)]
    pub fn copy_diagnostics_for_evidence(&self) -> (bool, bool, bool, bool, Option<bool>) {
        (
            self.state.is_unlocked(),
            self.lifecycle.is_ok(),
            self.lifecycle.as_ref().is_ok_and(|value| value.healthy()),
            self.owner != 0,
            self.evidence_copy_queued,
        )
    }
    fn background_logic(&mut self, ctx: &egui::Context, now: Instant) {
        // run_logic deliberately retains the last UI events/time. Read neither
        // for activity/shortcuts here; only real show passes record new input.
        if self.lifecycle.as_ref().is_ok_and(|l| l.take_lock()) || self.state.idle_due(now) {
            self.lock();
        }
        self.persist_codes_preferences();
        if self.closing {
            self.advance_close();
        }
        if self.state.session_active() {
            ctx.request_repaint_after(Duration::from_secs(1));
        }
    }
    fn show_dialog(&mut self, ctx: &egui::Context) {
        if let Some(dialog) = &mut self.dialog {
            let help = self.help_open;
            let workflow = self.state.pending();
            let close=dialog.show(ctx,|ui,first| {
                if help {
                    if self.state.route == Route::BackupCodes {
                        c::body(ui,"Backup codes are issued by each service and may work only once. Keep an independent copy somewhere safe in case you lose access to this vault.");
                        c::secondary(ui,"Add Backup Codes selects an existing 2FA account. Search by its service or account identifier and check the selected account before continuing. Existing sets, including sets in Trash, are identified before you continue.");
                        c::secondary(ui,"Enter one service-issued code per line or read a local UTF-8 text file, then Preview changes before Save. Exact duplicates require review. Editing keeps exact matching used history; omitted codes move to Trash. Available and used totals include active codes only. Detail reveal, copy and used-state controls are not available in this build yet.");
                    }
                    c::body(ui,"Authenticator keeps your vault on this device. Use the three tabs for verification codes, recovery codes and local settings.");
                    c::secondary(ui,"Your vault passphrase cannot be reset. Settings → Export & Import saves a local encrypted backup protected by a separate backup passphrase. Keep the file and passphrase separately. Screenshot to Scan and image/manual entry work locally. Review each account before saving.");
                    c::secondary(ui,"Keyboard: Tab and Shift+Tab move focus. Enter or Space activates buttons. Ctrl+L locks the vault. Escape cancels passphrase work or closes dialogs.");
                } else {
                    c::body(ui,match workflow {
                        Some(Workflow::VaultLifecycle)=>"Vault setup and unlock will be connected in the next development stage. The vault remains locked.",
                        Some(Workflow::AddAccount)=>"Use Screenshot to Scan on the 2FA Codes screen to select a QR in another window. Editable review, saving and manual entry are the next development stage. No account was added.",
                        Some(Workflow::AddBackupCodes)=>"Choose Add Backup Codes on the Backup Codes tab to select an account.",
                        None=>"No action is pending.",
                    });
                }
                let close=c::button(ui,"Close",c::ButtonStyle::Primary);
                if first {close.request_focus();}
                close.clicked()
            });
            if close {
                self.dialog = None;
                self.help_open = false;
                self.state.cancel_pending();
            }
        }
    }
}
impl eframe::App for AuthenticatorApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.lock();
        self.clipboard.shutdown();
    }
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Eframe calls logic while hidden without painting UI. Lifecycle events
        // independently request this pass; monotonic idle checks need no input.
        self.background_logic(ctx, Instant::now());
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

#[cfg(all(test, windows))]
#[path = "backup_codes_tests.rs"]
mod backup_codes_tests;
#[cfg(all(test, windows))]
#[path = "backup_import_tests.rs"]
mod backup_import_tests;
#[cfg(all(test, windows))]
#[path = "export_tests.rs"]
mod export_tests;
#[cfg(all(test, windows))]
#[path = "recovery_editor_tests.rs"]
mod recovery_editor_tests;
#[cfg(all(test, windows))]
mod capture_privacy_tests {
    use super::*;
    use crate::domain::records::{Timestamp, Vault};
    fn labels(output: &egui::FullOutput) -> Vec<String> {
        output
            .platform_output
            .accesskit_update
            .as_ref()
            .map_or_else(Vec::new, |tree| {
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
    }
    #[test]
    fn route_searches_stay_separate_and_corrected_input_clears_only_query_feedback()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::domain::{
            otp::{OtpSettings, Secret},
            records::{AccountDraft, Timestamp, Vault},
        };
        let directory = tempfile::tempdir()?;
        let pass = || Zeroizing::new("disposable-route-search".into());
        let mut vault = Vault::new(Timestamp(1));
        let public = vault.add_account(
            AccountDraft::new(
                "Public search fixture",
                None,
                None,
                Secret::from_base32("MY")?,
                OtpSettings::default(),
            )?,
            Timestamp(2),
        )?;
        vault.add_account(
            AccountDraft::new(
                "Other search fixture",
                None,
                None,
                Secret::from_base32("MY")?,
                OtpSettings::default(),
            )?,
            Timestamp(3),
        )?;
        let mut preferences = vault.preferences().codes();
        preferences.query = Zeroizing::new("public".into());
        vault.set_codes_preferences(preferences, Timestamp(4))?;
        let mut store = VaultStore::open_directory(directory.path())?;
        store.create(pass(), vault)?;
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut app = AuthenticatorApp::with_context(context.clone(), Ok(store), 0);
        app.state.repository_mut()?.unlock(pass())?;
        let render = |app: &mut AuthenticatorApp, events| {
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(600., 1800.),
                    )),
                    events,
                    ..Default::default()
                },
                |root| app.show(root),
            );
            output.textures_delta.clear();
            output
        };
        let first = render(&mut app, vec![]);
        assert_eq!(app.state.query, "public");
        assert_eq!(app.ordered_ids.as_deref(), Some([public].as_slice()));
        assert!(
            labels(&first)
                .iter()
                .any(|text| text == "1 matching account(s)")
        );
        app.state.navigate(Route::BackupCodes);
        app.focus_form = true;
        render(&mut app, vec![]);
        render(&mut app, vec![egui::Event::Paste("absent".into())]);
        assert_eq!(app.state.backup_query, "absent");
        assert_eq!(app.state.query, "public");
        assert_eq!(
            &*app.codes_preferences.as_ref().ok_or("preferences")?.query,
            "public"
        );
        app.state.navigate(Route::Codes);
        app.focus_form = true;
        let restored = render(&mut app, vec![]);
        assert_eq!(app.state.query, "public");
        assert_eq!(app.ordered_ids.as_deref(), Some([public].as_slice()));
        assert!(
            labels(&restored)
                .iter()
                .any(|text| text == "Public search fixture")
        );
        assert!(
            !labels(&restored)
                .iter()
                .any(|text| text == "Other search fixture" || text == "absent")
        );
        assert_eq!(&*app.state.vault()?.preferences().codes().query, "public");

        // Drive the actual TextEdit change path. These are deterministic input
        // events, not a claim of an OS pointer/keyboard acceptance check.
        let replace = |value: String| {
            vec![
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: Some(egui::Key::A),
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    },
                },
                egui::Event::Paste(value),
            ]
        };
        app.status = Some("Unrelated operation notice");
        render(
            &mut app,
            replace("x".repeat(CodesPreferences::MAX_QUERY_BYTES + 1)),
        );
        assert_eq!(app.state.query.len(), CodesPreferences::MAX_QUERY_BYTES + 1);
        let invalid = render(&mut app, vec![]);
        assert!(
            labels(&invalid)
                .iter()
                .any(|text| text.contains("Use at most 1024 UTF-8 bytes"))
        );
        assert_eq!(
            &*app.codes_preferences.as_ref().ok_or("preferences")?.query,
            "public"
        );
        assert_eq!(app.status, Some("Unrelated operation notice"));
        render(&mut app, replace("public".into()));
        app.preferences_dirty = None;
        let corrected = render(&mut app, vec![]);
        assert_eq!(app.state.query, "public");
        assert_eq!(app.status, Some("Unrelated operation notice"));
        assert!(
            labels(&corrected)
                .iter()
                .all(|text| !text.contains("1024 UTF-8 bytes"))
        );
        assert_eq!(app.ordered_ids.as_deref(), Some([public].as_slice()));
        app.state.navigate(Route::BackupCodes);
        render(&mut app, vec![]);
        assert_eq!(app.state.backup_query, "absent");
        app.lock();
        assert!(app.state.query.is_empty() && app.state.backup_query.is_empty());
        drop(app);
        let mut reopened = VaultStore::open_directory(directory.path())?;
        reopened.unlock(pass())?;
        assert_eq!(&*reopened.vault()?.preferences().codes().query, "public");
        Ok(())
    }
    #[test]
    fn ordinary_close_write_failure_retains_hidden_latest_query_until_retry_or_deliberate_discard()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let pass = || Zeroizing::new("disposable-close-error".into());
        let mut store = VaultStore::open_directory(directory.path())?;
        store.create(pass(), Vault::new(Timestamp(1)))?;
        drop(store);
        for retry in [true, false] {
            let original = std::fs::read(directory.path().join("vault.bin"))?;
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = AuthenticatorApp::with_context(
                ctx.clone(),
                VaultStore::open_directory(directory.path()),
                0,
            );
            app.state.repository_mut()?.unlock(pass())?;
            let input = || egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(600., 900.),
                )),
                ..Default::default()
            };
            let mut first = ctx.run_ui(input(), |root| app.show(root));
            first.textures_delta.clear();
            app.state.query = if retry {
                "private-close-error-query"
            } else {
                "private-discard-query"
            }
            .into();
            app.codes_preferences.as_mut().ok_or("preferences")?.query =
                Zeroizing::new(app.state.query.clone());
            app.preferences_changed();
            std::fs::write(
                directory.path().join("vault.bin"),
                b"disposable-preference-conflict",
            )?;
            let mut close = input();
            close
                .viewports
                .entry(egui::ViewportId::ROOT)
                .or_default()
                .events
                .push(egui::ViewportEvent::Close);
            let mut started = ctx.run_ui(close, |root| app.show(root));
            started.textures_delta.clear();
            let deadline = Instant::now() + Duration::from_secs(25);
            let mut error_labels = Vec::new();
            while !app.preferences_error && Instant::now() < deadline {
                let mut frame = ctx.run_ui(input(), |root| app.show(root));
                frame.textures_delta.clear();
                error_labels = labels(&frame);
                assert!(
                    !frame
                        .viewport_output
                        .get(&egui::ViewportId::ROOT)
                        .is_some_and(|v| v
                            .commands
                            .iter()
                            .any(|command| matches!(command, egui::ViewportCommand::Close)))
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            assert!(app.closing && app.preferences_error && !app.close_locked);
            assert!(
                error_labels
                    .iter()
                    .any(|v| v.contains("Display changes could not be saved"))
            );
            assert!(
                error_labels
                    .iter()
                    .any(|v| v == "Retry saving display changes")
            );
            assert!(
                error_labels
                    .iter()
                    .any(|v| v == "Close without saving display changes")
            );
            assert!(
                error_labels
                    .iter()
                    .all(|v| !v.contains("private-close-error"))
            );
            assert_eq!(
                std::fs::read(directory.path().join("vault.bin"))?,
                b"disposable-preference-conflict"
            );
            if retry {
                std::fs::write(directory.path().join("vault.bin"), &original)?;
                app.preferences_changed();
            } else {
                app.lock();
                assert!(app.close_locked && app.close_cleanup_deadline.is_some());
                app.advance_close();
            }
            while app.closing && Instant::now() < deadline {
                let mut frame = ctx.run_ui(input(), |root| app.show(root));
                frame.textures_delta.clear();
                std::thread::sleep(Duration::from_millis(10));
            }
            assert!(!app.closing && !app.state.is_unlocked());
            assert!(app.state.query.is_empty());
            drop(app);
            if !retry {
                assert_eq!(
                    std::fs::read(directory.path().join("vault.bin"))?,
                    b"disposable-preference-conflict"
                );
                std::fs::write(directory.path().join("vault.bin"), &original)?;
            }
            let mut reopened = VaultStore::open_directory(directory.path())?;
            reopened.unlock(pass())?;
            assert_eq!(
                &*reopened.vault()?.preferences().codes().query,
                "private-close-error-query"
            );
        }
        Ok(())
    }
    #[test]
    fn hidden_logic_ignores_stale_events_and_requests_one_nonsensitive_ax_paint_without_showing()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::domain::{
            otp::{OtpSettings, Secret},
            records::AccountDraft,
        };
        let directory = tempfile::tempdir()?;
        let pass = || Zeroizing::new("disposable-hidden-logic".into());
        let mut vault = Vault::new(Timestamp(1));
        vault.add_account(
            AccountDraft::new(
                "private-hidden-service-sentinel",
                None,
                Some("private-hidden-user-sentinel"),
                Secret::from_base32("MY")?,
                OtpSettings::default(),
            )?,
            Timestamp(2),
        )?;
        let mut store = VaultStore::open_directory(directory.path())?;
        store.create(pass(), vault)?;
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = AuthenticatorApp::with_context(ctx.clone(), Ok(store), 0);
        app.state.repository_mut()?.unlock(pass())?;
        let mut shown = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(600., 1800.),
            )),
            events: vec![egui::Event::PointerMoved(egui::pos2(30., 30.))],
            ..Default::default()
        };
        let mut frame = ctx.run_ui(shown.clone(), |root| app.show(root));
        frame.textures_delta.clear();
        assert!(
            labels(&frame)
                .iter()
                .any(|value| value.contains("private-hidden-service"))
        );
        assert!(ctx.input(|input| !input.events.is_empty()));
        let base = Instant::now();
        app.state.activity(base);
        shown.events.clear();
        let info = shown.viewports.entry(egui::ViewportId::ROOT).or_default();
        info.minimized = Some(true);
        let logic = ctx.run_logic(&shown, |context| {
            app.background_logic(context, base + Duration::from_secs(301))
        });
        assert!(!app.state.session_active());
        assert!(app.state.query.is_empty());
        assert!(
            ctx.input(|input| !input.events.is_empty()),
            "the framework contract retains old UI input"
        );
        let commands = logic
            .viewport_commands
            .get(&egui::ViewportId::ROOT)
            .ok_or("hidden scrub command")?;
        assert_eq!(
            commands
                .iter()
                .filter(|command| matches!(command, egui::ViewportCommand::RequestPaintWhileHidden))
                .count(),
            1
        );
        assert!(!commands.iter().any(|command| matches!(
            command,
            egui::ViewportCommand::Visible(true) | egui::ViewportCommand::Minimized(false)
        )));
        // Execute the one pass the integration was explicitly asked to run.
        let mut scrubbed = ctx.run_ui(shown.clone(), |root| app.show(root));
        scrubbed.textures_delta.clear();
        assert!(
            labels(&scrubbed)
                .iter()
                .all(|value| !value.contains("private-hidden-"))
        );
        assert!(
            scrubbed
                .viewport_output
                .get(&egui::ViewportId::ROOT)
                .is_none_or(|viewport| !viewport.commands.iter().any(|command| matches!(
                    command,
                    egui::ViewportCommand::RequestPaintWhileHidden
                )))
        );
        Ok(())
    }
    #[test]
    fn live_query_coalesces_during_real_held_write_and_normal_close_flushes_latest_without_ax()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let pass = || Zeroizing::new("disposable-close-preferences".into());
        let mut store = VaultStore::open_directory(directory.path())?;
        store.create(pass(), Vault::new(Timestamp(1)))?;
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = AuthenticatorApp::with_context(ctx.clone(), Ok(store), 0);
        app.state.repository_mut()?.unlock(pass())?;
        let input = || egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(600., 900.),
            )),
            ..Default::default()
        };
        let mut initial = ctx.run_ui(input(), |root| app.show(root));
        initial.textures_delta.clear();
        app.state.query = "private-first-query".into();
        app.codes_preferences.as_mut().ok_or("preferences")?.query =
            Zeroizing::new(app.state.query.clone());
        app.preferences_changed();
        app.preferences_dirty = Instant::now().checked_sub(Duration::from_secs(1));
        let gate = Arc::new(crate::storage::test_gate::CommitGate::default());
        let _release = gate.release_on_drop();
        app.state.hold_next_save_after_admission(gate.clone());
        app.persist_codes_preferences();
        assert!(gate.wait_entered(Duration::from_secs(10)));
        app.state.query = "private-latest-query".into();
        let preferences = app.codes_preferences.as_mut().ok_or("preferences")?;
        preferences.query = Zeroizing::new(app.state.query.clone());
        preferences.sort = crate::domain::records::SortOrder::ServiceDescending;
        preferences.view = ViewMode::Grid;
        app.preferences_changed();
        let mut live = ctx.run_ui(input(), |root| app.show(root));
        live.textures_delta.clear();
        assert!(labels(&live).iter().any(|v| v == "private-latest-query"));
        assert!(app.state.busy());
        let mut close = input();
        close
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .events
            .push(egui::ViewportEvent::Close);
        let mut closing = ctx.run_ui(close, |root| app.show(root));
        closing.textures_delta.clear();
        assert!(app.closing);
        assert!(
            labels(&closing)
                .iter()
                .all(|v| !v.contains("private-first") && !v.contains("private-latest"))
        );
        assert!(
            closing
                .viewport_output
                .get(&egui::ViewportId::ROOT)
                .is_some_and(|viewport| viewport
                    .commands
                    .iter()
                    .any(|command| matches!(command, egui::ViewportCommand::CancelClose)))
        );
        gate.release();
        let deadline = Instant::now() + Duration::from_secs(30);
        while app.closing && Instant::now() < deadline {
            let mut frame = ctx.run_ui(input(), |root| app.show(root));
            frame.textures_delta.clear();
            assert!(labels(&frame).iter().all(|v| !v.contains("private-latest")));
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(!app.closing);
        assert!(!app.state.is_unlocked());
        assert!(app.state.query.is_empty());
        assert!(app.exit_approved);
        // The integration reports the programmatic Close back as another close
        // request. It must be allowed through, not start another CancelClose loop.
        let mut approved = input();
        approved
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .events
            .push(egui::ViewportEvent::Close);
        let mut final_frame = ctx.run_ui(approved, |root| app.show(root));
        final_frame.textures_delta.clear();
        assert!(!app.closing);
        assert!(
            final_frame
                .viewport_output
                .get(&egui::ViewportId::ROOT)
                .is_none_or(|viewport| !viewport
                    .commands
                    .iter()
                    .any(|command| matches!(command, egui::ViewportCommand::CancelClose)))
        );
        drop(app);
        let mut reopened = VaultStore::open_directory(directory.path())?;
        reopened.unlock(pass())?;
        let preferences = reopened.vault()?.preferences();
        assert_eq!(&*preferences.codes().query, "private-latest-query");
        assert_eq!(
            preferences.sort(),
            crate::domain::records::SortOrder::ServiceDescending
        );
        assert_eq!(preferences.view(), ViewMode::Grid);
        Ok(())
    }
    #[test]
    fn codes_frame_clock_jump_failure_preferences_restore_copy_output_and_lock_ax()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::domain::{
            otp::{OtpSettings, Secret},
            records::{AccountDraft, Timestamp, Vault},
        };
        use std::sync::atomic::AtomicU64;
        struct TestClock(Arc<AtomicU64>);
        impl Clock for TestClock {
            fn unix_seconds(&self) -> Result<u64, crate::error::AppError> {
                let value = self.0.load(Ordering::SeqCst);
                if value == u64::MAX {
                    Err(crate::error::AppError::ClockUnavailable)
                } else {
                    Ok(value)
                }
            }
        }
        let directory = tempfile::tempdir()?;
        let pass = || Zeroizing::new("disposable-code-frame".into());
        let mut v = Vault::new(Timestamp(1));
        v.add_account(
            AccountDraft::new(
                "Private clock metadata sentinel",
                None,
                Some("fixture-user"),
                Secret::from_base32("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")?,
                OtpSettings::default(),
            )?,
            Timestamp(2),
        )?;
        let mut preferences = v.preferences().codes();
        preferences.query = Zeroizing::new("metadata".into());
        preferences.view = ViewMode::Grid;
        v.set_codes_preferences(preferences, Timestamp(3))?;
        let mut store = VaultStore::open_directory(directory.path())?;
        store.create(pass(), v)?;
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = AuthenticatorApp::with_context(ctx.clone(), Ok(store), 0);
        app.state.repository_mut()?.unlock(pass())?;
        let seconds = Arc::new(AtomicU64::new(59));
        app.wall_clock = Box::new(TestClock(seconds.clone()));
        let render = |app: &mut AuthenticatorApp| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(600., 1800.),
                    )),
                    ..Default::default()
                },
                |root| app.show(root),
            );
            output.textures_delta.clear();
            output
        };
        let before = render(&mut app);
        let copy_ids = |frame: &egui::FullOutput| {
            frame
                .platform_output
                .accesskit_update
                .as_ref()
                .map(|tree| {
                    tree.nodes
                        .iter()
                        .filter(|(_, node)| node.label() == Some("Copy code"))
                        .map(|(id, _)| *id)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };
        let ids_before = copy_ids(&before);
        assert!(!ids_before.is_empty());
        app.clipboard_feedback = Some(("Copied.", Instant::now() + Duration::from_secs(5)));
        let feedback_frame = render(&mut app);
        assert_eq!(
            copy_ids(&feedback_frame),
            ids_before,
            "clipboard feedback must preserve card/control identity and keyboard focus"
        );
        app.clipboard_feedback = None;
        let cleared_feedback = render(&mut app);
        assert_eq!(copy_ids(&cleared_feedback), ids_before);
        assert_eq!(app.state.query, "metadata");
        assert_eq!(
            app.codes_preferences.as_ref().map(|p| p.view),
            Some(ViewMode::Grid)
        );
        let initial = labels(&before)
            .into_iter()
            .find(|v| v.chars().filter(char::is_ascii_digit).count() == 6 && v.len() == 7)
            .ok_or("missing initial code")?;
        seconds.store(60, Ordering::SeqCst);
        let boundary = render(&mut app);
        assert!(!labels(&boundary).contains(&initial));
        let next = labels(&boundary)
            .into_iter()
            .find(|v| v.chars().filter(char::is_ascii_digit).count() == 6 && v.len() == 7)
            .ok_or("missing next code")?;
        seconds.store(0, Ordering::SeqCst);
        let back = render(&mut app);
        assert!(!labels(&back).contains(&next));
        assert!(labels(&back).iter().any(|v| v.contains("clock changed")));
        seconds.store(u64::MAX, Ordering::SeqCst);
        let failed = render(&mut app);
        assert!(
            labels(&failed)
                .iter()
                .any(|v| v.contains("Codes are unavailable"))
        );
        assert!(!labels(&failed).contains(&initial));
        assert!(!labels(&failed).contains(&next));
        assert!(
            failed
                .platform_output
                .accesskit_update
                .as_ref()
                .is_some_and(|tree| tree
                    .nodes
                    .iter()
                    .filter(|(_, node)| node.label() == Some("Copy code"))
                    .all(|(_, node)| node.is_disabled()))
        );
        // Simulate the framework output route directly. Native owner=0 fails
        // before OpenClipboard: no interactive user clipboard is touched.
        ctx.copy_text("public framework metadata".into());
        let guarded = render(&mut app);
        assert!(
            !guarded
                .platform_output
                .commands
                .iter()
                .any(|command| matches!(command, egui::OutputCommand::CopyText(_)))
        );
        app.lifecycle = Err(crate::error::AppError::StorageUnavailable);
        app.copy_with_window(
            Zeroizing::new("123456".into()),
            Some(crate::platform::clipboard::CodeWindow {
                counter: 1,
                period: 30,
            }),
        );
        assert!(
            app.clipboard_feedback
                .is_some_and(|(text, _)| text.contains("lifecycle protection is unavailable"))
        );
        #[cfg(debug_assertions)]
        assert_eq!(app.evidence_copy_queued, None);
        app.lock();
        let locked = render(&mut app);
        assert!(app.state.query.is_empty());
        assert!(app.codes_preferences.is_none());
        assert!(app.ordered_ids.is_none());
        assert!(
            labels(&locked)
                .iter()
                .all(|value| !value.contains("metadata sentinel")
                    && !value.contains("fixture-user")
                    && !value.contains(&initial)
                    && !value.contains(&next))
        );
        Ok(())
    }
    #[test]
    fn lock_clears_ui_before_an_actual_admitted_save_action_is_released()
    -> Result<(), Box<dyn std::error::Error>> {
        use super::super::import_review::{Editor, ReviewMode};
        use crate::storage::test_gate::CommitGate;
        let directory = tempfile::tempdir()?;
        let pass = || Zeroizing::new("disposable-admitted-save-only".into());
        let mut store = VaultStore::open_directory(directory.path())?;
        store.create(pass(), Vault::new(Timestamp(1)))?;
        let original = std::fs::read(directory.path().join("vault.bin"))?;
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut app = AuthenticatorApp::with_context(context.clone(), Ok(store), 0);
        app.state.repository_mut()?.unlock(pass())?;
        app.state.query.push_str("private-held-query-sentinel");
        let mut review = ImportReview::manual();
        review.mode = ReviewMode::Import;
        review.editors.push(Editor::blank());
        for (index, editor) in review.editors.iter_mut().enumerate() {
            editor
                .service
                .push_str(&format!("private-held-review-sentinel-{index}"));
            editor.secret.push_str("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");
        }
        review
            .backup_text
            .push_str("private-held-recovery-sentinel");
        app.review = Some(review);
        let input = || egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(600., 1500.),
            )),
            ..Default::default()
        };
        let mut before = context.run_ui(input(), |root| app.show(root));
        before.textures_delta.clear();
        assert!(
            labels(&before)
                .iter()
                .any(|label| label.contains("private-held-review-sentinel"))
        );
        let widget = egui::Id::unique("held-save-native-history");
        egui::text_edit::TextEditState::default().store(&context, widget);
        assert!(egui::text_edit::TextEditState::load(&context, widget).is_some());
        let generation = app.state.generation();
        let gate = Arc::new(CommitGate::default());
        let intent = app
            .review
            .as_ref()
            .ok_or("review missing")?
            .save_intent(app.state.vault()?)?;
        let watch = app.lifecycle.as_ref().map_err(|e| *e)?.watch();
        app.state.hold_next_save_after_admission(gate.clone());
        let mut app = std::thread::scope(
            |scope| -> Result<AuthenticatorApp, Box<dyn std::error::Error>> {
                // Must release before scope joins if an assertion/timeout fails.
                let _release_on_failure = gate.release_on_drop();
                app.state.begin_save(intent, watch, || {})?;
                assert!(gate.wait_entered(Duration::from_secs(10)));
                let (proof_tx, proof_rx) = std::sync::mpsc::sync_channel(1);
                let ui_context = context.clone();
                let locking = scope.spawn(move || {
                    app.lock();
                    let widgets_gone =
                        egui::text_edit::TextEditState::load(&ui_context, widget).is_none();
                    let mut locked = ui_context.run_ui(input(), |root| app.show(root));
                    locked.textures_delta.clear();
                    let names = labels(&locked);
                    let ax_private = names.iter().any(|label| {
                        label.contains("private-held-")
                            || label.contains("GEZDGNBV")
                            || label.contains("Secret key")
                            || label.contains("Save selected")
                    });
                    let proof = (
                        app.state.generation() != generation,
                        app.state.query.is_empty(),
                        app.review.is_none(),
                        widgets_gone,
                        !ax_private,
                        app.state.busy(),
                        !app.state.session_active(),
                        app.capture_message.is_none(),
                    );
                    let _ = proof_tx.send(proof);
                    app
                });
                let proof = proof_rx.recv_timeout(Duration::from_secs(2))?;
                assert_eq!(proof, (true, true, true, true, true, true, true, true));
                assert!(
                    !gate.is_released(),
                    "privacy/Lock proof must precede action release"
                );
                assert_eq!(std::fs::read(directory.path().join("vault.bin"))?, original);
                gate.release();
                Ok(locking.join().map_err(|_| "UI lock thread panic")?)
            },
        )?;
        let deadline = Instant::now() + Duration::from_secs(30);
        while app.state.busy() {
            assert!(Instant::now() < deadline);
            assert!(
                app.state.poll(Instant::now()).is_none(),
                "late save must not publish success"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(!app.state.is_unlocked());
        assert!(app.state.take_imported().is_none());
        assert!(app.review.is_none());
        assert!(app.capture_message.is_none());
        assert_ne!(std::fs::read(directory.path().join("vault.bin"))?, original);
        app.state.repository_mut()?.unlock(pass())?;
        let accounts = app.state.vault()?.accounts();
        assert_eq!(
            accounts.len(),
            2,
            "already-admitted operation completes as a whole"
        );
        assert!(accounts.iter().all(|a| {
            a.credential()
                .service()
                .starts_with("private-held-review-sentinel-")
        }));
        Ok(())
    }
    #[test]
    fn editable_review_masks_secret_and_lock_removes_all_credential_nodes()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let mut store = VaultStore::open_directory(directory.path())?;
        store.create(
            Zeroizing::new("disposable-review-nodes".into()),
            Vault::new(Timestamp(1)),
        )?;
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut app = AuthenticatorApp::with_context(context.clone(), Ok(store), 0);
        app.state
            .repository_mut()?
            .unlock(Zeroizing::new("disposable-review-nodes".into()))?;
        let mut review = ImportReview::manual();
        review.editors[0]
            .service
            .push_str("private-review-service-sentinel");
        review.editors[0]
            .account
            .push_str("private-review-account-sentinel");
        review.editors[0]
            .secret
            .push_str("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");
        app.review = Some(review);
        let input = || egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(340., 1500.),
            )),
            ..Default::default()
        };
        let mut frame = context.run_ui(input(), |root| app.show(root));
        frame.textures_delta.clear();
        let review_labels = labels(&frame);
        assert!(
            review_labels
                .iter()
                .any(|l| l.contains("private-review-service-sentinel"))
        );
        assert!(review_labels.iter().any(|l| l.contains('•')));
        assert!(!review_labels.iter().any(|l| l.contains("GEZDGNBV")));
        assert!(app.review.as_ref().is_some_and(|r| r.approved().is_ok()));
        app.dialog = Some(c::Dialog::new(
            "Offline help",
            egui::Id::unique("review-help-opener"),
        ));
        app.help_open = true;
        let mut modal = context.run_ui(input(), |root| app.show(root));
        modal.textures_delta.clear();
        let mut escape = input();
        escape.events.push(egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: Some(egui::Key::Escape),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
        let mut closed = context.run_ui(escape, |root| app.show(root));
        closed.textures_delta.clear();
        assert!(
            app.review.is_some(),
            "Escape in help must preserve unsaved review"
        );
        assert!(app.dialog.is_none());
        egui::Popup::open_id(&context, egui::Id::unique("disposable-review-popup"));
        app.lock();
        assert!(!egui::Popup::is_any_open(&context));
        let mut frame = context.run_ui(input(), |root| app.show(root));
        frame.textures_delta.clear();
        let locked_labels = labels(&frame);
        assert!(!locked_labels.iter().any(|l| l.contains("private-review-")
            || l.contains("Secret key")
            || l.contains("GEZDGNBV")
            || l.contains("Save selected")));
        assert!(app.review.is_none());
        assert!(app.image.is_none());
        assert!(app.imported.is_empty());
        Ok(())
    }
    #[test]
    fn preparing_accessibility_frame_precedes_hide_and_lock_clears_pending_capture()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let mut store = VaultStore::open_directory(directory.path())?;
        let mut vault = Vault::new(Timestamp(1));
        let mut preferences = vault.preferences().codes();
        preferences.query = Zeroizing::new("disposable private query sentinel".into());
        vault.set_codes_preferences(preferences, Timestamp(1))?;
        store.create(Zeroizing::new("disposable-private-frame".into()), vault)?;
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut app = AuthenticatorApp::with_context(context.clone(), Ok(store), 0);
        app.state
            .repository_mut()?
            .unlock(Zeroizing::new("disposable-private-frame".into()))?;
        app.state.query = "disposable private query sentinel".into();
        let input = || egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(600., 800.),
            )),
            ..Default::default()
        };
        let mut before = context.run_ui(input(), |root| app.show(root));
        before.textures_delta.clear();
        assert!(
            labels(&before)
                .iter()
                .any(|s| s.contains("private query sentinel"))
        );
        app.capture_preparing = Some((app.state.generation(), false));
        c::clear_sensitive_widget_memory(&context);
        let mut preparing = context.run_ui(input(), |root| app.show(root));
        preparing.textures_delta.clear();
        assert!(app.capture_preparing.is_some_and(|(_, ready)| ready));
        assert!(!app.state.capture_busy());
        assert!(
            !labels(&preparing)
                .iter()
                .any(|s| s.contains("private query sentinel") || s == "Search accounts")
        );
        assert!(
            labels(&preparing)
                .iter()
                .any(|s| s.contains("Preparing private"))
        );
        assert!(
            !preparing
                .viewport_output
                .values()
                .flat_map(|v| v.commands.iter())
                .any(|c| matches!(c, egui::ViewportCommand::Visible(false)))
        );
        let mut hidden = context.run_ui(input(), |root| app.show(root));
        hidden.textures_delta.clear();
        assert!(
            hidden
                .viewport_output
                .values()
                .flat_map(|v| v.commands.iter())
                .any(|c| matches!(c, egui::ViewportCommand::Visible(false)))
        );
        assert!(
            !labels(&hidden)
                .iter()
                .any(|s| s.contains("private query sentinel") || s == "Search accounts")
        );
        assert_eq!(
            app.state.begin_capture(
                0,
                app.lifecycle.as_ref().map_err(|_| "listener")?.watch(),
                || {}
            ),
            Err(crate::error::AppError::VaultBusy)
        );
        assert_eq!(
            app.state.request_scan_alternative(ScanAlternative::Image),
            Err(crate::error::AppError::VaultBusy)
        );
        app.lock();
        let mut locked = context.run_ui(input(), |root| app.show(root));
        locked.textures_delta.clear();
        assert!(app.state.query.is_empty());
        assert!(app.capture_preparing.is_none());
        assert!(app.state.scan_review().is_none());
        assert_eq!(
            app.state.request_scan_alternative(ScanAlternative::Manual),
            Err(crate::error::AppError::VaultLocked)
        );
        assert!(
            !labels(&locked)
                .iter()
                .any(|s| s.contains("private query sentinel"))
        );
        Ok(())
    }
}
impl Drop for AuthenticatorApp {
    fn drop(&mut self) {
        self.lock();
    }
}
