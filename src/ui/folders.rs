//! Folder screens contain metadata only; the sole repository worker commits changes.
use super::{backup_codes, codes, components as c, state::SaveIntent, theme};
use crate::{
    domain::{
        folder_edit::{self, FolderTarget},
        records::*,
    },
    error::AppError,
};
use eframe::egui;
use std::collections::{BTreeMap, BTreeSet};
use zeroize::Zeroizing;

pub fn label(vault: &Vault, folder: &Folder) -> String {
    // Escape the name inside quotes so it cannot imitate an identity suffix.
    let ambiguous = matches!(folder.name(), "All" | "All accounts" | "Unfiled")
        || vault
            .folders()
            .iter()
            .filter(|f| f.name() == folder.name())
            .count()
            > 1;
    if ambiguous {
        format!(
            "Folder: {:?} · ID: {}",
            folder.name(),
            folder.id().as_uuid()
        )
    } else {
        format!("Folder: {:?}", folder.name())
    }
}
/// Build the full selector once, rather than rescanning 10,000 names per option.
pub fn labels(vault: &Vault) -> Vec<(FolderId, String)> {
    let mut counts = BTreeMap::new();
    for folder in vault.folders() {
        *counts.entry(folder.name()).or_insert(0usize) += 1;
    }
    let mut result: Vec<_> = vault
        .folders()
        .iter()
        .filter(|f| f.state().deletion == DeletionState::Active)
        .map(|f| {
            let ambiguous = matches!(f.name(), "All" | "All accounts" | "Unfiled")
                || counts.get(f.name()).is_some_and(|n| *n > 1);
            (
                f.id(),
                if ambiguous {
                    format!("Folder: {:?} · ID: {}", f.name(), f.id().as_uuid())
                } else {
                    format!("Folder: {:?}", f.name())
                },
            )
        })
        .collect();
    result.sort_by(|(a, an), (b, bn)| an.cmp(bn).then(a.cmp(b)));
    result
}

#[derive(Default)]
pub struct Manager {
    pub editor: Option<Editor>,
    pub focus: bool,
    query: Zeroizing<String>,
    page: usize,
    error: Option<String>,
}
pub enum Action {
    None,
    Back,
    Save(bool),
    Lock,
}
impl Manager {
    pub fn new() -> Self {
        Self {
            focus: true,
            ..Self::default()
        }
    }
    pub fn title(&self) -> &'static str {
        self.editor.as_ref().map_or("Folders", |e| {
            if e.target.folder().is_some() {
                "Edit folder"
            } else {
                "Create folder"
            }
        })
    }
    pub fn content(&mut self, ui: &mut egui::Ui, vault: &Vault) -> Action {
        if let Some(editor) = &mut self.editor {
            return editor.content(ui, vault);
        }
        let back = ui
            .push_id("folders-back", |ui| {
                c::button(ui, "Back to 2FA Codes", c::ButtonStyle::Secondary)
            })
            .inner;
        if self.focus {
            back.request_focus();
            back.scroll_to_me(None);
            self.focus = false;
        }
        if back.clicked() {
            return Action::Back;
        }
        c::body(
            ui,
            "Group accounts in more than one folder. All accounts and Unfiled are available in the Codes folder selector.",
        );
        if ui
            .push_id("folders-create", |ui| {
                c::button(ui, "Create folder", c::ButtonStyle::Primary)
            })
            .inner
            .clicked()
        {
            match Editor::new(vault, None) {
                Ok(editor) => {
                    self.editor = Some(editor);
                }
                Err(e) => self.error = Some(e.to_string()),
            }
            return Action::None;
        }
        let error = query_error(&self.query);
        let search = c::labeled_field(
            ui,
            "folders-search",
            "Search folders",
            &mut self.query,
            "Folder names only.",
            error.as_deref(),
        );
        if search.changed() {
            self.page = 0;
        }
        if c::button(ui, "Clear folder search", c::ButtonStyle::Secondary).clicked() {
            self.query.clear();
            self.page = 0;
        }
        ui.push_id("folders-error", |ui| {
            if let Some(error) = &self.error {
                c::status(ui, "error", error, c::StatusKind::Error);
            }
        });
        if query_error(&self.query).is_some() {
            return Action::None;
        }
        let folded = Zeroizing::new(caseless::default_case_fold_str(&self.query));
        let mut folders: Vec<_> = vault
            .folders()
            .iter()
            .filter(|f| f.state().deletion == DeletionState::Active)
            .map(|f| (f, Zeroizing::new(caseless::default_case_fold_str(f.name()))))
            .filter(|(_, name)| name.contains(folded.as_str()))
            .collect();
        folders.sort_by(|(a, an), (b, bn)| an.cmp(bn).then(a.id().cmp(&b.id())));
        if folders.is_empty() {
            c::heading(
                ui,
                if self.query.is_empty() {
                    "No folders yet"
                } else {
                    "No matching folders"
                },
                2,
            );
            c::body(
                ui,
                "Create a folder or clear the search. Accounts remain available in All accounts.",
            );
        } else {
            let active: BTreeSet<_> = vault
                .accounts()
                .iter()
                .filter(|a| a.state().deletion == DeletionState::Active)
                .map(Account::id)
                .collect();
            let mut counts = BTreeMap::new();
            for m in vault.memberships().iter().filter(|m| {
                m.state().deletion == DeletionState::Active && active.contains(&m.account_id())
            }) {
                *counts.entry(m.folder_id()).or_insert(0usize) += 1;
            }
            pages(ui, &mut self.page, folders.len(), "folder-pages");
            for (folder, _) in folders
                .into_iter()
                .skip(self.page * backup_codes::PAGE_SIZE)
                .take(backup_codes::PAGE_SIZE)
            {
                ui.push_id(folder.id(), |ui| {
                    c::card(ui, |ui| {
                        c::heading(ui, &label(vault, folder), 2);
                        c::body(ui, &format!("Color: {}", color_name(folder.color())));
                        let count = counts.get(&folder.id()).copied().unwrap_or(0);
                        c::secondary(ui, &format!("{count} active account(s)"));
                        if c::button(ui, "Edit folder", c::ButtonStyle::Secondary).clicked() {
                            match Editor::new(vault, Some(folder.id())) {
                                Ok(editor) => self.editor = Some(editor),
                                Err(e) => self.error = Some(e.to_string()),
                            }
                        }
                    });
                });
                ui.add_space(theme::SMALL_GAP);
            }
        }
        c::secondary(
            ui,
            "Folders in Trash have no automatic expiry. Open Trash in Settings to restore or permanently delete them.",
        );
        Action::None
    }
    pub fn complete(&mut self, result: Result<(), AppError>, ctx: &egui::Context) {
        if result.is_ok() {
            self.editor = None;
            self.focus = true;
            c::clear_sensitive_widget_memory(ctx);
        } else if let Some(editor) = &mut self.editor {
            editor.saving = false;
            editor.error = Some(format!(
                "{} Your input and previous stored data are kept. Retry Save or cancel.",
                result.err().unwrap_or(AppError::StorageUnavailable)
            ));
            editor.focus = true;
        }
    }
}

enum Confirmation {
    Discard,
    Trash,
}
pub struct Editor {
    pub target: FolderTarget,
    pub name: Zeroizing<String>,
    pub color: FolderColor,
    pub selected: BTreeSet<AccountId>,
    pub duplicate_consent: bool,
    pub saving: bool,
    pub error: Option<String>,
    pub focus: bool,
    query: Zeroizing<String>,
    sort: SortOrder,
    page: usize,
    dialog: Option<(Confirmation, c::Dialog)>,
    cancel_opener: Option<egui::Id>,
}
impl Editor {
    pub fn new(vault: &Vault, id: Option<FolderId>) -> Result<Self, AppError> {
        let target = FolderTarget::capture(vault, id)?;
        Ok(Self {
            name: Zeroizing::new(target.folder().map_or("", Folder::name).to_owned()),
            color: target.folder().map_or(FolderColor::Blue, Folder::color),
            selected: target.selected(),
            target,
            duplicate_consent: false,
            saving: false,
            error: None,
            focus: true,
            query: Zeroizing::new(String::new()),
            sort: SortOrder::ServiceAscending,
            page: 0,
            dialog: None,
            cancel_opener: None,
        })
    }
    pub fn dirty(&self) -> bool {
        self.name.as_str() != self.target.folder().map_or("", Folder::name)
            || self.color
                != self
                    .target
                    .folder()
                    .map_or(FolderColor::Blue, Folder::color)
            || self.selected != self.target.selected()
    }
    pub fn request_cancel(&mut self) {
        self.dialog = Some((
            Confirmation::Discard,
            c::Dialog::new(
                "Discard folder edits?",
                self.cancel_opener
                    .unwrap_or_else(|| egui::Id::unique("folder-back")),
            ),
        ));
    }
    pub fn intent(&self, trash: bool, preferences: CodesPreferences) -> SaveIntent {
        if trash {
            SaveIntent::FolderTrash {
                target: self.target.clone(),
                preferences,
            }
        } else {
            SaveIntent::FolderSave {
                target: self.target.clone(),
                name: self.name.clone(),
                color: self.color,
                selected: self.selected.clone(),
                duplicate_consent: self.duplicate_consent,
                preferences,
            }
        }
    }
    pub fn content(&mut self, ui: &mut egui::Ui, vault: &Vault) -> Action {
        if self.saving {
            c::progress(ui, "Saving encrypted folder changes…", None);
            return if c::button(ui, "Cancel and lock", c::ButtonStyle::Secondary).clicked() {
                Action::Lock
            } else {
                Action::None
            };
        }
        let mut action = Action::None;
        let back = ui
            .push_id("folder-back", |ui| {
                c::button(ui, "Back to folders", c::ButtonStyle::Secondary)
            })
            .inner;
        self.cancel_opener = Some(back.id);
        if back.clicked() {
            if self.dirty() {
                self.request_cancel();
            } else {
                return Action::Back;
            }
        }
        let name_error = folder_edit::normalized_name(&self.name)
            .err()
            .map(|e| e.to_string());
        let name = c::labeled_field(
            ui,
            "folder-name",
            "Folder name",
            &mut self.name,
            "Up to 256 UTF-8 bytes. Surrounding whitespace is trimmed.",
            name_error.as_deref(),
        );
        if self.focus {
            name.request_focus();
            name.scroll_to_me(None);
            self.focus = false;
        }
        if name.changed() {
            self.duplicate_consent = false;
            self.error = None;
        }
        c::body(ui, "Folder color");
        ui.columns(2, |columns| {
            for (i, color) in COLORS.into_iter().enumerate() {
                let ui = &mut columns[i % 2];
                let selected = self.color == color;
                let text = format!(
                    "{}{}",
                    color_name(color),
                    if selected { " · Selected" } else { "" }
                );
                let response = c::button(
                    ui,
                    &text,
                    if selected {
                        c::ButtonStyle::Primary
                    } else {
                        c::ButtonStyle::Secondary
                    },
                );
                response.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::Role::RadioButton,
                        true,
                        selected,
                        format!("Folder color: {}", color_name(color)),
                    )
                });
                // An opaque swatch pairs dark foreground with each light reference-style color.
                let swatch = egui::Rect::from_min_size(
                    response.rect.left_top() + egui::vec2(theme::UNIT, theme::UNIT),
                    egui::vec2(theme::UNIT, response.rect.height() - theme::SMALL_GAP),
                );
                ui.painter()
                    .rect_filled(swatch, theme::CONTROL_RADIUS, color_value(color));
                if response.clicked() {
                    self.color = color;
                }
            }
        });
        let duplicates =
            folder_edit::duplicate_names(vault, &self.name, self.target.folder().map(Folder::id));
        if !duplicates.is_empty() {
            c::body(
                ui,
                &format!(
                    "{} other folder(s) have this exact name, including Trash. They stay separate and show full IDs in navigation.",
                    duplicates.len()
                ),
            );
            let response = ui.add(egui::Checkbox::new(
                &mut self.duplicate_consent,
                theme::BODY.text("Keep separate folders with this duplicate name"),
            ));
            c::focus(ui, &response);
        }
        c::heading(ui, "Select accounts", 2);
        c::body(
            ui,
            &format!(
                "{} selected account(s). An account may belong to several folders.",
                self.selected.len()
            ),
        );
        if self.target.hidden_links() > 0 {
            c::secondary(
                ui,
                &format!(
                    "{} membership(s) for accounts in Trash are hidden and preserved. Restoring an account can make its surviving active memberships visible again.",
                    self.target.hidden_links()
                ),
            );
        }
        c::secondary(
            ui,
            "Clearing an active selection moves only that membership to Trash. Selecting it again restores the same link. Accounts and backup codes stay intact.",
        );
        let search_error = query_error(&self.query);
        let search = c::labeled_field(
            ui,
            "folder-account-search",
            "Search accounts",
            &mut self.query,
            "Search by service or account identifier.",
            search_error.as_deref(),
        );
        let mut changed = search.changed();
        let mut sort = codes::sort_label(self.sort).to_owned();
        if c::labeled_choice(
            ui,
            "folder-account-sort",
            "Sort accounts",
            &mut sort,
            &codes::SORT_LABELS,
        ) && let Some(index) = codes::SORT_LABELS.iter().position(|s| *s == sort)
        {
            self.sort = codes::SORTS[index];
            changed = true;
        }
        if c::button(ui, "Clear account search", c::ButtonStyle::Secondary).clicked() {
            self.query.clear();
            changed = true;
        }
        if changed {
            self.page = 0;
        }
        if query_error(&self.query).is_none() {
            let ids = codes::ordered_accounts(vault, &self.query, self.sort);
            if ids.is_empty() {
                c::body(
                    ui,
                    "No matching active accounts. Clear the search or add accounts from 2FA Codes. Existing selections are kept.",
                );
            } else {
                pages(ui, &mut self.page, ids.len(), "folder-account-pages");
                let accounts: BTreeMap<_, _> =
                    vault.accounts().iter().map(|a| (a.id(), a)).collect();
                let mut identities = BTreeMap::new();
                for a in vault
                    .accounts()
                    .iter()
                    .filter(|a| a.state().deletion == DeletionState::Active)
                {
                    *identities
                        .entry(backup_codes::display_identity(a))
                        .or_insert(0usize) += 1;
                }
                for id in ids
                    .into_iter()
                    .skip(self.page * backup_codes::PAGE_SIZE)
                    .take(backup_codes::PAGE_SIZE)
                {
                    let Some(account) = accounts.get(&id) else {
                        continue;
                    };
                    ui.push_id(("folder-member", id), |ui| {
                        c::card(ui, |ui| {
                            backup_codes::identity(
                                ui,
                                account,
                                identities
                                    .get(&backup_codes::display_identity(account))
                                    .is_some_and(|n| *n > 1),
                            );
                            let mut selected = self.selected.contains(&id);
                            let selection_label = if selected {
                                "Selected"
                            } else {
                                "Select account"
                            };
                            let (service, identifier) = backup_codes::display_identity(account);
                            let response = ui.add_sized(
                                [ui.available_width(), theme::CONTROL_HEIGHT],
                                egui::Checkbox::new(
                                    &mut selected,
                                    theme::BODY.text(selection_label),
                                ),
                            );
                            response.widget_info(|| {
                                egui::WidgetInfo::selected(
                                    egui::Role::CheckBox,
                                    true,
                                    selected,
                                    format!(
                                        "{service} · {identifier} · Account ID: {}",
                                        id.as_uuid()
                                    ),
                                )
                            });
                            c::focus(ui, &response);
                            if response.changed() {
                                if selected {
                                    self.selected.insert(id);
                                } else {
                                    self.selected.remove(&id);
                                }
                            }
                        });
                    });
                    ui.add_space(theme::SMALL_GAP);
                }
            }
        }
        ui.push_id("folder-feedback", |ui| {
            if let Some(error) = &self.error {
                c::status(ui, "error", error, c::StatusKind::Error);
            }
        });
        ui.push_id("folder-save", |ui| {
            ui.add_enabled_ui(
                name_error.is_none() && (duplicates.is_empty() || self.duplicate_consent),
                |ui| {
                    if c::button(ui, "Save folder", c::ButtonStyle::Primary).clicked() {
                        action = Action::Save(false);
                    }
                },
            )
        });
        let cancel = ui
            .push_id("folder-cancel", |ui| {
                c::button(ui, "Cancel", c::ButtonStyle::Secondary)
            })
            .inner;
        if cancel.clicked() {
            if self.dirty() {
                self.cancel_opener = Some(cancel.id);
                self.request_cancel();
            } else {
                action = Action::Back;
            }
        }
        if self.target.folder().is_some() {
            let trash = ui
                .push_id("folder-trash", |ui| {
                    c::button(ui, "Move folder to Trash…", c::ButtonStyle::Secondary)
                })
                .inner;
            if trash.clicked() {
                self.dialog = Some((
                    Confirmation::Trash,
                    c::Dialog::new("Move folder to Trash?", trash.id),
                ));
            }
        }
        if let Some((kind, mut dialog)) = self.dialog.take() {
            let mut consent = false;
            let close = dialog.show(ui.ctx(), |ui, first| {
                c::body(ui, match kind { Confirmation::Discard => "Discard unsaved folder edits? Stored folders and accounts stay unchanged.", Confirmation::Trash => "Move this folder to Trash? All accounts, credential settings, backup codes and used history are preserved. Its links remain encrypted with no automatic expiry. If selected in Codes, the view returns to All accounts. Unsaved folder edits are discarded. Restore this folder from Trash in Settings." });
                let cancel = c::button(ui, "Keep editing", c::ButtonStyle::Secondary);
                if first { cancel.request_focus(); }
                if c::button(ui, match kind { Confirmation::Discard => "Discard edits", Confirmation::Trash => "Move folder to Trash" }, c::ButtonStyle::Primary).clicked() { consent = true; }
                cancel.clicked() || consent
            });
            if consent {
                action = match kind {
                    Confirmation::Discard => Action::Back,
                    Confirmation::Trash => Action::Save(true),
                };
            }
            if !close {
                self.dialog = Some((kind, dialog));
            }
        }
        action
    }
}

fn query_error(query: &str) -> Option<String> {
    (query.len() > CodesPreferences::MAX_QUERY_BYTES || query.chars().any(char::is_control))
        .then(|| "Use at most 1024 UTF-8 bytes without control characters.".to_owned())
}
fn pages(ui: &mut egui::Ui, page: &mut usize, total: usize, scope: &str) {
    let count = total.div_ceil(backup_codes::PAGE_SIZE).max(1);
    *page = (*page).min(count - 1);
    ui.push_id(scope, |ui| {
        c::secondary(
            ui,
            &format!("Page {} of {count} · {total} result(s)", *page + 1),
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
pub const COLORS: [FolderColor; 6] = [
    FolderColor::Blue,
    FolderColor::Purple,
    FolderColor::Pink,
    FolderColor::Red,
    FolderColor::Orange,
    FolderColor::Green,
];
pub fn color_name(color: FolderColor) -> &'static str {
    match color {
        FolderColor::Blue => "Blue",
        FolderColor::Purple => "Purple",
        FolderColor::Pink => "Pink",
        FolderColor::Red => "Red",
        FolderColor::Orange => "Orange",
        FolderColor::Green => "Green",
    }
}
fn color_value(color: FolderColor) -> egui::Color32 {
    theme::FOLDER_COLORS[COLORS.iter().position(|c| *c == color).unwrap_or(0)]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn folder_navigation_keeps_imported_duplicates_and_reserved_names_unambiguous()
    -> Result<(), AppError> {
        let mut vault = Vault::new(Timestamp(100));
        for name in [
            "All",
            "All accounts",
            "Unfiled",
            "Public",
            "Public",
            "public",
        ] {
            vault.add_folder(name, FolderColor::Blue, Timestamp(100))?;
        }
        let choices = labels(&vault);
        assert_eq!(choices.len(), 6);
        assert_eq!(
            choices
                .iter()
                .map(|(_, s)| s)
                .collect::<BTreeSet<_>>()
                .len(),
            6
        );
        for folder in vault.folders() {
            let text = &choices
                .iter()
                .find(|(id, _)| *id == folder.id())
                .ok_or(AppError::InvalidRecord)?
                .1;
            assert!(text.starts_with("Folder: "));
            if folder.name() != "public" {
                assert!(text.contains(&folder.id().as_uuid().to_string()));
            }
            assert_eq!(text, &label(&vault, folder));
        }
        assert_eq!(vault.folders().len(), 6);
        vault.validate()
    }

    #[test]
    fn folder_navigation_name_cannot_imitate_generated_identity() -> Result<(), AppError> {
        let mut vault = Vault::new(Timestamp(100));
        let first = vault.add_folder("Public", FolderColor::Blue, Timestamp(100))?;
        vault.add_folder("Public", FolderColor::Green, Timestamp(100))?;
        let forged_suffix = format!("Public · ID: {}", first.as_uuid());
        for name in [
            forged_suffix.as_str(),
            "Public\" · ID: example",
            "Public\\\" · ID: example",
            "\"Public\"",
            "All accounts",
        ] {
            vault.add_folder(name, FolderColor::Purple, Timestamp(100))?;
        }
        let original_ids: BTreeSet<_> = vault.folders().iter().map(Folder::id).collect();
        let choices = labels(&vault);
        assert_eq!(choices.len(), original_ids.len());
        assert_eq!(
            choices.iter().map(|(id, _)| *id).collect::<BTreeSet<_>>(),
            original_ids
        );
        assert_eq!(
            choices
                .iter()
                .map(|(_, text)| text)
                .collect::<BTreeSet<_>>()
                .len(),
            choices.len(),
            "distinct folder identities must have distinguishable captions"
        );
        for folder in vault.folders() {
            let (_, text) = choices
                .iter()
                .find(|(id, _)| *id == folder.id())
                .ok_or(AppError::InvalidRecord)?;
            assert_eq!(text, &label(&vault, folder));
        }
        assert_eq!(vault.folders()[2].name(), forged_suffix);
        vault.validate()
    }
}
