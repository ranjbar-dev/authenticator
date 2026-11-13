//! Metadata-only review guards and reversible folder transactions.
use super::{records::*, validation};
use crate::error::AppError;
use std::collections::{BTreeMap, BTreeSet};
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone, Eq, PartialEq)]
struct AccountMetadata {
    id: AccountId,
    state: RecordState,
    service: Zeroizing<String>,
    identifier: Option<Zeroizing<String>>,
    website: Option<Zeroizing<String>>,
}
impl AccountMetadata {
    fn of(a: &Account) -> Self {
        Self {
            id: a.id(),
            state: a.state(),
            service: Zeroizing::new(a.credential().service().to_owned()),
            identifier: a
                .credential()
                .account_identifier()
                .map(|s| Zeroizing::new(s.to_owned())),
            website: a
                .credential()
                .website()
                .map(|s| Zeroizing::new(s.to_owned())),
        }
    }
}

/// Contains no seeds, code values, keys or duplicate mutable vault.
#[derive(Clone)]
pub struct FolderTarget {
    folder: Option<Folder>,
    links: Vec<Membership>,
    accounts: Vec<AccountMetadata>,
    names: Vec<(FolderId, RecordState, Zeroizing<String>)>,
}
impl FolderTarget {
    pub fn capture(vault: &Vault, id: Option<FolderId>) -> Result<Self, AppError> {
        let folder = id
            .map(|id| {
                vault
                    .folders()
                    .iter()
                    .find(|f| f.id() == id && f.state().deletion == DeletionState::Active)
                    .cloned()
                    .ok_or(AppError::StaleFolder)
            })
            .transpose()?;
        Ok(Self {
            links: vault
                .memberships()
                .iter()
                .filter(|m| Some(m.folder_id()) == id)
                .cloned()
                .collect(),
            folder,
            accounts: vault.accounts().iter().map(AccountMetadata::of).collect(),
            names: vault
                .folders()
                .iter()
                .map(|f| (f.id(), f.state(), Zeroizing::new(f.name().to_owned())))
                .collect(),
        })
    }
    pub fn folder(&self) -> Option<&Folder> {
        self.folder.as_ref()
    }
    pub fn selected(&self) -> BTreeSet<AccountId> {
        let active: BTreeSet<_> = self
            .accounts
            .iter()
            .filter(|a| a.state.deletion == DeletionState::Active)
            .map(|a| a.id)
            .collect();
        self.links
            .iter()
            .filter(|m| {
                m.state().deletion == DeletionState::Active && active.contains(&m.account_id())
            })
            .map(Membership::account_id)
            .collect()
    }
    pub fn hidden_links(&self) -> usize {
        let hidden: BTreeSet<_> = self
            .accounts
            .iter()
            .filter(|a| a.state.deletion != DeletionState::Active)
            .map(|a| a.id)
            .collect();
        self.links
            .iter()
            .filter(|m| hidden.contains(&m.account_id()))
            .count()
    }
    pub fn check(&self, vault: &Vault) -> Result<(), AppError> {
        // Content binding matters when logical time is equal after clock correction.
        if self.accounts.len() != vault.accounts().len()
            || !self
                .accounts
                .iter()
                .zip(vault.accounts())
                .all(|(old, a)| *old == AccountMetadata::of(a))
            || self.names.len() != vault.folders().len()
            || !self
                .names
                .iter()
                .zip(vault.folders())
                .all(|((id, state, name), f)| {
                    *id == f.id() && *state == f.state() && name.as_str() == f.name()
                })
        {
            return Err(AppError::StaleFolder);
        }
        if let Some(old) = &self.folder {
            let current = vault
                .folders()
                .iter()
                .find(|f| f.id() == old.id())
                .ok_or(AppError::StaleFolder)?;
            if current.state() != old.state()
                || current.name() != old.name()
                || current.color() != old.color()
                || current.state().deletion != DeletionState::Active
            {
                return Err(AppError::StaleFolder);
            }
            let links: Vec<_> = vault
                .memberships()
                .iter()
                .filter(|m| m.folder_id() == old.id())
                .collect();
            if links.len() != self.links.len()
                || !links.iter().zip(&self.links).all(|(m, old)| {
                    m.id() == old.id()
                        && m.state() == old.state()
                        && m.folder_id() == old.folder_id()
                        && m.account_id() == old.account_id()
                })
            {
                return Err(AppError::StaleFolder);
            }
        }
        Ok(())
    }
}

pub fn normalized_name(name: &str) -> Result<String, AppError> {
    validation::text(name, 256, AppError::InvalidFolderName)
}
/// Import identity stays ID-based. Exact trimmed names only produce a warning;
/// case variants are distinct, and Trash participates in the same warning.
pub fn duplicate_names(vault: &Vault, name: &str, editing: Option<FolderId>) -> Vec<FolderId> {
    vault
        .folders()
        .iter()
        .filter(|f| Some(f.id()) != editing && f.name() == name.trim())
        .map(Folder::id)
        .collect()
}

impl Vault {
    pub fn save_folder(
        &mut self,
        target: &FolderTarget,
        changes: FolderUpdate<'_>,
        at: Timestamp,
    ) -> Result<FolderId, AppError> {
        let FolderUpdate {
            name,
            color,
            selected,
            duplicate_consent,
        } = changes;
        target.check(self)?;
        let name = normalized_name(name)?;
        let editing = target.folder().map(Folder::id);
        if !duplicate_consent && !duplicate_names(self, &name, editing).is_empty() {
            return Err(AppError::ReviewFolderDuplicates);
        }
        let active: BTreeSet<_> = self
            .accounts()
            .iter()
            .filter(|a| a.state().deletion == DeletionState::Active)
            .map(Account::id)
            .collect();
        if !selected.is_subset(&active) {
            return Err(AppError::StaleFolder);
        }
        let existing: BTreeMap<_, _> = self
            .memberships()
            .iter()
            .filter(|m| Some(m.folder_id()) == editing)
            .map(|m| (m.account_id(), m.id()))
            .collect();
        let added = selected
            .iter()
            .filter(|id| !existing.contains_key(id))
            .count();
        if (editing.is_none() && self.folders.len() >= Self::MAX_RECORDS)
            || added > Self::MAX_RECORDS.saturating_sub(self.memberships.len())
        {
            return Err(AppError::FolderLimit);
        }
        // Every fallible check precedes mutation, including capacity and hidden links.
        let at = self.mutation_time(at);
        let id = if let Some(id) = editing {
            let folder = self
                .folders
                .iter_mut()
                .find(|f| f.id == id)
                .ok_or(AppError::StaleFolder)?;
            folder.name.zeroize();
            folder.name = name;
            folder.color = color;
            folder.state.updated_at = at;
            id
        } else {
            let folder = Folder::new(&name, color, at)?;
            let id = folder.id();
            self.folders.push(folder);
            id
        };
        for m in &mut self.memberships {
            if m.folder_id != id || !active.contains(&m.account_id) {
                continue;
            }
            let deletion = if selected.contains(&m.account_id) {
                DeletionState::Active
            } else if m.state.deletion == DeletionState::Active {
                DeletionState::Trashed { at }
            } else {
                m.state.deletion
            };
            if deletion != m.state.deletion {
                m.state.deletion = deletion;
                m.state.updated_at = at;
            }
        }
        for account_id in selected.iter().filter(|id| !existing.contains_key(id)) {
            self.memberships.push(Membership {
                id: MembershipId::new(),
                state: RecordState::new(at),
                folder_id: id,
                account_id: *account_id,
            });
        }
        self.state.updated_at = at;
        Ok(id)
    }
    /// Links retain their own states. Parent visibility is inherited, allowing
    /// task 18 to restore surviving relationships without recreating identities.
    pub fn trash_folder(&mut self, target: &FolderTarget, at: Timestamp) -> Result<(), AppError> {
        target.check(self)?;
        let id = target.folder().ok_or(AppError::StaleFolder)?.id();
        let at = self.mutation_time(at);
        let folder = self
            .folders
            .iter_mut()
            .find(|f| f.id == id)
            .ok_or(AppError::StaleFolder)?;
        folder.state.updated_at = at;
        folder.state.deletion = DeletionState::Trashed { at };
        if self.preferences.folder == FolderFilter::Folder(id) {
            self.preferences.folder = FolderFilter::All;
            self.preferences.state.updated_at = at;
        }
        self.state.updated_at = at;
        Ok(())
    }
}

pub struct FolderUpdate<'a> {
    pub name: &'a str,
    pub color: FolderColor,
    pub selected: &'a BTreeSet<AccountId>,
    pub duplicate_consent: bool,
}
