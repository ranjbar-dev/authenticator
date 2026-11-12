//! Inventory-derived shell. Setup/unlock and credential workflows are later tasks.
use super::{
    components as c,
    state::{AppState, Route, Workflow},
    theme,
};
use crate::{domain::records::DeletionState, storage::VaultStore};
use eframe::egui;

pub struct AuthenticatorApp {
    pub state: AppState,
    dialog: Option<c::Dialog>,
    help_open: bool,
    status: Option<&'static str>,
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
        Self {
            state: AppState::new(repository),
            dialog: None,
            help_open: false,
            status: None,
        }
    }
    pub fn lock(&mut self) {
        self.state.lock();
        self.dialog = None;
        self.help_open = false;
        self.status = None;
    }
    fn open_workflow(&mut self, workflow: Workflow, opener: egui::Id) {
        if self.state.request(workflow).is_ok() {
            self.dialog = Some(c::Dialog::new("Development build", opener));
            self.help_open = false;
        }
    }
    pub fn show(&mut self, root: &mut egui::Ui) {
        let ctx = root.ctx().clone();
        c::restore_dialog_focus(&ctx);
        // This single scroll holder keeps all controls reachable at high app zoom.
        egui::CentralPanel::default().frame(egui::Frame::new().fill(theme::BACKGROUND).inner_margin(theme::PADDING)).show(root,|ui| {
            egui::ScrollArea::vertical().id_salt("screen-holder").show(ui,|ui| {
                    ui.set_max_width(theme::CONTENT_MAX_WIDTH.min(ui.available_width()));
                c::secondary(ui,"AUTHENTICATOR · LOCAL VAULT");
                c::toolbar(ui,|ui| {
                    c::heading(ui,self.state.route.title(),1);
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
                    c::empty_state(ui,c::Icon::Lock,"Your vault is locked","Account names, codes and recovery codes stay hidden until you unlock your local vault.");
                    let action=c::button(ui,"Vault setup and unlock",c::ButtonStyle::Primary);
                    if action.clicked() {self.open_workflow(Workflow::VaultLifecycle,action.id);}
                    c::secondary(ui,"Development build: setup and unlock are the next implementation stage.");
                } else { self.unlocked_content(ui); }
                if let Some(status)=self.status {c::status(ui,"shell-feedback",status,c::StatusKind::Info);}
            });
        });
        self.show_dialog(&ctx);
    }
    fn unlocked_content(&mut self, ui: &mut egui::Ui) {
        match self.state.route {
            Route::Codes | Route::BackupCodes => {
                let backups = self.state.route == Route::BackupCodes;
                c::labeled_field(
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
                        "Vault data is encrypted on this device. Settings and recovery workflows are still being built.",
                    );
                    c::secondary(ui, "This build does not change your vault preferences.");
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
                    c::secondary(ui,"This development build contains the shared interface. Setup, unlock, capture, import and recovery flows are not yet available.");
                    c::secondary(ui,"Keyboard: Tab and Shift+Tab move focus. Enter or Space activates buttons. Escape closes dialogs.");
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
