//! Inventory-derived shell and local vault lifecycle.
use super::import_review::ImportReview;
use super::{
    components as c,
    state::{AppState, CaptureEvent, Route, ScanAlternative, VaultAction, Workflow},
    theme,
};
use crate::{
    domain::{
        clock::SystemClock,
        otp,
        records::{AccountId, DeletionState},
    },
    storage::VaultStore,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
type ImageResult = Result<Option<crate::qr::ScanReview>, crate::qr::QrError>;
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
    image: Option<ImageJob>,
    imported: Vec<AccountId>,
    show_imported: bool,
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
            image: None,
            imported: Vec::new(),
            show_imported: false,
        }
    }
    pub fn lock(&mut self) {
        egui::Popup::close_all(&self.context);
        self.review = None;
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
        self.context
            .send_viewport_cmd(egui::ViewportCommand::Visible(true));
    }
    fn clear_form(&mut self) {
        self.passphrase.zeroize();
        self.confirmation.zeroize();
        // Includes native TextEdit undo/cursor memory for search and dialogs.
        // Secret editors additionally drop undo history after every edit pass.
        c::clear_sensitive_widget_memory(&self.context);
    }
    fn open_workflow(&mut self, workflow: Workflow, opener: egui::Id) {
        if workflow == Workflow::AddAccount {
            self.start_alternative(ScanAlternative::Manual);
            return;
        }
        if self.state.request(workflow).is_ok() {
            self.dialog = Some(c::Dialog::new("Development build", opener));
            self.help_open = false;
        }
    }
    pub fn show(&mut self, root: &mut egui::Ui) {
        let ctx = root.ctx().clone();
        let now = Instant::now();
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
                i.viewport().close_requested()
                    || lock_shortcut
                    || (i.key_pressed(egui::Key::Escape) && self.state.busy())
            })
        {
            self.lock();
        }
        let was_importing = self.state.importing();
        let save_mode = self.review.as_ref().map(|r| r.mode);
        if let Some(outcome) = self.state.poll(now) {
            if was_importing {
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
        if !self.state.session_active() && !self.state.busy() {
            self.review = None;
            self.image = None;
        }
        self.poll_image();
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
        ctx.request_repaint_after(Duration::from_millis(200));
        c::restore_dialog_focus(&ctx);
        // This single scroll holder keeps all controls reachable at high app zoom.
        egui::CentralPanel::default().frame(egui::Frame::new().fill(theme::BACKGROUND).inner_margin(theme::PADDING)).show(root,|ui| {
            let holder = egui::ScrollArea::vertical().id_salt("screen-holder");
            let holder = if self.review.as_ref().is_some_and(|r|r.focus) { holder.vertical_scroll_offset(0.) } else { holder };
            holder.show(ui,|ui| {
                    ui.set_max_width(theme::CONTENT_MAX_WIDTH.min(ui.available_width()));
                c::secondary(ui,"AUTHENTICATOR · LOCAL VAULT");
                c::toolbar(ui,|ui| {
                    c::heading(ui,if self.state.importing() {"Saving accounts"} else if self.review.as_ref().is_some_and(|r| !r.mode.is_import()) {"Account"} else if self.state.is_unlocked() && (self.capture_preparing.is_some() || self.state.capture_busy()) {"Screenshot to Scan"} else if self.state.session_active() {self.state.route.title()} else if self.exists {"Unlock vault"} else {"Create vault"},1);
                    let help=c::icon_button(ui,c::Icon::Help,"Help for this screen");
                    if help.clicked() {self.dialog=Some(c::Dialog::new("Offline help",help.id)); self.help_open=true;}
                    if self.state.session_active() && c::icon_button(ui,c::Icon::Lock,"Lock vault").clicked() {self.lock();}
                });
                c::secondary(ui,if self.state.session_active() {"Vault unlocked on this device"} else {"Vault locked · credentials hidden"});
                ui.add_space(theme::SMALL_GAP);
                if self.review.is_none() && self.image.is_none() && self.capture_preparing.is_none() && !self.state.capture_busy() && let Some(route)=c::tabs(ui,self.state.route) { self.state.navigate(route); self.status=None; }
                ui.add_space(theme::SMALL_GAP);
                if let Some(error)=self.state.storage_error() {
                    c::status(ui,"storage-error",&error.to_string(),c::StatusKind::Error);
                    c::secondary(ui,"Close other Authenticator instances and restart. No replacement vault is created here.");
                } else if self.state.importing() { self.review_content(ui);
                } else if !self.state.is_unlocked() {
                    self.locked_content(ui);
                } else { self.unlocked_content(ui); }
                if let Some(status)=self.status {c::status(ui,"shell-feedback",status,c::StatusKind::Info);}
            });
        });
        self.show_dialog(&ctx);
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
            Route::Codes | Route::BackupCodes => {
                let backups = self.state.route == Route::BackupCodes;
                if !backups {
                    self.capture_controls(ui);
                    // The opening pass itself also removes old query/count/
                    // account widgets from ROOT's new visual + accessible tree.
                    if self.capture_preparing.is_some() {
                        return;
                    }
                }
                let search = c::labeled_field(
                    ui,
                    "search",
                    if backups {
                        "Search backup codes"
                    } else {
                        "Search accounts"
                    },
                    &mut self.state.query,
                    "Search your unlocked vault.",
                    None,
                );
                if self.focus_form {
                    search.request_focus();
                    self.focus_form = false;
                }
                // Counts derive solely from the authenticated repository. No sample
                // accounts or invented current codes exist in production UI.
                let count = self
                    .state
                    .vault()
                    .map(|v| {
                        if backups {
                            v.backup_sets().len()
                        } else {
                            v.accounts()
                                .iter()
                                .filter(|a| a.state().deletion == DeletionState::Active)
                                .count()
                        }
                    })
                    .unwrap_or(0);
                if count == 0 {
                    c::empty_state(
                        ui,
                        if backups {
                            c::Icon::Recovery
                        } else {
                            c::Icon::Key
                        },
                        if backups {
                            "No backup codes yet"
                        } else {
                            "No accounts yet"
                        },
                        if backups {
                            "Keep recovery codes with the account they belong to. Adding backup codes is a later development stage."
                        } else {
                            "Scan a QR code from another window, review the account, then save it securely."
                        },
                    );
                } else if backups {
                    c::secondary(ui, "Backup-code management is the next development stage.");
                } else {
                    self.code_cards(ui);
                }
                if !backups {
                    return;
                }
                let action = c::button(
                    ui,
                    if backups {
                        "Add Backup Codes"
                    } else {
                        "Manually Input"
                    },
                    c::ButtonStyle::Primary,
                );
                if action.clicked() {
                    self.open_workflow(
                        if backups {
                            Workflow::AddBackupCodes
                        } else {
                            Workflow::AddAccount
                        },
                        action.id,
                    );
                }
            }
            Route::Settings => {
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
                            self.status = Some(if self.state.set_idle_timeout(seconds).is_ok() {
                                "Auto lock saved in your encrypted vault."
                            } else {
                                "Auto lock could not be saved. Your previous setting is unchanged."
                            });
                        }
                    }
                });
                if c::button(ui, "Lock vault", c::ButtonStyle::Primary).clicked() {
                    self.lock();
                }
            }
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
        let start = c::button(
            ui,
            if self.capture_failed {
                "Retry Screenshot to Scan"
            } else {
                "Screenshot to Scan"
            },
            c::ButtonStyle::Primary,
        );
        if self.capture_focus {
            start.request_focus();
            self.capture_focus = false;
        }
        c::secondary(
            ui,
            "Drag around a QR code in another window. Escape or Cancel returns here. Up to 4096 pixels per side and 4,194,304 pixels total.",
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
        if c::button(ui, "Manually Input", c::ButtonStyle::Secondary).clicked() {
            self.start_alternative(ScanAlternative::Manual);
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
    fn code_cards(&mut self, ui: &mut egui::Ui) {
        let scroll_to = self
            .show_imported
            .then(|| self.imported.first().copied())
            .flatten();
        self.show_imported = false;
        let Ok(vault) = self.state.vault() else {
            return;
        };
        let query = Zeroizing::new(self.state.query.to_lowercase());
        let mut found = false;
        // Imported identity is ordered first, derived from committed account IDs.
        let accounts = vault
            .accounts()
            .iter()
            .filter(|a| a.state().deletion == DeletionState::Active);
        let mut edit = None;
        for account in accounts
            .clone()
            .filter(|a| self.imported.contains(&a.id()))
            .chain(accounts.filter(|a| !self.imported.contains(&a.id())))
        {
            let draft = account.credential();
            if !query.is_empty()
                && !draft.service().to_lowercase().contains(&*query)
                && !draft
                    .account_identifier()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&*query)
            {
                continue;
            }
            found = true;
            let card = c::card(ui, |ui| {
                if self.imported.contains(&account.id()) {
                    c::secondary(ui, "SAVED · encrypted locally");
                }
                c::heading(ui, draft.service(), 2);
                if let Some(identifier) = draft.account_identifier() {
                    c::body(ui, identifier);
                }
                match otp::current_code(draft.secret(), draft.settings(), &SystemClock) {
                    Ok(code) => {
                        // Ordinary text, not a live region: avoid announcing every
                        // countdown/code update to assistive technology.
                        ui.add(egui::Label::new(theme::CODE.text(code.expose_value())));
                        c::secondary(
                            ui,
                            &format!(
                                "{} seconds remaining · {} digits",
                                code.remaining_seconds,
                                draft.settings().digits.count()
                            ),
                        );
                        ui.add(
                            egui::ProgressBar::new(
                                code.remaining_seconds as f32
                                    / draft.settings().period.seconds() as f32,
                            )
                            .fill(theme::BLUE),
                        );
                    }
                    Err(error) => {
                        c::status(ui, "clock-error", &error.to_string(), c::StatusKind::Error);
                    }
                }
                if c::button(ui, "Edit account", c::ButtonStyle::Secondary).clicked() {
                    edit = Some(account.id());
                }
            });
            if scroll_to == Some(account.id()) {
                card.response.scroll_to_me(Some(egui::Align::Center));
            }
            ui.add_space(theme::SMALL_GAP);
        }
        if let Some(id) = edit {
            self.review = vault
                .accounts()
                .iter()
                .find(|a| a.id() == id)
                .and_then(|a| ImportReview::edit(a).ok());
            c::clear_sensitive_widget_memory(&self.context);
        }
        if !found {
            c::secondary(
                ui,
                "No matching accounts. Clear the search to see your accounts.",
            );
        }
    }
    fn show_dialog(&mut self, ctx: &egui::Context) {
        if let Some(dialog) = &mut self.dialog {
            let help = self.help_open;
            let workflow = self.state.pending();
            let close=dialog.show(ctx,|ui,first| {
                if help {
                    c::body(ui,"Authenticator keeps your vault on this device. Use the three tabs for verification codes, recovery codes and local settings.");
                    c::secondary(ui,"Your vault passphrase cannot be reset. Keep a separate encrypted backup and its passphrase. Screenshot to Scan and image/manual entry work locally. Review each account before saving. Backup screens are still being built.");
                    c::secondary(ui,"Keyboard: Tab and Shift+Tab move focus. Enter or Space activates buttons. Ctrl+L locks the vault. Escape cancels passphrase work or closes dialogs.");
                } else {
                    c::body(ui,match workflow {
                        Some(Workflow::VaultLifecycle)=>"Vault setup and unlock will be connected in the next development stage. The vault remains locked.",
                        Some(Workflow::AddAccount)=>"Use Screenshot to Scan on the 2FA Codes screen to select a QR in another window. Editable review, saving and manual entry are the next development stage. No account was added.",
                        Some(Workflow::AddBackupCodes)=>"The account selector and backup-code editor are not available yet. No recovery codes were added.",
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
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

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
        store.create(
            Zeroizing::new("disposable-private-frame".into()),
            Vault::new(Timestamp(1)),
        )?;
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
