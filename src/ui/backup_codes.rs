//! Concealed recovery metadata and ID-only navigation. Never reads code values.
use super::{codes, components as c, theme};
use crate::domain::records::*;
use eframe::egui;
use std::collections::{BTreeMap, BTreeSet};

// Bounded rendering keeps up to 10,000 accounts usable without building 10,000
// widget trees. Pagination has explicit, stable Previous / Next identities.
pub const PAGE_SIZE: usize = 20;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Screen {
    #[default]
    List,
    Select,
    Editor(AccountId),
    Detail {
        account: AccountId,
        set: BackupSetId,
        from_selection: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetMetadata {
    pub id: BackupSetId,
    pub trashed: bool,
    pub available: usize,
    pub used: usize,
}
impl SetMetadata {
    fn from_set(set: &BackupCodeSet) -> Self {
        let mut metadata = Self {
            id: set.id(),
            trashed: set.state().deletion != DeletionState::Active,
            available: 0,
            used: 0,
        };
        if !metadata.trashed {
            for code in set
                .codes()
                .iter()
                .filter(|c| c.state().deletion == DeletionState::Active)
            {
                if code.used_at().is_some() {
                    metadata.used += 1;
                } else {
                    metadata.available += 1;
                }
            }
        }
        metadata
    }
    pub fn label(self) -> String {
        if self.trashed {
            "Existing backup-code set is in Trash".into()
        } else {
            format!("{} available · {} used", self.available, self.used)
        }
    }
}

/// Only IDs and derived counts are retained; all display names stay in Vault.
pub struct Projection {
    pub accounts: Vec<AccountId>,
    pub sets: BTreeMap<AccountId, SetMetadata>,
    ambiguous: BTreeSet<AccountId>,
    active_accounts: usize,
    active_sets: usize,
}
impl Projection {
    pub fn new(vault: &Vault, query: &str, sort: SortOrder) -> Self {
        let active: BTreeSet<_> = vault
            .accounts()
            .iter()
            .filter(|a| a.state().deletion == DeletionState::Active)
            .map(Account::id)
            .collect();
        let sets: BTreeMap<_, _> = vault
            .backup_sets()
            .iter()
            .filter(|s| active.contains(&s.account_id()))
            .map(|s| {
                // A trashed set has no currently available codes. Keep its ID
                // so selection cannot accidentally offer a conflicting new set.
                (s.account_id(), SetMetadata::from_set(s))
            })
            .collect();
        let mut names: BTreeMap<(&str, &str), Vec<AccountId>> = BTreeMap::new();
        for account in vault.accounts().iter().filter(|a| active.contains(&a.id())) {
            names
                .entry(display_identity(account))
                .or_default()
                .push(account.id());
        }
        let ambiguous = names
            .into_values()
            .filter(|ids| ids.len() > 1)
            .flatten()
            .collect();
        Self {
            accounts: codes::ordered_accounts(vault, query, sort),
            active_accounts: active.len(),
            active_sets: sets.values().filter(|s| !s.trashed).count(),
            sets,
            ambiguous,
        }
    }
    pub fn visible_sets(&self) -> Vec<AccountId> {
        self.accounts
            .iter()
            .copied()
            .filter(|id| self.sets.get(id).is_some_and(|s| !s.trashed))
            .collect()
    }
}

#[derive(Default)]
pub struct Session {
    pub screen: Screen,
    pub selected: Option<AccountId>,
    pub sort: SortOrder,
    pub page: usize,
    pub focus: bool,
    projection: Option<Projection>,
}
impl Session {
    pub fn invalidate(&mut self) {
        self.projection = None;
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn select_accounts(&mut self) {
        self.screen = Screen::Select;
        self.selected = None;
        self.page = 0;
        self.focus = true;
        self.invalidate();
    }
    pub fn continue_to(&mut self, vault: &Vault) -> bool {
        let Some(id) = self.selected else {
            return false;
        };
        if !vault
            .accounts()
            .iter()
            .any(|a| a.id() == id && a.state().deletion == DeletionState::Active)
        {
            return false;
        }
        self.screen = if let Some(set) = vault.backup_sets().iter().find(|s| s.account_id() == id) {
            Screen::Detail {
                account: id,
                set: set.id(),
                from_selection: true,
            }
        } else {
            Screen::Editor(id)
        };
        self.focus = true;
        true
    }
    pub fn back(&mut self) {
        self.screen = match self.screen {
            Screen::Editor(_)
            | Screen::Detail {
                from_selection: true,
                ..
            } => Screen::Select,
            _ => Screen::List,
        };
        self.focus = true;
    }
    fn validate_route(&mut self, vault: &Vault) {
        let active = |id| {
            vault
                .accounts()
                .iter()
                .any(|a| a.id() == id && a.state().deletion == DeletionState::Active)
        };
        if self.selected.is_some_and(|id| !active(id)) {
            self.selected = None;
        }
        let valid = match self.screen {
            Screen::Editor(id) => {
                active(id) && !vault.backup_sets().iter().any(|s| s.account_id() == id)
            }
            Screen::Detail { account, set, .. } => {
                active(account)
                    && vault
                        .backup_sets()
                        .iter()
                        .any(|s| s.id() == set && s.account_id() == account)
            }
            _ => true,
        };
        if !valid {
            self.screen = Screen::List;
            self.focus = true;
            self.invalidate();
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Action {
    None,
    Help(egui::Id),
    AddAccount,
    Edit(AccountId),
    Delete(AccountId),
}

fn display_identity(account: &Account) -> (&str, &str) {
    (
        account.credential().service(),
        account
            .credential()
            .account_identifier()
            .unwrap_or("No account identifier"),
    )
}
fn identity(ui: &mut egui::Ui, account: &Account, ambiguous: bool) {
    let (service, identifier) = display_identity(account);
    c::heading(ui, service, 2);
    c::body(ui, identifier);
    if let Some(website) = account.credential().website() {
        c::secondary(ui, website);
    }
    if ambiguous {
        c::secondary(ui, &format!("Account ID: {}", account.id().as_uuid()));
    }
}

pub(super) fn selected_identity(ui: &mut egui::Ui, vault: &Vault, id: AccountId) {
    if let Some(account) = vault.accounts().iter().find(|a| a.id() == id) {
        let ambiguous = vault
            .accounts()
            .iter()
            .filter(|a| {
                a.state().deletion == DeletionState::Active
                    && display_identity(a) == display_identity(account)
            })
            .count()
            > 1;
        identity(ui, account, ambiguous);
    }
}

fn page_controls(ui: &mut egui::Ui, page: &mut usize, total: usize, scope: &str) {
    let pages = total.div_ceil(PAGE_SIZE).max(1);
    *page = (*page).min(pages - 1);
    ui.push_id(scope, |ui| {
        c::secondary(
            ui,
            &format!(
                "Page {} of {} · {} account{}",
                *page + 1,
                pages,
                total,
                if total == 1 { "" } else { "s" }
            ),
        );
        ui.columns(2, |columns| {
            columns[0].push_id("previous", |ui| {
                ui.add_enabled_ui(*page > 0, |ui| {
                    if c::button(ui, "Previous", c::ButtonStyle::Secondary).clicked() {
                        *page -= 1;
                    }
                });
            });
            columns[1].push_id("next", |ui| {
                ui.add_enabled_ui(*page + 1 < pages, |ui| {
                    if c::button(ui, "Next", c::ButtonStyle::Secondary).clicked() {
                        *page += 1;
                    }
                });
            });
        });
    });
}

pub fn content(
    ui: &mut egui::Ui,
    vault: &Vault,
    session: &mut Session,
    query: &mut String,
) -> Action {
    session.validate_route(vault);
    let mut action = Action::None;
    let selecting = session.screen == Screen::Select;
    if session.screen != Screen::List {
        let back = ui
            .push_id("backup-back", |ui| {
                c::button(
                    ui,
                    if selecting {
                        "Back to Backup Codes"
                    } else {
                        "Back"
                    },
                    c::ButtonStyle::Secondary,
                )
            })
            .inner;
        if back.clicked() {
            session.back();
            return action;
        }
    }
    match session.screen {
        Screen::List => {
            c::body(
                ui,
                "Keep service-issued recovery codes with the account they belong to. These codes can help when your usual verification method is unavailable.",
            );
            let add = ui
                .push_id("backup-add", |ui| {
                    c::button(ui, "Add Backup Codes", c::ButtonStyle::Primary)
                })
                .inner;
            if add.clicked() {
                query.clear();
                session.select_accounts();
                return action;
            }
            let help = ui
                .push_id("backup-learn", |ui| {
                    c::button(ui, "Learn about backup codes", c::ButtonStyle::Secondary)
                })
                .inner;
            if help.clicked() {
                action = Action::Help(help.id);
            }
        }
        Screen::Select => {
            c::heading(ui, "Select an account", 2);
            c::body(
                ui,
                "Choose the service account these backup codes belong to. Accounts in Trash are hidden.",
            );
        }
        Screen::Editor(id) | Screen::Detail { account: id, .. } => {
            // Typed boundaries for Tasks15/16. Values never enter this page or AX.
            let Some(account) = vault.accounts().iter().find(|a| a.id() == id) else {
                return action;
            };
            let duplicate = vault
                .accounts()
                .iter()
                .filter(|a| {
                    a.state().deletion == DeletionState::Active
                        && display_identity(a) == display_identity(account)
                })
                .count()
                > 1;
            c::card(ui, |ui| {
                c::icon_tile(ui, c::Icon::Recovery);
                c::heading(
                    ui,
                    if matches!(session.screen, Screen::Editor(_)) {
                        "Add Backup Codes"
                    } else {
                        "Backup-code set"
                    },
                    2,
                );
                identity(ui, account, duplicate);
                if let Screen::Detail { set, .. } = session.screen {
                    if let Some(metadata) = vault
                        .backup_sets()
                        .iter()
                        .find(|s| s.id() == set && s.account_id() == id)
                        .map(SetMetadata::from_set)
                    {
                        c::body(ui, &metadata.label());
                        c::secondary(
                            ui,
                            if metadata.trashed {
                                "This account already has a set in Trash. It must be restored or permanently removed before adding another set."
                            } else {
                                "Code values are concealed. Revealing, copying and marking used are not available in this build yet. Edit or delete this set below."
                            },
                        );
                        if !metadata.trashed {
                            if c::button(ui, "Edit backup codes", c::ButtonStyle::Primary).clicked()
                            {
                                action = Action::Edit(id);
                            }
                            let delete =
                                c::button(ui, "Delete backup-code set…", c::ButtonStyle::Secondary);
                            if delete.clicked() {
                                action = Action::Delete(id);
                            }
                        }
                    }
                } else {
                    c::body(ui, "No backup-code set is stored for this account.");
                    action = Action::Edit(id);
                }
            });
            c::body(
                ui,
                "Get backup codes from this service's security settings. Authenticator does not generate them or check whether the service has accepted a code.",
            );
            let done = ui
                .push_id("backup-handoff-done", |ui| {
                    c::button(ui, "Back to Backup Codes", c::ButtonStyle::Primary)
                })
                .inner;
            if session.focus {
                done.request_focus();
                session.focus = false;
            }
            if done.clicked() {
                session.screen = Screen::List;
                session.focus = true;
            }
            return action;
        }
    }
    let invalid_query =
        query.len() > CodesPreferences::MAX_QUERY_BYTES || query.chars().any(char::is_control);
    let search = c::labeled_field(
        ui,
        "backup-search",
        if selecting {
            "Search accounts"
        } else {
            "Search backup codes"
        },
        query,
        "Search by service or account identifier.",
        invalid_query.then_some("Use at most 1024 UTF-8 bytes without control characters."),
    );
    if session.focus {
        search.request_focus();
        session.focus = false;
    }
    let mut changed = search.changed();
    let mut label = codes::sort_label(session.sort).to_owned();
    if c::labeled_choice(
        ui,
        "backup-sort",
        "Sort accounts",
        &mut label,
        &codes::SORT_LABELS,
    ) && let Some(index) = codes::SORT_LABELS.iter().position(|s| *s == label)
    {
        session.sort = codes::SORTS[index];
        changed = true;
    }
    // Always allocate the feedback region and action's explicit ID, so changing
    // error/no-match state cannot move focus to a different control.
    let clear = ui
        .push_id("backup-clear", |ui| {
            c::button(ui, "Clear search", c::ButtonStyle::Secondary)
        })
        .inner;
    if clear.clicked() {
        query.clear();
        session.focus = true;
        changed = true;
    }
    if changed {
        session.invalidate();
        session.page = 0;
        session.selected = None;
    }
    if query.len() > CodesPreferences::MAX_QUERY_BYTES || query.chars().any(char::is_control) {
        return action;
    }
    if session.projection.is_none() {
        session.projection = Some(Projection::new(vault, query, session.sort));
    }
    let Some(projection) = &session.projection else {
        return action;
    };
    if selecting
        && let Some(id) = session.selected
        && let Some(account) = vault.accounts().iter().find(|a| a.id() == id)
    {
        let mut proceed = false;
        ui.push_id("backup-selected-summary", |ui| {
            c::card(ui, |ui| {
                c::heading(ui, "Selected account", 2);
                identity(ui, account, projection.ambiguous.contains(&id));
                c::body(
                    ui,
                    &projection
                        .sets
                        .get(&id)
                        .map(|s| s.label())
                        .unwrap_or_else(|| "No backup-code set yet".into()),
                );
                c::secondary(
                    ui,
                    if projection.sets.contains_key(&id) {
                        "Continue opens the existing set. It does not replace stored codes."
                    } else {
                        "Continue opens this account's backup-code entry screen."
                    },
                );
                proceed = c::button(ui, "Continue", c::ButtonStyle::Primary).clicked();
            })
        });
        if proceed {
            session.continue_to(vault);
            return action;
        }
    }
    let ids = if selecting {
        projection.accounts.clone()
    } else {
        projection.visible_sets()
    };
    if ids.is_empty() {
        let empty = if selecting {
            projection.active_accounts == 0
        } else {
            projection.active_sets == 0
        };
        c::empty_state(
            ui,
            c::Icon::Recovery,
            if empty {
                if selecting {
                    "No accounts available"
                } else {
                    "No backup codes yet"
                }
            } else {
                "No matching accounts"
            },
            if empty {
                if selecting {
                    "Add a 2FA account first, then keep its service-issued backup codes here."
                } else {
                    "Add service-issued codes to keep a concealed, encrypted recovery record."
                }
            } else {
                "Try another service or account identifier, or clear the search."
            },
        );
        if selecting
            && empty
            && c::button(ui, "Add a 2FA account", c::ButtonStyle::Primary).clicked()
        {
            action = Action::AddAccount;
        }
        return action;
    }
    page_controls(
        ui,
        &mut session.page,
        ids.len(),
        if selecting {
            "backup-accounts-page"
        } else {
            "backup-sets-page"
        },
    );
    for id in ids
        .into_iter()
        .skip(session.page * PAGE_SIZE)
        .take(PAGE_SIZE)
    {
        let Some(account) = vault.accounts().iter().find(|a| a.id() == id) else {
            continue;
        };
        ui.push_id(("backup-account", id), |ui| {
            c::card(ui, |ui| {
                c::icon_tile(ui, c::Icon::Recovery);
                identity(ui, account, projection.ambiguous.contains(&id));
                let metadata = projection.sets.get(&id);
                c::body(
                    ui,
                    &metadata
                        .map(|s| s.label())
                        .unwrap_or_else(|| "No backup-code set yet".into()),
                );
                if selecting {
                    let selected = session.selected == Some(id);
                    let response = c::button(
                        ui,
                        if selected {
                            "Selected account"
                        } else {
                            "Select account"
                        },
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
                            "Select account",
                        )
                    });
                    if response.clicked() {
                        session.selected = Some(id);
                    }
                } else if c::button(ui, "View set", c::ButtonStyle::Secondary).clicked()
                    && let Some(metadata) = metadata
                {
                    session.screen = Screen::Detail {
                        account: id,
                        set: metadata.id,
                        from_selection: false,
                    };
                    session.focus = true;
                }
            })
        });
        ui.add_space(theme::SMALL_GAP);
    }
    action
}
