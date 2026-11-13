//! Explicit, content-bound Trash reviews. No expiry, filesystem or UI work.
use super::records::*;
use crate::error::AppError;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ItemId {
    Account(AccountId),
    Folder(FolderId),
    Membership(MembershipId),
    Set(BackupSetId),
    Code(BackupSetId, BackupCodeId),
}
impl ItemId {
    pub fn kind(self) -> &'static str {
        match self {
            Self::Account(_) => "Account",
            Self::Folder(_) => "Folder",
            Self::Membership(_) => "Membership",
            Self::Set(_) => "Backup-code set",
            Self::Code(_, _) => "Backup code",
        }
    }
    pub fn uuid(self) -> uuid::Uuid {
        match self {
            Self::Account(id) => id.as_uuid(),
            Self::Folder(id) => id.as_uuid(),
            Self::Membership(id) => id.as_uuid(),
            Self::Set(id) => id.as_uuid(),
            Self::Code(_, id) => id.as_uuid(),
        }
    }
    pub fn state(self, vault: &Vault) -> Option<RecordState> {
        match self {
            Self::Account(id) => vault
                .accounts()
                .iter()
                .find(|a| a.id() == id)
                .map(Account::state),
            Self::Folder(id) => vault
                .folders()
                .iter()
                .find(|f| f.id() == id)
                .map(Folder::state),
            Self::Membership(id) => vault
                .memberships()
                .iter()
                .find(|m| m.id() == id)
                .map(Membership::state),
            Self::Set(id) => vault
                .backup_sets()
                .iter()
                .find(|s| s.id() == id)
                .map(BackupCodeSet::state),
            Self::Code(set, id) => vault
                .backup_sets()
                .iter()
                .find(|s| s.id() == set)
                .and_then(|s| s.codes().iter().find(|c| c.id() == id))
                .map(BackupCode::state),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Item {
    pub id: ItemId,
    pub deleted_at: Timestamp,
}

/// Own-Trash records only. Active children hidden by a parent are not separate
/// deletions; their own state is preserved when that parent is restored.
pub fn items(vault: &Vault) -> Vec<Item> {
    let mut result = Vec::new();
    let mut add = |id, state: RecordState| {
        if let DeletionState::Trashed { at } = state.deletion {
            result.push(Item { id, deleted_at: at });
        }
    };
    for a in vault.accounts() {
        add(ItemId::Account(a.id()), a.state());
    }
    for f in vault.folders() {
        add(ItemId::Folder(f.id()), f.state());
    }
    for m in vault.memberships() {
        add(ItemId::Membership(m.id()), m.state());
    }
    for s in vault.backup_sets() {
        add(ItemId::Set(s.id()), s.state());
        for c in s.codes() {
            add(ItemId::Code(s.id(), c.id()), c.state());
        }
    }
    result.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at).then(a.id.cmp(&b.id)));
    result
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    Restore(ItemId),
    Delete(ItemId),
    Empty,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Counts {
    pub accounts: usize,
    pub folders: usize,
    pub memberships: usize,
    pub sets: usize,
    pub codes: usize,
}
impl Counts {
    pub fn total(self) -> usize {
        self.accounts + self.folders + self.memberships + self.sets + self.codes
    }
}

#[derive(Clone, Default)]
pub struct Summary {
    pub affected: Counts,
    /// Included associated records whose own state is active, even when hidden by a
    /// trashed parent. This is not a count of currently visible UI records.
    pub active_associations: usize,
    pub empty_sets: usize,
    pub conflicts: Vec<ItemId>,
    pub parents: Vec<ItemId>,
}

#[derive(Clone)]
pub struct Review {
    operation: Operation,
    binding: Zeroizing<[u8; 32]>,
    summary: Summary,
}
impl std::fmt::Debug for Review {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TrashReview([REDACTED])")
    }
}
impl Review {
    /// Explicit-action work, never called for each rendered row/frame. The
    /// binding streams content without cloning keys, the Vault or code values.
    pub fn new(vault: &Vault, operation: Operation) -> Result<Self, AppError> {
        let selected = match operation {
            Operation::Restore(id) | Operation::Delete(id) => {
                require_trashed(vault, id)?;
                vec![id]
            }
            Operation::Empty => items(vault).into_iter().map(|i| i.id).collect(),
        };
        if selected.is_empty() {
            return Err(AppError::StaleTrash);
        }
        let summary = match operation {
            Operation::Restore(id) => restore_summary(vault, id),
            Operation::Delete(_) | Operation::Empty => {
                let removal = Removal::new(vault, &selected);
                Summary {
                    affected: removal.counts(),
                    active_associations: removal.active_count(vault),
                    empty_sets: removal.empty_sets,
                    ..Default::default()
                }
            }
        };
        Ok(Self {
            operation,
            binding: binding(vault),
            summary,
        })
    }
    pub fn operation(&self) -> Operation {
        self.operation
    }
    pub fn summary(&self) -> &Summary {
        &self.summary
    }
    pub fn check(&self, vault: &Vault) -> Result<(), AppError> {
        if self.binding.as_ref() != binding(vault).as_ref() {
            return Err(AppError::StaleTrash);
        }
        Ok(())
    }
    pub fn apply(
        &self,
        vault: &mut Vault,
        duplicate_consent: bool,
        at: Timestamp,
    ) -> Result<(), AppError> {
        self.check(vault)?;
        if !self.summary.parents.is_empty() {
            return Err(AppError::TrashParentHidden);
        }
        if !self.summary.conflicts.is_empty() && !duplicate_consent {
            return Err(AppError::ReviewTrashConflicts);
        }
        let at = vault.mutation_time(at);
        match self.operation {
            Operation::Restore(id) => restore(vault, id, at)?,
            Operation::Delete(id) => Removal::new(vault, &[id]).apply(vault, at),
            Operation::Empty => {
                let selected: Vec<_> = items(vault).into_iter().map(|i| i.id).collect();
                Removal::new(vault, &selected).apply(vault, at);
            }
        }
        vault.state.updated_at = at;
        Ok(())
    }
}

fn require_trashed(vault: &Vault, id: ItemId) -> Result<(), AppError> {
    if !id
        .state(vault)
        .is_some_and(|s| matches!(s.deletion, DeletionState::Trashed { .. }))
    {
        return Err(AppError::StaleTrash);
    }
    Ok(())
}

fn restore_summary(v: &Vault, id: ItemId) -> Summary {
    let mut s = Summary::default();
    let mut parent = |id: ItemId| {
        if id
            .state(v)
            .is_some_and(|s| s.deletion != DeletionState::Active)
        {
            s.parents.push(id);
        }
    };
    match id {
        ItemId::Account(id) => {
            s.affected.accounts = 1;
            s.affected.memberships = v
                .memberships()
                .iter()
                .filter(|m| m.account_id() == id)
                .count();
            for set in v.backup_sets().iter().filter(|set| set.account_id() == id) {
                s.affected.sets += 1;
                s.affected.codes += set.codes().len();
            }
            if let Some(account) = v.accounts().iter().find(|a| a.id() == id) {
                s.conflicts = v
                    .duplicates(account.credential(), Some(id))
                    .into_iter()
                    .map(|d| ItemId::Account(d.account_id))
                    .collect();
            }
        }
        ItemId::Folder(id) => {
            s.affected.folders = 1;
            s.affected.memberships = v
                .memberships()
                .iter()
                .filter(|m| m.folder_id() == id)
                .count();
            if let Some(folder) = v.folders().iter().find(|f| f.id() == id) {
                s.conflicts = v
                    .folders()
                    .iter()
                    .filter(|f| f.id() != id && f.name() == folder.name())
                    .map(|f| ItemId::Folder(f.id()))
                    .collect();
            }
        }
        ItemId::Membership(id) => {
            s.affected.memberships = 1;
            if let Some(m) = v.memberships().iter().find(|m| m.id() == id) {
                parent(ItemId::Account(m.account_id()));
                parent(ItemId::Folder(m.folder_id()));
            }
        }
        ItemId::Set(id) => {
            if let Some(set) = v.backup_sets().iter().find(|set| set.id() == id) {
                s.affected.sets = 1;
                s.affected.codes = set.codes().len();
                parent(ItemId::Account(set.account_id()));
            }
        }
        ItemId::Code(set, _) => {
            s.affected.codes = 1;
            if let Some(set) = v.backup_sets().iter().find(|s| s.id() == set) {
                parent(ItemId::Account(set.account_id()));
                parent(ItemId::Set(set.id()));
            }
        }
    }
    s
}

fn restore(v: &mut Vault, id: ItemId, at: Timestamp) -> Result<(), AppError> {
    require_trashed(v, id)?;
    let mut accounts = BTreeSet::new();
    let mut folders = BTreeSet::new();
    let mut sets = BTreeSet::new();
    match id {
        ItemId::Account(id) => {
            if let Some(a) = v.accounts.iter_mut().find(|a| a.id == id) {
                a.state.deletion = DeletionState::Active;
            }
            accounts.insert(id);
        }
        ItemId::Folder(id) => {
            if let Some(f) = v.folders.iter_mut().find(|f| f.id == id) {
                f.state.deletion = DeletionState::Active;
            }
            folders.insert(id);
        }
        ItemId::Membership(id) => {
            if let Some(m) = v.memberships.iter_mut().find(|m| m.id == id) {
                m.state.deletion = DeletionState::Active;
                m.state.updated_at = at;
                accounts.insert(m.account_id);
                folders.insert(m.folder_id);
            }
        }
        ItemId::Set(id) => {
            if let Some(s) = v.backup_sets.iter_mut().find(|s| s.id == id) {
                s.state.deletion = DeletionState::Active;
                accounts.insert(s.account_id);
            }
            sets.insert(id);
        }
        ItemId::Code(set, id) => {
            if let Some(s) = v.backup_sets.iter_mut().find(|s| s.id == set) {
                if let Some(c) = s.codes.iter_mut().find(|c| c.id == id) {
                    c.state.deletion = DeletionState::Active;
                    c.state.updated_at = at;
                }
                accounts.insert(s.account_id);
            }
            sets.insert(set);
        }
    }
    touch(v, &accounts, &folders, &sets, at);
    Ok(())
}

#[derive(Default)]
struct Removal {
    accounts: BTreeSet<AccountId>,
    folders: BTreeSet<FolderId>,
    memberships: BTreeSet<MembershipId>,
    sets: BTreeSet<BackupSetId>,
    codes: BTreeSet<BackupCodeId>,
    empty_sets: usize,
}
impl Removal {
    fn new(v: &Vault, selected: &[ItemId]) -> Self {
        let mut r = Self::default();
        for id in selected {
            match *id {
                ItemId::Account(id) => {
                    r.accounts.insert(id);
                }
                ItemId::Folder(id) => {
                    r.folders.insert(id);
                }
                ItemId::Membership(id) => {
                    r.memberships.insert(id);
                }
                ItemId::Set(id) => {
                    r.sets.insert(id);
                }
                ItemId::Code(_, id) => {
                    r.codes.insert(id);
                }
            }
        }
        for m in v.memberships() {
            if r.accounts.contains(&m.account_id()) || r.folders.contains(&m.folder_id()) {
                r.memberships.insert(m.id());
            }
        }
        for s in v.backup_sets() {
            if r.accounts.contains(&s.account_id()) {
                r.sets.insert(s.id());
            }
            if !r.sets.contains(&s.id()) && s.codes().iter().all(|c| r.codes.contains(&c.id())) {
                r.empty_sets += 1;
                r.sets.insert(s.id());
            }
            if r.sets.contains(&s.id()) {
                r.codes.extend(s.codes().iter().map(BackupCode::id));
            }
        }
        r
    }
    fn counts(&self) -> Counts {
        Counts {
            accounts: self.accounts.len(),
            folders: self.folders.len(),
            memberships: self.memberships.len(),
            sets: self.sets.len(),
            codes: self.codes.len(),
        }
    }
    fn active_count(&self, v: &Vault) -> usize {
        let active = |state: RecordState| state.deletion == DeletionState::Active;
        v.memberships()
            .iter()
            .filter(|m| self.memberships.contains(&m.id()) && active(m.state()))
            .count()
            + v.backup_sets()
                .iter()
                .filter(|s| self.sets.contains(&s.id()) && active(s.state()))
                .count()
            + v.backup_sets()
                .iter()
                .flat_map(|s| s.codes())
                .filter(|c| self.codes.contains(&c.id()) && active(c.state()))
                .count()
    }
    fn apply(self, v: &mut Vault, at: Timestamp) {
        let mut accounts = BTreeSet::new();
        let mut folders = BTreeSet::new();
        let mut sets = BTreeSet::new();
        for m in v
            .memberships()
            .iter()
            .filter(|m| self.memberships.contains(&m.id()))
        {
            accounts.insert(m.account_id());
            folders.insert(m.folder_id());
        }
        for s in v.backup_sets() {
            if self.sets.contains(&s.id()) || s.codes().iter().any(|c| self.codes.contains(&c.id()))
            {
                accounts.insert(s.account_id());
                sets.insert(s.id());
            }
        }
        v.accounts.retain(|a| !self.accounts.contains(&a.id()));
        v.folders.retain(|f| !self.folders.contains(&f.id()));
        v.memberships
            .retain(|m| !self.memberships.contains(&m.id()));
        v.backup_sets.retain(|s| !self.sets.contains(&s.id()));
        for s in &mut v.backup_sets {
            s.codes.retain(|c| !self.codes.contains(&c.id()));
        }
        if let FolderFilter::Folder(id) = v.preferences.folder
            && self.folders.contains(&id)
        {
            v.preferences.folder = FolderFilter::All;
            v.preferences.state.updated_at = at;
        }
        touch(v, &accounts, &folders, &sets, at);
    }
}
fn touch(
    v: &mut Vault,
    accounts: &BTreeSet<AccountId>,
    folders: &BTreeSet<FolderId>,
    sets: &BTreeSet<BackupSetId>,
    at: Timestamp,
) {
    for a in &mut v.accounts {
        if accounts.contains(&a.id) {
            a.state.updated_at = at;
        }
    }
    for f in &mut v.folders {
        if folders.contains(&f.id) {
            f.state.updated_at = at;
        }
    }
    for s in &mut v.backup_sets {
        if sets.contains(&s.id) {
            s.state.updated_at = at;
        }
    }
}

/// Length/type/count-delimited in-memory stale-review binding, not a storage
/// format, authenticator or persisted plaintext-derived diagnostic. Includes
/// every domain field, including preferences and logical time; an intervening
/// preference commit safely requires review again. No passphrase/DEK is present.
fn binding(v: &Vault) -> Zeroizing<[u8; 32]> {
    fn number(h: &mut Sha256, n: u64) {
        h.update(n.to_le_bytes());
    }
    fn text(h: &mut Sha256, s: &[u8]) {
        number(h, s.len() as u64);
        h.update(s);
    }
    fn optional(h: &mut Sha256, s: Option<&str>) {
        h.update([u8::from(s.is_some())]);
        if let Some(s) = s {
            text(h, s.as_bytes());
        }
    }
    fn state(h: &mut Sha256, s: RecordState) {
        number(h, s.created_at.0);
        number(h, s.updated_at.0);
        match s.deletion {
            DeletionState::Active => h.update([0]),
            DeletionState::Trashed { at } => {
                h.update([1]);
                number(h, at.0);
            }
        }
    }
    let mut h = Sha256::new();
    h.update(b"Authenticator Trash review v1\0");
    number(&mut h, v.schema_version as u64);
    h.update(v.id.as_uuid().as_bytes());
    state(&mut h, v.state);
    let p = &v.preferences;
    h.update(p.id.as_uuid().as_bytes());
    state(&mut h, p.state);
    h.update([p.sort as u8, p.view as u8, u8::from(p.auto_start)]);
    number(&mut h, p.lock_timeout_seconds.into());
    number(&mut h, p.clipboard_timeout_seconds.into());
    text(&mut h, p.query.as_bytes());
    match p.folder {
        FolderFilter::All => h.update([0]),
        FolderFilter::Unfiled => h.update([1]),
        FolderFilter::Folder(id) => {
            h.update([2]);
            h.update(id.as_uuid().as_bytes());
        }
    }
    h.update([0]);
    number(&mut h, v.accounts.len() as u64);
    for a in &v.accounts {
        h.update(a.id.as_uuid().as_bytes());
        state(&mut h, a.state);
        h.update([a.icon as u8]);
        let c = &a.credential;
        text(&mut h, c.service().as_bytes());
        optional(&mut h, c.website());
        optional(&mut h, c.account_identifier());
        text(&mut h, c.secret().expose_bytes());
        let s = c.settings();
        h.update([s.algorithm as u8, s.digits as u8, s.period as u8]);
    }
    h.update([1]);
    number(&mut h, v.folders.len() as u64);
    for f in &v.folders {
        h.update(f.id.as_uuid().as_bytes());
        state(&mut h, f.state);
        text(&mut h, f.name.as_bytes());
        h.update([f.color as u8]);
    }
    h.update([2]);
    number(&mut h, v.memberships.len() as u64);
    for m in &v.memberships {
        h.update(m.id.as_uuid().as_bytes());
        state(&mut h, m.state);
        h.update(m.folder_id.as_uuid().as_bytes());
        h.update(m.account_id.as_uuid().as_bytes());
    }
    h.update([3]);
    number(&mut h, v.backup_sets.len() as u64);
    for s in &v.backup_sets {
        h.update(s.id.as_uuid().as_bytes());
        state(&mut h, s.state);
        h.update(s.account_id.as_uuid().as_bytes());
        number(&mut h, s.codes.len() as u64);
        for c in &s.codes {
            h.update(c.id.as_uuid().as_bytes());
            state(&mut h, c.state);
            text(&mut h, c.expose_value().as_bytes());
            h.update([u8::from(c.used_at.is_some())]);
            if let Some(at) = c.used_at {
                number(&mut h, at.0);
            }
        }
    }
    Zeroizing::new(h.finalize().into())
}
