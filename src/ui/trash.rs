//! Metadata-only Trash presentation; commits use the existing repository worker.
use super::{backup_codes, components as c, theme};
use crate::{
    domain::{
        records::*,
        trash::{self, Counts, ItemId, Operation, Review},
    },
    error::AppError,
};
use eframe::egui;

pub struct Session {
    pub saving: bool,
    pub preparing: bool,
    pub busy_focus: bool,
    pub focus: bool,
    pub review: Option<Confirmation>,
    pub error: Option<String>,
    page: usize,
    back_opener: Option<egui::Id>,
    pending_opener: Option<egui::Id>,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            saving: false,
            preparing: false,
            busy_focus: false,
            focus: true,
            review: None,
            error: None,
            page: 0,
            back_opener: None,
            pending_opener: None,
        }
    }
}
pub struct Confirmation {
    pub review: Review,
    dialog: c::Dialog,
    consent: bool,
    conflict_page: usize,
}
pub enum Action {
    None,
    Back,
    Lock,
    Save(Review, bool),
    Prepare(Operation, egui::Id),
}

impl Session {
    pub fn fail(&mut self, message: String) {
        self.saving = false;
        self.error = Some(message);
        if let Some(r) = &mut self.review {
            r.dialog = c::Dialog::new(r.dialog.title, r.dialog.opener);
        }
    }
    pub fn start_review(&mut self, opener: egui::Id) {
        self.pending_opener = Some(opener);
        self.preparing = true;
        self.busy_focus = true;
        self.review = None;
        self.error = None;
    }
    pub fn prepared(&mut self, result: Result<Review, AppError>) {
        self.preparing = false;
        match result {
            Ok(review) => {
                if let Some(opener) = self.pending_opener.take().or(self.back_opener) {
                    let title = match review.operation() {
                        Operation::Restore(_) => "Restore from Trash?",
                        Operation::Delete(_) => "Permanently delete this item?",
                        Operation::Empty => "Empty trash permanently?",
                    };
                    self.review = Some(Confirmation {
                        review,
                        dialog: c::Dialog::new(title, opener),
                        consent: false,
                        conflict_page: 0,
                    });
                    self.error = None;
                } else {
                    self.error = Some("Return to Trash and review the item again.".into());
                    self.focus = true;
                }
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.focus = true;
            }
        }
    }
    pub fn has_dialog(&self) -> bool {
        self.review.is_some()
    }
    pub fn complete(&mut self, result: Result<(), AppError>, ctx: &egui::Context) {
        self.saving = false;
        if let Err(e) = result {
            self.fail(format!(
                "{e} No partial Trash change was saved. Retry or close this review and review the current item again."
            ));
        } else {
            self.review = None;
            self.error = None;
            self.focus = true;
            c::clear_sensitive_widget_memory(ctx);
        }
    }
    pub fn content(&mut self, ui: &mut egui::Ui, vault: &Vault) -> Action {
        if self.saving || self.preparing {
            c::progress(
                ui,
                if self.preparing {
                    "Reviewing Trash contents…"
                } else {
                    "Saving encrypted Trash changes…"
                },
                None,
            );
            return if c::button(ui, "Cancel and lock", c::ButtonStyle::Secondary).clicked() {
                Action::Lock
            } else {
                Action::None
            };
        }
        let rows = trash::items(vault);
        let back = ui
            .push_id("trash-back", |ui| {
                c::button(ui, "Back to Settings", c::ButtonStyle::Secondary)
            })
            .inner;
        self.back_opener = Some(back.id);
        if self.focus {
            back.request_focus();
            back.scroll_to_me(None);
            self.focus = false;
        }
        if back.clicked() {
            return Action::Back;
        }
        c::body(ui, &format!("{} item(s) in Trash", rows.len()));
        c::body(
            ui,
            "Items stay here until you explicitly permanently delete them. There is no automatic expiry.",
        );
        c::secondary(
            ui,
            "This count lists each record with its own Trash state, including memberships and recovery records. Active children hidden by a deleted parent are kept with that parent, not counted as separate deletions. Restoring a parent does not restore children deleted separately.",
        );
        let mut request = None;
        ui.push_id("trash-empty", |ui| {
            ui.add_enabled_ui(!rows.is_empty(), |ui| {
                let response = c::button(ui, "Empty trash…", c::ButtonStyle::Secondary);
                if response.clicked() {
                    request = Some((Operation::Empty, response.id));
                }
            });
        });
        ui.push_id("trash-error", |ui| {
            if self.review.is_none()
                && let Some(error) = &self.error
            {
                c::status(ui, "error", error, c::StatusKind::Error);
            }
        });
        if rows.is_empty() {
            c::heading(ui, "Trash is empty", 2);
            c::body(
                ui,
                "Accounts, folders and recovery codes you move to Trash will appear here. Return to Settings to continue.",
            );
        } else {
            pages(ui, &mut self.page, rows.len(), "trash-pages");
            for item in rows
                .into_iter()
                .skip(self.page * backup_codes::PAGE_SIZE)
                .take(backup_codes::PAGE_SIZE)
            {
                ui.push_id(("trash-item", item.id), |ui| {
                    c::card(ui, |ui| {
                        identity(ui, vault, item.id);
                        c::secondary(ui, &format!("Deleted: {}", date(item.deleted_at)));
                        let restore = item_button(
                            ui,
                            "Restore…",
                            "Restore",
                            item.id,
                            c::ButtonStyle::Primary,
                        );
                        if restore.clicked() {
                            request = Some((Operation::Restore(item.id), restore.id));
                        }
                        let delete = item_button(
                            ui,
                            "Permanently delete…",
                            "Permanently delete",
                            item.id,
                            c::ButtonStyle::Secondary,
                        );
                        if delete.clicked() {
                            request = Some((Operation::Delete(item.id), delete.id));
                        }
                    });
                });
                ui.add_space(theme::SMALL_GAP);
            }
        }
        if let Some((operation, opener)) = request {
            return Action::Prepare(operation, opener);
        }
        self.confirmation(ui, vault)
    }
    fn confirmation(&mut self, ui: &mut egui::Ui, vault: &Vault) -> Action {
        let Some(mut confirmation) = self.review.take() else {
            return Action::None;
        };
        let mut submit = false;
        let mut reopen = None;
        let mut review_current = false;
        let is_restore = matches!(confirmation.review.operation(), Operation::Restore(_));
        let close = confirmation.dialog.show(ui.ctx(), |ui, first| {
            let keep = c::button(ui, "Keep in Trash", c::ButtonStyle::Secondary);
            if first { keep.request_focus(); keep.scroll_to_me(Some(egui::Align::Min)); }
            if let Some(error) = &self.error {
                c::status(ui, "trash-confirm-error", error, c::StatusKind::Error);
                if c::button(ui, "Close review and show current items", c::ButtonStyle::Secondary).clicked() { review_current = true; }
            }
            if let Operation::Restore(id) | Operation::Delete(id) = confirmation.review.operation() { identity(ui, vault, id); }
            let summary = confirmation.review.summary();
            if is_restore {
                c::body(ui, "Restore this record's own state. Its existing seed, settings, values, IDs and used-code history stay unchanged. Records deleted separately remain in Trash.");
                c::secondary(ui, &format!("Associated records preserved, including any still in Trash: {}", counts(summary.affected)));
                if let Operation::Restore(ItemId::Set(id)) = confirmation.review.operation()
                    && vault.backup_sets().iter().find(|s| s.id() == id).is_some_and(|s| s.codes().iter().all(|c| c.state().deletion != DeletionState::Active))
                {
                    c::body(ui, "Every code in this set was deleted separately. After restoring the set, restore individual codes from Trash to make them available.");
                }
                if !summary.parents.is_empty() {
                    c::body(ui, "Restore these parents first so this item can be visible. Each parent opens a separate review; no parent is restored automatically.");
                    for id in &summary.parents {
                        ui.push_id(("trash-parent", id), |ui| {
                            identity(ui, vault, *id);
                            if item_button(ui, "Review parent restore…", "Review parent restore", *id, c::ButtonStyle::Primary).clicked() { reopen = Some(*id); }
                        });
                    }
                }
                if !summary.conflicts.is_empty() {
                    c::body(ui, "Matching entries already exist, including possibly in Trash. Restoring keeps separate records and does not overwrite or merge them.");
                    pages(ui, &mut confirmation.conflict_page, summary.conflicts.len(), "trash-conflicts");
                    for id in summary.conflicts.iter().skip(confirmation.conflict_page * backup_codes::PAGE_SIZE).take(backup_codes::PAGE_SIZE) {
                        ui.push_id(("trash-conflict", id), |ui| {
                            identity(ui, vault, *id);
                            c::secondary(ui, if id.state(vault).is_some_and(|s| s.deletion == DeletionState::Active) { "Existing active record · preserved" } else { "Existing record in Trash · preserved" });
                        });
                    }
                    let consent = ui.add(egui::Checkbox::new(&mut confirmation.consent, theme::BODY.text("Restore as a separate record and keep matching entries")));
                    c::focus(ui, &consent);
                }
            } else {
                c::body(ui, &format!("Permanently remove these encrypted records and logical references: {}", counts(summary.affected)));
                c::body(ui, &format!("This includes {} associated record(s) whose own state is active, even if a deleted parent currently hides them.", summary.active_associations));
                if summary.empty_sets > 0 {
                    c::body(ui, &format!("{} backup-code set(s) will become empty when their final stored code is removed, so those sets will also be permanently removed. Their accounts are kept unless separately included above.", summary.empty_sets));
                }
                c::body(ui, "This cannot be undone here. Filesystem, SSD, memory and old-backup secure erasure is not guaranteed. Existing exported backups are unchanged and may still contain deleted records.");
            }
            ui.add_enabled_ui(summary.parents.is_empty() && (summary.conflicts.is_empty() || confirmation.consent), |ui| {
                let submit_button = c::button(ui, if is_restore { "Restore record" } else { "Permanently delete reviewed records" }, c::ButtonStyle::Primary);
                if submit_button.clicked() { submit = true; }
            });
            keep.clicked() || submit || reopen.is_some() || review_current
        });
        if let Some(id) = reopen {
            // A parent review may replace this dialog; return focus to the
            // persistent Back control, never a disappeared dialog-only opener.
            if let Some(opener) = self.back_opener {
                return Action::Prepare(Operation::Restore(id), opener);
            } else {
                self.error = Some("Return to Trash and review the parent again.".into());
                self.focus = true;
            }
            return Action::None;
        }
        if submit {
            let action = Action::Save(confirmation.review.clone(), confirmation.consent);
            self.review = Some(confirmation);
            return action;
        }
        if review_current {
            self.error = None;
            self.focus = true;
        }
        if !close {
            self.review = Some(confirmation);
        }
        Action::None
    }
}

fn item_button(
    ui: &mut egui::Ui,
    text: &str,
    action: &str,
    id: ItemId,
    style: c::ButtonStyle,
) -> egui::Response {
    let response = c::button(ui, text, style);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::Role::Button,
            ui.is_enabled(),
            format!("{action} {} · ID: {}", id.kind(), id.uuid()),
        )
    });
    response
}
fn identity(ui: &mut egui::Ui, vault: &Vault, id: ItemId) {
    let account_id = match id {
        ItemId::Account(id) => Some(id),
        ItemId::Folder(_) => None,
        ItemId::Membership(id) => vault
            .memberships()
            .iter()
            .find(|m| m.id() == id)
            .map(Membership::account_id),
        ItemId::Set(id) | ItemId::Code(id, _) => vault
            .backup_sets()
            .iter()
            .find(|s| s.id() == id)
            .map(BackupCodeSet::account_id),
    };
    let displayed_account =
        account_id.and_then(|id| vault.accounts().iter().find(|a| a.id() == id));
    if let Some(account) = displayed_account {
        backup_codes::identity(ui, account, true);
    }
    match id {
        ItemId::Folder(id) => {
            if let Some(f) = vault.folders().iter().find(|f| f.id() == id) {
                c::heading(ui, &format!("Folder: {:?}", f.name()), 2);
            }
        }
        ItemId::Membership(id) => {
            if let Some(f) = vault
                .memberships()
                .iter()
                .find(|m| m.id() == id)
                .and_then(|m| vault.folders().iter().find(|f| f.id() == m.folder_id()))
            {
                c::body(
                    ui,
                    &format!("Folder: {:?} · ID: {}", f.name(), f.id().as_uuid()),
                );
            }
        }
        ItemId::Code(set, code) => {
            if let Some(code) = vault
                .backup_sets()
                .iter()
                .find(|s| s.id() == set)
                .and_then(|s| s.codes().iter().find(|c| c.id() == code))
            {
                c::body(
                    ui,
                    &code.used_at().map_or_else(
                        || "Available before deletion".into(),
                        |at| format!("Used since {}", date(at)),
                    ),
                );
            }
        }
        _ => {}
    }
    if !matches!(id, ItemId::Account(_)) || displayed_account.is_none() {
        c::secondary(ui, &format!("{} · ID: {}", id.kind(), id.uuid()));
    }
}
fn counts(c: Counts) -> String {
    format!(
        "{} account(s), {} folder(s), {} membership(s), {} backup-code set(s), {} code(s)",
        c.accounts, c.folders, c.memberships, c.sets, c.codes
    )
}
fn pages(ui: &mut egui::Ui, page: &mut usize, total: usize, scope: &str) {
    let count = total.div_ceil(backup_codes::PAGE_SIZE).max(1);
    *page = (*page).min(count - 1);
    ui.push_id(scope, |ui| {
        c::secondary(
            ui,
            &format!("Page {} of {count} · {total} item(s)", *page + 1),
        );
        ui.columns(2, |columns| {
            columns[0].add_enabled_ui(*page > 0, |ui| {
                if c::button(ui, "Previous", c::ButtonStyle::Secondary).clicked() {
                    *page -= 1;
                }
            });
            columns[1].add_enabled_ui(*page + 1 < count, |ui| {
                if c::button(ui, "Next", c::ButtonStyle::Secondary).clicked() {
                    *page += 1;
                }
            });
        });
    });
}
/// English UTC calendar display. Bound the year loop; extreme imported logical
/// times retain their exact Unix-second representation instead of overflowing.
fn date(at: Timestamp) -> String {
    if at.0 > 253_402_300_799 {
        return format!("Unix second {} (UTC)", at.0);
    }
    let mut days = at.0 / 86_400;
    let mut year = 1970u64;
    let leap = |year: u64| {
        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
    };
    while days >= 365 + u64::from(leap(year)) {
        days -= 365 + u64::from(leap(year));
        year += 1;
    }
    let months = [
        31,
        28 + u64::from(leap(year)),
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 0;
    while days >= months[month] {
        days -= months[month];
        month += 1;
    }
    let seconds = at.0 % 86_400;
    format!(
        "{year:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        month + 1,
        days + 1,
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trash_deletion_dates_are_utc_and_extreme_logical_times_remain_exact() {
        for (at, expected) in [
            (0, "1970-01-01 00:00:00 UTC"),
            (951_782_400, "2000-02-29 00:00:00 UTC"),
            (4_107_542_400, "2100-03-01 00:00:00 UTC"),
            (253_402_300_799, "9999-12-31 23:59:59 UTC"),
        ] {
            assert_eq!(date(Timestamp(at)), expected);
        }
        assert_eq!(
            date(Timestamp(u64::MAX)),
            "Unix second 18446744073709551615 (UTC)"
        );
    }
}
