//! Inventory-derived shell and local vault lifecycle.
use super::{
    components as c,
    state::{AppState, Route, VaultAction, Workflow},
    theme,
};
use crate::{domain::records::DeletionState, storage::VaultStore};
use eframe::egui;
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
        theme::apply(&cc.egui_ctx);
        let state = AppState::new(repository);
        let exists = state.vault_exists().unwrap_or(true);
        let context = cc.egui_ctx.clone();
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
        }
    }
    pub fn lock(&mut self) {
        self.state.lock();
        self.dialog = None;
        self.help_open = false;
        self.status = None;
        self.clear_form();
        self.form_error = None;
        self.focus_form = true;
        self.operation_cancelled = self.state.busy();
    }
    fn clear_form(&mut self) {
        self.passphrase.zeroize();
        self.confirmation.zeroize();
        // Includes native TextEdit undo/cursor memory for search and dialogs.
        // Secret editors additionally drop undo history after every edit pass.
        c::clear_sensitive_widget_memory(&self.context);
    }
    fn open_workflow(&mut self, workflow: Workflow, opener: egui::Id) {
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
        if let Some(outcome) = self.state.poll(now) {
            self.clear_form();
            self.focus_form = true;
            self.form_error = outcome.err().map(|e| e.to_string());
            self.operation_cancelled = false;
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
            egui::ScrollArea::vertical().id_salt("screen-holder").show(ui,|ui| {
                    ui.set_max_width(theme::CONTENT_MAX_WIDTH.min(ui.available_width()));
                c::secondary(ui,"AUTHENTICATOR · LOCAL VAULT");
                c::toolbar(ui,|ui| {
                    c::heading(ui,if self.state.is_unlocked() {self.state.route.title()} else if self.exists {"Unlock vault"} else {"Create vault"},1);
                    let help=c::icon_button(ui,c::Icon::Help,"Help for this screen");
                    if help.clicked() {self.dialog=Some(c::Dialog::new("Offline help",help.id)); self.help_open=true;}
                    if self.state.is_unlocked() && c::icon_button(ui,c::Icon::Lock,"Lock vault").clicked() {self.lock();}
                });
                c::secondary(ui,if self.state.is_unlocked() {"Vault unlocked on this device"} else {"Vault locked · credentials hidden"});
                ui.add_space(theme::SMALL_GAP);
                if let Some(route)=c::tabs(ui,self.state.route) { self.state.navigate(route); self.status=None; }
                ui.add_space(theme::SMALL_GAP);
                if let Some(error)=self.state.storage_error() {
                    c::status(ui,"storage-error",&error.to_string(),c::StatusKind::Error);
                    c::secondary(ui,"Close other Authenticator instances and restart. No replacement vault is created here.");
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
        match self.state.route {
            Route::Codes | Route::BackupCodes => {
                let backups = self.state.route == Route::BackupCodes;
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
                            "Add an account to keep its verification codes here. Account capture and entry are a later development stage."
                        },
                    );
                } else {
                    c::card(ui, |ui| {
                        c::heading(ui, "Vault records available", 2);
                        c::body(
                            ui,
                            &format!(
                                "{count} records in this section. The full list workflow is not implemented in this build."
                            ),
                        );
                    });
                }
                let action = c::button(
                    ui,
                    if backups {
                        "Add Backup Codes"
                    } else {
                        "Add account"
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
    fn show_dialog(&mut self, ctx: &egui::Context) {
        if let Some(dialog) = &mut self.dialog {
            let help = self.help_open;
            let workflow = self.state.pending();
            let close=dialog.show(ctx,|ui,first| {
                if help {
                    c::body(ui,"Authenticator keeps your vault on this device. Use the three tabs for verification codes, recovery codes and local settings.");
                    c::secondary(ui,"Your vault passphrase cannot be reset. Keep a separate encrypted backup and its passphrase. Capture, import and backup screens are still being built.");
                    c::secondary(ui,"Keyboard: Tab and Shift+Tab move focus. Enter or Space activates buttons. Ctrl+L locks the vault. Escape cancels passphrase work or closes dialogs.");
                } else {
                    c::body(ui,match workflow {
                        Some(Workflow::VaultLifecycle)=>"Vault setup and unlock will be connected in the next development stage. The vault remains locked.",
                        Some(Workflow::AddAccount)=>"Account capture and manual entry are not available yet. No account was added.",
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
impl Drop for AuthenticatorApp {
    fn drop(&mut self) {
        self.lock();
    }
}
