//! Selected recovery-code detail. Retains IDs and at most one guarded Undo.
use super::{backup_codes::Action, components as c, state::SaveIntent, theme};
use crate::domain::{
    records::*,
    recovery_code::{CodeChange, CodeTarget},
};
use eframe::egui;
use std::collections::BTreeSet;

#[derive(Default)]
pub struct Detail {
    route: Option<(AccountId, BackupSetId)>,
    revealed: BTreeSet<BackupCodeId>,
    page: usize,
    pub undo: Option<(CodeTarget, Option<Timestamp>)>,
    delete: Option<(CodeTarget, c::Dialog)>,
    focus_rows: bool,
    focus_code: Option<BackupCodeId>,
    pub sensitive_rects: Vec<egui::Rect>,
}
impl Detail {
    pub fn has_dialog(&self) -> bool {
        self.delete.is_some()
    }
    pub fn clear(&mut self) {
        // Keep this frame's redaction bounds even when a later control leaves
        // the view after its value shapes have already been painted.
        let bounds = std::mem::take(&mut self.sensitive_rects);
        *self = Self::default();
        self.sensitive_rects = bounds;
    }
    pub fn conceal(&mut self) {
        self.revealed.clear();
    }
    pub fn changed(&mut self) {
        self.conceal();
        self.undo = None;
        self.delete = None;
    }
    pub fn saved(
        &mut self,
        vault: &Vault,
        account: AccountId,
        set: BackupSetId,
        code: BackupCodeId,
        previous: Option<Timestamp>,
        undoable: bool,
    ) {
        self.changed();
        if undoable && self.route == Some((account, set)) {
            self.undo = CodeTarget::capture(vault, account, set, code)
                .ok()
                .map(|t| (t, previous));
        }
        self.focus_rows = true;
        self.focus_code = CodeTarget::capture(vault, account, set, code)
            .ok()
            .map(|t| t.code_id());
        // Undo can be invoked from another page. Render the affected active
        // row's page before restoring its focus after the busy layout.
        if let Some(index) = vault
            .backup_sets()
            .iter()
            .find(|s| s.id() == set)
            .and_then(|s| {
                s.codes()
                    .iter()
                    .filter(|c| c.state().deletion == DeletionState::Active)
                    .position(|c| c.id() == code)
            })
        {
            self.page = index / super::backup_codes::PAGE_SIZE;
        }
    }
    pub fn content(
        &mut self,
        ui: &mut egui::Ui,
        vault: &Vault,
        account: AccountId,
        set_id: BackupSetId,
    ) -> Action {
        self.sensitive_rects.clear();
        if self.route != Some((account, set_id)) {
            self.clear();
            self.route = Some((account, set_id));
        }
        let mut action = Action::None;
        let Some(set) = vault.backup_sets().iter().find(|s| {
            s.id() == set_id
                && s.account_id() == account
                && s.state().deletion == DeletionState::Active
        }) else {
            self.changed();
            return action;
        };
        c::body(
            ui,
            "These codes are issued by the service and may work only once. Copying never marks a code used. Mark it used after the service accepts it.",
        );
        if self
            .undo
            .as_ref()
            .is_some_and(|(target, _)| target.check(vault).is_err())
        {
            self.undo = None;
        }
        ui.push_id("recovery-used-undo", |ui| {
            ui.add_enabled_ui(self.undo.is_some(), |ui| {
                if c::button(ui, "Undo last used-state change", c::ButtonStyle::Secondary).clicked()
                    && let Some((target, previous)) = &self.undo
                {
                    action = Action::Save(Box::new(SaveIntent::RecoveryCode {
                        target: target.clone(),
                        change: CodeChange::RestoreUsed(*previous),
                    }));
                }
            });
        });
        c::secondary(
            ui,
            "Undo applies to the last saved used-state change in this view. Leaving the view or locking clears Undo.",
        );
        let active: Vec<_> = set
            .codes()
            .iter()
            .filter(|c| c.state().deletion == DeletionState::Active)
            .collect();
        if active.is_empty() {
            c::body(
                ui,
                "No active codes in this set. Edit the set to add service-issued codes. Codes in Trash remain encrypted.",
            );
        }
        let pages = active.len().div_ceil(super::backup_codes::PAGE_SIZE).max(1);
        self.page = self.page.min(pages - 1);
        c::secondary(
            ui,
            &format!(
                "Page {} of {} · {} active codes",
                self.page + 1,
                pages,
                active.len()
            ),
        );
        let old_page = self.page;
        ui.push_id("recovery-code-page", |ui| {
            ui.columns(2, |cols| {
                cols[0].add_enabled_ui(self.page > 0, |ui| {
                    if c::button(ui, "Previous codes", c::ButtonStyle::Secondary).clicked() {
                        self.page -= 1;
                    }
                });
                cols[1].add_enabled_ui(self.page + 1 < pages, |ui| {
                    if c::button(ui, "Next codes", c::ButtonStyle::Secondary).clicked() {
                        self.page += 1;
                    }
                });
            });
        });
        if old_page != self.page {
            self.revealed.clear();
            self.focus_rows = true;
            self.focus_code = None;
        }
        for (index, code) in active
            .into_iter()
            .enumerate()
            .skip(self.page * super::backup_codes::PAGE_SIZE)
            .take(super::backup_codes::PAGE_SIZE)
        {
            ui.push_id(("recovery-code", code.id()), |ui| {
                c::card(ui, |ui| {
                    c::body(
                        ui,
                        &format!(
                            "Code {} · {}",
                            index + 1,
                            if code.used_at().is_some() {
                                "Used"
                            } else {
                                "Available"
                            }
                        ),
                    );
                    // Actions precede the bounded value holder: even a 4096-byte
                    // code cannot push its controls beyond thousands of pixels.
                    ui.columns(2, |columns| {
                        let shown = self.revealed.contains(&code.id());
                        let reveal = columns[0]
                            .push_id("reveal", |ui| {
                                c::button(
                                    ui,
                                    if shown { "Conceal" } else { "Reveal" },
                                    c::ButtonStyle::Secondary,
                                )
                            })
                            .inner;
                        reveal.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::Role::Button,
                                true,
                                format!(
                                    "{} code {}",
                                    if shown { "Conceal" } else { "Reveal" },
                                    index + 1
                                ),
                            )
                        });
                        if self.focus_rows && self.focus_code.is_none() {
                            reveal.request_focus();
                            reveal.scroll_to_me(Some(egui::Align::Center));
                            self.focus_rows = false;
                        }
                        if reveal.clicked() {
                            if shown {
                                self.revealed.remove(&code.id());
                            } else {
                                self.revealed.insert(code.id());
                            }
                        }
                        let copy = columns[1]
                            .push_id("copy", |ui| {
                                c::button(ui, "Copy", c::ButtonStyle::Secondary)
                            })
                            .inner;
                        copy.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::Role::Button,
                                true,
                                format!("Copy code {}", index + 1),
                            )
                        });
                        if copy.clicked() {
                            action = Action::Copy {
                                account,
                                set: set_id,
                                code: code.id(),
                            };
                        }
                    });
                    ui.columns(2, |columns| {
                        let used = code.used_at().is_some();
                        let mark = columns[0]
                            .push_id("used", |ui| {
                                c::button(
                                    ui,
                                    if used { "Mark available" } else { "Mark used" },
                                    c::ButtonStyle::Secondary,
                                )
                            })
                            .inner;
                        if self.focus_rows && self.focus_code == Some(code.id()) {
                            mark.request_focus();
                            mark.scroll_to_me(Some(egui::Align::Center));
                            self.focus_rows = false;
                        }
                        mark.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::Role::Button,
                                true,
                                format!(
                                    "Mark code {} {}",
                                    index + 1,
                                    if used { "available" } else { "used" }
                                ),
                            )
                        });
                        if mark.clicked()
                            && let Ok(target) =
                                CodeTarget::capture(vault, account, set_id, code.id())
                        {
                            action = Action::Save(Box::new(SaveIntent::RecoveryCode {
                                target,
                                change: CodeChange::MarkUsed(!used),
                            }));
                        }
                        let delete = columns[1]
                            .push_id("trash", |ui| {
                                c::button(ui, "Delete…", c::ButtonStyle::Secondary)
                            })
                            .inner;
                        delete.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::Role::Button,
                                true,
                                format!("Delete code {}…", index + 1),
                            )
                        });
                        if delete.clicked()
                            && let Ok(target) =
                                CodeTarget::capture(vault, account, set_id, code.id())
                        {
                            self.delete = Some((
                                target,
                                c::Dialog::new("Move recovery code to Trash?", delete.id),
                            ));
                        }
                    });
                    if self.revealed.contains(&code.id()) {
                        let holder = egui::ScrollArea::vertical()
                            .id_salt("revealed-value")
                            .scroll_source(egui::scroll_area::ScrollSource {
                                drag: egui::scroll_area::DragScroll::Never,
                                ..Default::default()
                            })
                            .max_height(theme::MULTILINE_HEIGHT)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::Label::new(theme::BODY.text(code.expose_value()))
                                        .wrap()
                                        .selectable(false),
                                );
                            });
                        self.sensitive_rects.push(holder.inner_rect);
                    } else {
                        c::body(ui, "Code concealed");
                    }
                });
            });
            ui.add_space(theme::SMALL_GAP);
        }
        if let Some((target, mut dialog)) = self.delete.take()
            && target.check(vault).is_ok()
        {
            let mut consent = false;
            let close = dialog.show(ui.ctx(), |ui, first| {
                    c::body(ui, "Move this code to Trash? Its value and used history stay encrypted. The account and other codes stay active. Restoration and permanent deletion belong to the Trash screen, which is not available in this build yet.");
                    let keep = c::button(ui, "Keep code", c::ButtonStyle::Secondary);
                    if first { keep.request_focus(); }
                    consent = c::button(ui, "Move code to Trash", c::ButtonStyle::Primary).clicked();
                    keep.clicked() || consent
                });
            if consent {
                // The row disappears after saving; focus the first visible
                // row (or Edit if empty), never a removed dialog opener.
                action = Action::Save(Box::new(SaveIntent::RecoveryCode {
                    target,
                    change: CodeChange::Trash,
                }));
            } else if !close {
                self.delete = Some((target, dialog));
            }
        }
        action
    }
}
