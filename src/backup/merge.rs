use super::*;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Counts {
    pub accounts: usize,
    pub folders: usize,
    pub memberships: usize,
    pub recovery_sets: usize,
    pub recovery_codes: usize,
    pub used_codes: usize,
    pub trashed_records: usize,
}
impl Counts {
    pub(super) fn of(v: &Vault) -> Self {
        let states = v
            .accounts
            .iter()
            .map(|a| a.state)
            .chain(v.folders.iter().map(|f| f.state))
            .chain(v.memberships.iter().map(|m| m.state))
            .chain(
                v.backup_sets
                    .iter()
                    .flat_map(|s| std::iter::once(s.state).chain(s.codes.iter().map(|c| c.state))),
            );
        Self {
            accounts: v.accounts.len(),
            folders: v.folders.len(),
            memberships: v.memberships.len(),
            recovery_sets: v.backup_sets.len(),
            recovery_codes: v.backup_sets.iter().map(|s| s.codes.len()).sum(),
            used_codes: v
                .backup_sets
                .iter()
                .flat_map(|s| &s.codes)
                .filter(|c| c.used_at.is_some())
                .count(),
            trashed_records: states
                .filter(|s| s.deletion != DeletionState::Active)
                .count(),
        }
    }
}
/// Indices refer only to this immutable preview. Contains no names or secrets.
pub struct Conflict {
    pub incoming_account: Option<usize>,
    pub other_incoming_account: Option<usize>,
    pub local_account: Option<usize>,
    pub local_match_count: usize,
    pub incoming_match_count: usize,
    pub same_id: bool,
    pub same_seed_and_settings: bool,
    pub same_service_and_identifier: bool,
    pub folder_id_collision: bool,
    pub folder_name_collision: bool,
    pub global_id_collisions: usize,
}
redacted!(Conflict);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MergeChoice {
    KeepLocal,
    KeepBoth,
}
/// Engine-authoritative decisions for one incoming folder in Merge mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FolderDecisions {
    pub keep_local: bool,
    pub keep_both: bool,
}
impl FolderDecisions {
    fn for_collision(reusable: bool, occupied_id: bool) -> Self {
        Self {
            keep_local: reusable,
            keep_both: reusable || occupied_id,
        }
    }
    fn permits(self, choice: MergeChoice) -> bool {
        match choice {
            MergeChoice::KeepLocal => self.keep_local,
            MergeChoice::KeepBoth => self.keep_both,
        }
    }
}
pub(super) fn folder_decisions(local: &Vault, incoming: &Vault) -> Vec<FolderDecisions> {
    let occupied: BTreeSet<_> = ids(local).into_iter().collect();
    let folders: BTreeSet<_> = local.folders.iter().map(|f| f.id).collect();
    let names: BTreeSet<_> = local.folders.iter().map(|f| f.name.as_str()).collect();
    incoming
        .folders
        .iter()
        .map(|f| {
            FolderDecisions::for_collision(
                folders.contains(&f.id) || names.contains(f.name.as_str()),
                occupied.contains(&f.id.as_uuid()),
            )
        })
        .collect()
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportMode {
    Merge,
    Replace,
}
/// Merge defaults keep conflicting local account bundles and local preferences.
/// Choices address incoming account indices, not potentially colliding IDs.
#[derive(Clone)]
pub struct ImportOptions {
    pub mode: ImportMode,
    pub account_choices: Vec<(usize, MergeChoice)>,
    pub folder_choices: Vec<(usize, MergeChoice)>,
    pub import_display_preferences: bool,
    pub import_security_preferences: bool,
}
impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            mode: ImportMode::Merge,
            account_choices: Vec::new(),
            folder_choices: Vec::new(),
            import_display_preferences: false,
            import_security_preferences: false,
        }
    }
}
redacted!(ImportOptions);
#[derive(Clone, Eq, PartialEq)]
pub struct FolderCopy {
    pub incoming_folder: usize,
    pub original_id: FolderId,
    pub resulting_id: FolderId,
    pub local_folder_id_collision: bool,
    pub local_folder_name_collision: bool,
    pub local_global_id_collision: bool,
    pub matching_content_reused: bool,
    pub in_trash: bool,
    pub copied: bool,
    pub resulting_in_trash: bool,
}
redacted!(FolderCopy);
#[derive(Clone, Eq, PartialEq)]
pub struct ImportSummary {
    pub mode: ImportMode,
    pub before: Counts,
    pub incoming: Counts,
    pub after: Counts,
    pub skipped_account_bundles: usize,
    pub remapped_ids: usize,
    pub removed_local_records: Counts,
    pub local_vault_identity_preserved: bool,
    pub local_preferences_identity_preserved: bool,
    pub display_preferences_imported: bool,
    pub security_preferences_imported: bool,
    pub startup_preserved: bool,
    pub selected_folder_reset: bool,
    /// Every incoming folder resolves to an explicit copy or local reuse.
    /// Copied names/colors/timestamps stay unchanged. UI must display
    /// these resolutions with incoming_vault metadata before approving apply.
    pub folder_copies: Vec<FolderCopy>,
}
redacted!(ImportSummary);
fn ids(v: &Vault) -> Vec<Uuid> {
    std::iter::once(v.id.as_uuid())
        .chain(std::iter::once(v.preferences.id.as_uuid()))
        .chain(v.accounts.iter().map(|a| a.id.as_uuid()))
        .chain(v.folders.iter().map(|f| f.id.as_uuid()))
        .chain(v.memberships.iter().map(|m| m.id.as_uuid()))
        .chain(v.backup_sets.iter().flat_map(|s| {
            std::iter::once(s.id.as_uuid()).chain(s.codes.iter().map(|c| c.id.as_uuid()))
        }))
        .collect()
}
pub(super) fn unique_ids(v: &Vault) -> Result<()> {
    let ids = ids(v);
    if ids.iter().copied().collect::<BTreeSet<_>>().len() != ids.len() {
        return Err(BackupError::Invalid);
    }
    Ok(())
}
pub(super) fn conflicts(local: &Vault, incoming: &Vault) -> Result<Vec<Conflict>> {
    // Ordered indices borrow seeds; no persistent or owned seed fingerprints.
    // Canonical metadata keys zeroize on drop. Only actual candidate pairs run
    // the shared exact duplicate policy. Groups report exact union counts and
    // representatives per incoming account, avoiding dense pair enumeration.
    fn seed_key(a: &Account) -> (&[u8], u8, u8, u8) {
        let s = a.credential.settings();
        (
            a.credential.secret().expose_bytes(),
            s.algorithm as u8,
            s.digits as u8,
            s.period as u8,
        )
    }
    let accounts: Vec<_> = local.accounts.iter().chain(&incoming.accounts).collect();
    let names: Vec<_> = accounts
        .iter()
        .map(|a| {
            (
                Zeroizing::new(a.credential.service().to_lowercase()),
                Zeroizing::new(
                    a.credential
                        .account_identifier()
                        .unwrap_or_default()
                        .to_lowercase(),
                ),
            )
        })
        .collect();
    #[derive(Clone, Copy, Default)]
    struct Group {
        local: usize,
        incoming: usize,
        local_rep: Option<usize>,
        incoming_rep: Option<usize>,
    }
    impl Group {
        fn add(&mut self, index: usize, base: usize) {
            if index < base {
                self.local += 1;
                self.local_rep.get_or_insert(index);
            } else {
                self.incoming += 1;
                self.incoming_rep.get_or_insert(index);
            }
        }
        fn representative(self) -> Option<usize> {
            self.local_rep.or(self.incoming_rep)
        }
    }
    let mut seed_index: BTreeMap<_, Group> = BTreeMap::new();
    let mut name_index: BTreeMap<_, Group> = BTreeMap::new();
    let mut both_index: BTreeMap<_, Group> = BTreeMap::new();
    let mut id_index = BTreeMap::new();
    for (j, a) in local.accounts.iter().enumerate() {
        let name = (names[j].0.as_str(), names[j].1.as_str());
        seed_index
            .entry(seed_key(a))
            .or_default()
            .add(j, local.accounts.len());
        name_index
            .entry(name)
            .or_default()
            .add(j, local.accounts.len());
        both_index
            .entry((seed_key(a), name))
            .or_default()
            .add(j, local.accounts.len());
        id_index.insert(a.id, j);
    }
    let mut out = Vec::new();
    for (i, a) in incoming.accounts.iter().enumerate() {
        let index = local.accounts.len() + i;
        let name = (names[index].0.as_str(), names[index].1.as_str());
        let seed = seed_index.get(&seed_key(a)).copied().unwrap_or_default();
        let identity = name_index.get(&name).copied().unwrap_or_default();
        let both = both_index
            .get(&(seed_key(a), name))
            .copied()
            .unwrap_or_default();
        let same_id = id_index.get(&a.id).copied();
        // Verify representative equivalence with the single shared domain
        // policy, rather than introduce a subtly different warning rule.
        if seed
            .representative()
            .is_some_and(|j| !a.credential.duplicate_criteria(&accounts[j].credential).0)
            || identity
                .representative()
                .is_some_and(|j| !a.credential.duplicate_criteria(&accounts[j].credential).1)
        {
            return Err(BackupError::Invalid);
        }
        let extra_id = same_id.is_some_and(|j| {
            let (s, n) = a.credential.duplicate_criteria(&accounts[j].credential);
            !s && !n
        });
        let local_count = seed.local + identity.local - both.local + usize::from(extra_id);
        let incoming_count = seed.incoming + identity.incoming - both.incoming;
        if local_count + incoming_count > 0 {
            out.push(Conflict {
                incoming_account: Some(i),
                other_incoming_account: seed
                    .incoming_rep
                    .or(identity.incoming_rep)
                    .map(|j| j - local.accounts.len()),
                local_account: seed.local_rep.or(identity.local_rep).or(same_id),
                local_match_count: local_count,
                incoming_match_count: incoming_count,
                same_id: same_id.is_some(),
                same_seed_and_settings: seed.representative().is_some(),
                same_service_and_identifier: identity.representative().is_some(),
                folder_id_collision: false,
                folder_name_collision: false,
                global_id_collisions: 0,
            });
        }
        seed_index
            .entry(seed_key(a))
            .or_default()
            .add(index, local.accounts.len());
        name_index
            .entry(name)
            .or_default()
            .add(index, local.accounts.len());
        both_index
            .entry((seed_key(a), name))
            .or_default()
            .add(index, local.accounts.len());
    }
    let local_folder_ids: BTreeSet<_> = local.folders.iter().map(|f| f.id).collect();
    let local_folder_names: BTreeSet<_> = local.folders.iter().map(|f| f.name.as_str()).collect();
    let folder_id = incoming
        .folders
        .iter()
        .any(|a| local_folder_ids.contains(&a.id));
    // Existing folder equality is exact, unchanged by credential Unicode policy.
    let folder_name = incoming
        .folders
        .iter()
        .any(|a| local_folder_names.contains(a.name.as_str()));
    let local_ids: BTreeSet<_> = ids(local).into_iter().collect();
    let collisions = ids(incoming)
        .iter()
        .filter(|id| local_ids.contains(id))
        .count();
    if folder_id || folder_name || collisions > 0 {
        out.push(Conflict {
            incoming_account: None,
            other_incoming_account: None,
            local_account: None,
            local_match_count: 0,
            incoming_match_count: 0,
            same_id: false,
            same_seed_and_settings: false,
            same_service_and_identifier: false,
            folder_id_collision: folder_id,
            folder_name_collision: folder_name,
            global_id_collisions: collisions,
        });
    }
    Ok(out)
}
pub(super) fn build(
    local: &Vault,
    incoming: &Vault,
    conflicts: &[Conflict],
    options: ImportOptions,
) -> Result<(Vault, ImportSummary)> {
    #[derive(Eq, PartialEq, Ord, PartialOrd)]
    struct FolderContent<'a> {
        name: &'a str,
        color: u8,
        created: Timestamp,
        updated: Timestamp,
        trashed_at: Option<Timestamp>,
    }
    impl<'a> FolderContent<'a> {
        fn of(folder: &'a Folder) -> Self {
            Self {
                name: folder.name.as_str(),
                color: folder.color as u8,
                created: folder.state.created_at,
                updated: folder.state.updated_at,
                trashed_at: match folder.state.deletion {
                    DeletionState::Active => None,
                    DeletionState::Trashed { at } => Some(at),
                },
            }
        }
    }
    let local_record_ids: BTreeSet<_> = ids(local).into_iter().collect();
    let local_folders_by_id: BTreeMap<_, _> = local.folders.iter().map(|f| (f.id, f)).collect();
    let mut local_folders_by_name: BTreeMap<&str, &Folder> = BTreeMap::new();
    let mut local_folders_by_content: BTreeMap<FolderContent<'_>, &Folder> = BTreeMap::new();
    for folder in &local.folders {
        local_folders_by_name
            .entry(folder.name.as_str())
            .and_modify(|previous| {
                if folder.id < previous.id {
                    *previous = folder;
                }
            })
            .or_insert(folder);
        local_folders_by_content
            .entry(FolderContent::of(folder))
            .and_modify(|previous| {
                if folder.id < previous.id {
                    *previous = folder;
                }
            })
            .or_insert(folder);
    }
    if options.account_choices.len() > Vault::MAX_RECORDS {
        return Err(BackupError::InvalidDecision);
    }
    if options.folder_choices.len() > Vault::MAX_RECORDS {
        return Err(BackupError::InvalidDecision);
    }
    let mut folder_choices = BTreeMap::new();
    for (i, choice) in options.folder_choices {
        let folder = incoming
            .folders
            .get(i)
            .ok_or(BackupError::InvalidDecision)?;
        let reusable = local_folders_by_id.contains_key(&folder.id)
            || local_folders_by_name.contains_key(folder.name.as_str());
        let allowed = FolderDecisions::for_collision(
            reusable,
            local_record_ids.contains(&folder.id.as_uuid()),
        );
        if options.mode == ImportMode::Replace
            || !allowed.permits(choice)
            || folder_choices.insert(i, choice).is_some()
        {
            return Err(BackupError::InvalidDecision);
        }
    }
    let conflicting: BTreeSet<_> = conflicts
        .iter()
        .filter_map(|c| c.incoming_account)
        .collect();
    let mut choices = BTreeMap::new();
    for (i, c) in options.account_choices {
        if !conflicting.contains(&i)
            || choices.insert(i, c).is_some()
            || options.mode == ImportMode::Replace
        {
            return Err(BackupError::InvalidDecision);
        }
    }
    let mut candidate = if options.mode == ImportMode::Replace {
        incoming.clone()
    } else {
        local.clone()
    };
    let mut summary = ImportSummary {
        mode: options.mode,
        before: Counts::of(local),
        incoming: Counts::of(incoming),
        after: Counts::default(),
        skipped_account_bundles: 0,
        remapped_ids: 0,
        removed_local_records: if options.mode == ImportMode::Replace {
            Counts::of(local)
        } else {
            Counts::default()
        },
        local_vault_identity_preserved: true,
        local_preferences_identity_preserved: true,
        display_preferences_imported: options.import_display_preferences,
        security_preferences_imported: options.import_security_preferences,
        startup_preserved: true,
        selected_folder_reset: false,
        folder_copies: Vec::new(),
    };
    let mut folder_map = BTreeMap::new();
    let mut folder_copied = BTreeMap::new();
    if options.mode == ImportMode::Merge {
        let mut used = local_record_ids.clone();
        let mut map = BTreeMap::new();
        for id in ids(incoming).into_iter().skip(2) {
            let mut mapped = id;
            while !used.insert(mapped) {
                mapped = Uuid::new_v4();
            }
            map.insert(id, mapped);
        }
        let mut account_map = BTreeMap::new();
        for (i, a) in incoming.accounts.iter().enumerate() {
            if conflicting.contains(&i)
                && choices.get(&i).copied().unwrap_or(MergeChoice::KeepLocal)
                    == MergeChoice::KeepLocal
            {
                summary.skipped_account_bundles += 1;
                continue;
            }
            let mut a = a.clone();
            let old = a.id;
            a.id = AccountId::from_uuid(map[&old.as_uuid()]).map_err(|_| BackupError::Invalid)?;
            summary.remapped_ids += usize::from(a.id != old);
            account_map.insert(old, a.id);
            candidate.accounts.push(a);
        }
        for (index, f) in incoming.folders.iter().enumerate() {
            let same_id = local_folders_by_id.get(&f.id).copied();
            let named = local_folders_by_name.get(f.name.as_str()).copied();
            // A prior import may have remapped this ID because another local
            // entity owns it. Reuse exact folder content before copying again.
            // Ordinary different-ID folders retain their distinct identities.
            let repeated_collision = local_record_ids
                .contains(&f.id.as_uuid())
                .then(|| local_folders_by_content.get(&FolderContent::of(f)).copied())
                .flatten();
            let reuse = match folder_choices.get(&index) {
                Some(MergeChoice::KeepBoth) => None,
                Some(MergeChoice::KeepLocal) => same_id.or(named),
                None => same_id.or(repeated_collision),
            };
            if let Some(existing) = reuse {
                folder_map.insert(f.id, existing.id);
                folder_copied.insert(f.id, false);
                continue;
            }
            let mut f = f.clone();
            let old = f.id;
            f.id = FolderId::from_uuid(map[&old.as_uuid()]).map_err(|_| BackupError::Invalid)?;
            summary.remapped_ids += usize::from(f.id != old);
            folder_map.insert(old, f.id);
            folder_copied.insert(old, true);
            candidate.folders.push(f);
        }
        for m in &incoming.memberships {
            let Some(account) = account_map.get(&m.account_id) else {
                continue;
            };
            let mut m = m.clone();
            let old = m.id;
            m.id =
                MembershipId::from_uuid(map[&m.id.as_uuid()]).map_err(|_| BackupError::Invalid)?;
            summary.remapped_ids += usize::from(m.id != old);
            m.account_id = *account;
            m.folder_id = folder_map[&m.folder_id];
            candidate.memberships.push(m);
        }
        for s in &incoming.backup_sets {
            let Some(account) = account_map.get(&s.account_id) else {
                continue;
            };
            let mut s = s.clone();
            let old = s.id;
            s.id =
                BackupSetId::from_uuid(map[&s.id.as_uuid()]).map_err(|_| BackupError::Invalid)?;
            summary.remapped_ids += usize::from(s.id != old);
            s.account_id = *account;
            for code in &mut s.codes {
                let old = code.id;
                code.id = BackupCodeId::from_uuid(map[&code.id.as_uuid()])
                    .map_err(|_| BackupError::Invalid)?;
                summary.remapped_ids += usize::from(code.id != old);
            }
            candidate.backup_sets.push(s);
        }
    } else {
        // Replace retains local vault/preferences identities. An incoming entity
        // can collide across types with either, so remap its entire graph too.
        let mut used = BTreeSet::from([local.id.as_uuid(), local.preferences.id.as_uuid()]);
        let mut map = BTreeMap::new();
        for id in ids(incoming).into_iter().skip(2) {
            let mut mapped = id;
            while !used.insert(mapped) {
                mapped = Uuid::new_v4();
            }
            if mapped != id {
                summary.remapped_ids += 1;
            }
            map.insert(id, mapped);
        }
        for a in &mut candidate.accounts {
            a.id = AccountId::from_uuid(map[&a.id.as_uuid()]).map_err(|_| BackupError::Invalid)?;
        }
        for f in &mut candidate.folders {
            let old = f.id;
            f.id = FolderId::from_uuid(map[&old.as_uuid()]).map_err(|_| BackupError::Invalid)?;
            folder_map.insert(old, f.id);
            folder_copied.insert(old, true);
        }
        for m in &mut candidate.memberships {
            m.id =
                MembershipId::from_uuid(map[&m.id.as_uuid()]).map_err(|_| BackupError::Invalid)?;
            m.account_id = AccountId::from_uuid(map[&m.account_id.as_uuid()])
                .map_err(|_| BackupError::Invalid)?;
            m.folder_id = folder_map[&m.folder_id];
        }
        for s in &mut candidate.backup_sets {
            s.id =
                BackupSetId::from_uuid(map[&s.id.as_uuid()]).map_err(|_| BackupError::Invalid)?;
            s.account_id = AccountId::from_uuid(map[&s.account_id.as_uuid()])
                .map_err(|_| BackupError::Invalid)?;
            for c in &mut s.codes {
                c.id = BackupCodeId::from_uuid(map[&c.id.as_uuid()])
                    .map_err(|_| BackupError::Invalid)?;
            }
        }
    }
    let local_folder_ids: BTreeSet<_> = local.folders.iter().map(|f| f.id).collect();
    let local_folder_names: BTreeSet<_> = local.folders.iter().map(|f| f.name.as_str()).collect();
    let resulting_folders: BTreeMap<_, _> = candidate.folders.iter().map(|f| (f.id, f)).collect();
    for (index, folder) in incoming.folders.iter().enumerate() {
        summary.folder_copies.push(FolderCopy {
            incoming_folder: index,
            original_id: folder.id,
            resulting_id: folder_map[&folder.id],
            local_folder_id_collision: local_folder_ids.contains(&folder.id),
            local_folder_name_collision: local_folder_names.contains(folder.name.as_str()),
            local_global_id_collision: local_record_ids.contains(&folder.id.as_uuid()),
            matching_content_reused: !folder_copied[&folder.id]
                && folder_map[&folder.id] != folder.id
                && local_folders_by_content
                    .get(&FolderContent::of(folder))
                    .is_some_and(|existing| existing.id == folder_map[&folder.id]),
            in_trash: folder.state.deletion != DeletionState::Active,
            copied: folder_copied[&folder.id],
            resulting_in_trash: resulting_folders[&folder_map[&folder.id]].state.deletion
                != DeletionState::Active,
        });
    }
    // Local identity and startup are always retained. Whole Replace replaces all
    // record bundles, never silently changes the destination vault's identity.
    candidate.id = local.id;
    candidate.preferences = local.preferences.clone();
    if options.import_display_preferences {
        candidate.preferences.sort = incoming.preferences.sort;
        candidate.preferences.view = incoming.preferences.view;
        candidate.preferences.query = incoming.preferences.query.clone();
        candidate.preferences.folder = match incoming.preferences.folder {
            FolderFilter::Folder(id) => FolderFilter::Folder(folder_map[&id]),
            f => f,
        };
    }
    if options.import_security_preferences {
        candidate.preferences.lock_timeout_seconds = incoming.preferences.lock_timeout_seconds;
        candidate.preferences.clipboard_timeout_seconds =
            incoming.preferences.clipboard_timeout_seconds;
    }
    if let FolderFilter::Folder(id) = candidate.preferences.folder
        && !candidate
            .folders
            .iter()
            .any(|f| f.id == id && f.state.deletion == DeletionState::Active)
    {
        candidate.preferences.folder = FolderFilter::All;
        summary.selected_folder_reset = true;
    }
    // Logical aggregate time accommodates future imported metadata without
    // changing creation/update/trash/used timestamps on individual records.
    candidate.state = local.state;
    candidate.state.updated_at =
        Timestamp(local.state.updated_at.0.max(incoming.state.updated_at.0));
    if options.import_display_preferences
        || options.import_security_preferences
        || summary.selected_folder_reset
    {
        candidate.preferences.state.updated_at = candidate.state.updated_at;
    }
    candidate.validate().map_err(|_| BackupError::Limit)?;
    unique_ids(&candidate)?;
    summary.after = Counts::of(&candidate);
    Ok((candidate, summary))
}
