//! Immutable selected-record guard and reversible recovery-set mutations.
use super::records::*;
use crate::error::AppError;
use std::collections::{BTreeMap, BTreeSet};
use zeroize::Zeroizing;

#[derive(Clone)]
pub struct RecoveryTarget {
    account: Account,
    set: Option<BackupCodeSet>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        otp::{OtpSettings, Secret},
        recovery_input::RecoveryInput,
    };
    fn fixture() -> Result<(Vault, AccountId), AppError> {
        let mut v = Vault::new(Timestamp(100));
        let id = v.add_account(
            AccountDraft::new(
                "Public recovery fixture",
                None,
                None,
                Secret::from_base32("MY")?,
                OtpSettings::default(),
            )?,
            Timestamp(100),
        )?;
        let mut used = BackupCode::new("public-used", Timestamp(100))?;
        used.set_used(true, Timestamp(100))?;
        v.add_backup_set(
            id,
            vec![used, BackupCode::new("public-removed", Timestamp(100))?],
            Timestamp(100),
        )?;
        Ok((v, id))
    }
    #[test]
    fn replacement_retains_exact_ids_used_history_and_reversible_removed_codes()
    -> Result<(), AppError> {
        let (mut v, id) = fixture()?;
        let target = RecoveryTarget::capture(&v, id)?;
        let original = target.set().ok_or(AppError::InvalidRecord)?;
        let used_id = original.codes()[0].id();
        let removed_id = original.codes()[1].id();
        let values =
            RecoveryInput::parse(" public-used\r\npublic-new\nPublic-used ")?.approved(false)?;
        let summary = RecoveryChanges::preview(&target, &values);
        assert_eq!(
            (
                summary.retained,
                summary.retained_used,
                summary.added,
                summary.removed
            ),
            (1, 1, 2, 1)
        );
        let set_id = v.save_recovery_codes(&target, values, Timestamp(2))?;
        v.validate()?;
        assert_eq!(set_id, original.id());
        let set = &v.backup_sets()[0];
        assert_eq!(set.state().updated_at, Timestamp(100));
        assert_eq!(set.codes()[0].id(), used_id);
        assert_eq!(set.codes()[0].used_at(), Some(Timestamp(100)));
        assert_eq!(set.codes()[1].id(), removed_id);
        assert!(matches!(
            set.codes()[1].state().deletion,
            DeletionState::Trashed { .. }
        ));
        assert!(
            set.codes()[2..]
                .iter()
                .all(|c| c.used_at().is_none() && c.state().created_at == Timestamp(100))
        );
        let next = RecoveryTarget::capture(&v, id)?;
        let values = RecoveryInput::parse("public-used\npublic-removed")?.approved(false)?;
        assert_eq!(RecoveryChanges::preview(&next, &values).restored, 1);
        v.save_recovery_codes(&next, values, Timestamp(101))?;
        v.validate()?;
        assert_eq!(v.backup_sets()[0].codes()[1].id(), removed_id);
        assert_eq!(
            v.backup_sets()[0].codes()[1].state().deletion,
            DeletionState::Active
        );
        assert_eq!(
            v.backup_sets()[0].codes()[0].used_at(),
            Some(Timestamp(100))
        );
        Ok(())
    }
    #[test]
    fn equal_timestamp_stale_content_and_trash_cannot_overwrite() -> Result<(), AppError> {
        let (mut v, id) = fixture()?;
        let old = RecoveryTarget::capture(&v, id)?;
        v.save_recovery_codes(
            &old,
            RecoveryInput::parse("public-used\npublic-fresh")?.approved(false)?,
            Timestamp(100),
        )?;
        assert_eq!(old.check(&v), Err(AppError::StaleBackupCodes));
        assert_eq!(
            v.trash_recovery_set(&old, Timestamp(100)),
            Err(AppError::StaleBackupCodes)
        );
        let live = RecoveryTarget::capture(&v, id)?;
        v.trash_recovery_set(&live, Timestamp(99))?;
        let trashed = RecoveryTarget::capture(&v, id)?;
        assert_eq!(
            v.save_recovery_codes(
                &trashed,
                RecoveryInput::parse("public-new")?.approved(false)?,
                Timestamp(100)
            ),
            Err(AppError::StaleBackupCodes)
        );
        assert_eq!(v.backup_sets()[0].codes().len(), 3);
        assert_eq!(v.accounts()[0].state().deletion, DeletionState::Active);
        v.validate()?;
        Ok(())
    }
    #[test]
    fn preserved_trash_counts_towards_total_limit_and_invalid_changes_are_atomic()
    -> Result<(), AppError> {
        let (mut v, id) = fixture()?;
        v.backup_sets[0].codes = (0..Vault::MAX_RECORDS)
            .map(|i| BackupCode::new(&format!("public-{i}"), Timestamp(100)))
            .collect::<Result<Vec<_>, _>>()?;
        let target = RecoveryTarget::capture(&v, id)?;
        assert_eq!(
            v.save_recovery_codes(
                &target,
                RecoveryInput::parse("public-new")?.approved(false)?,
                Timestamp(101)
            ),
            Err(AppError::InvalidBackupCodes)
        );
        target.check(&v)?;
        assert_eq!(
            v.save_recovery_codes(
                &target,
                vec![Zeroizing::new("same".into()), Zeroizing::new("same".into())],
                Timestamp(101)
            ),
            Err(AppError::InvalidBackupCodes)
        );
        target.check(&v)?;
        v.validate()?;
        Ok(())
    }
    #[test]
    fn dense_ten_thousand_code_preview_and_save_use_exact_matching() -> Result<(), AppError> {
        let (mut v, id) = fixture()?;
        v.backup_sets.clear();
        let target = RecoveryTarget::capture(&v, id)?;
        let text = (0..Vault::MAX_RECORDS)
            .map(|i| format!("public-prefix-{}-{i:05}\n", "x".repeat(64)))
            .collect::<String>();
        let parsed = RecoveryInput::parse(&text)?;
        assert_eq!(
            RecoveryChanges::preview(&target, &parsed.values).added,
            Vault::MAX_RECORDS
        );
        v.save_recovery_codes(&target, parsed.approved(false)?, Timestamp(99))?;
        let target = RecoveryTarget::capture(&v, id)?;
        let parsed = RecoveryInput::parse(&text)?;
        assert_eq!(
            RecoveryChanges::preview(&target, &parsed.values).retained,
            Vault::MAX_RECORDS
        );
        v.save_recovery_codes(&target, parsed.approved(false)?, Timestamp(100))?;
        v.validate()?;
        assert_eq!(v.backup_sets()[0].codes().len(), Vault::MAX_RECORDS);
        Ok(())
    }
}
impl RecoveryTarget {
    pub fn capture(vault: &Vault, id: AccountId) -> Result<Self, AppError> {
        let account = vault
            .accounts()
            .iter()
            .find(|a| a.id() == id && a.state().deletion == DeletionState::Active)
            .ok_or(AppError::StaleAccount)?
            .clone();
        let set = vault
            .backup_sets()
            .iter()
            .find(|s| s.account_id() == id)
            .cloned();
        Ok(Self { account, set })
    }
    pub fn account_id(&self) -> AccountId {
        self.account.id()
    }
    pub fn set(&self) -> Option<&BackupCodeSet> {
        self.set.as_ref()
    }
    /// Full content guard: equal timestamps do not imply equal records.
    pub fn check(&self, vault: &Vault) -> Result<(), AppError> {
        let live = vault
            .accounts()
            .iter()
            .find(|a| a.id() == self.account.id())
            .ok_or(AppError::StaleAccount)?;
        let a = live.credential();
        let b = self.account.credential();
        if live.state() != self.account.state()
            || live.state().deletion != DeletionState::Active
            || a.service() != b.service()
            || a.website() != b.website()
            || a.account_identifier() != b.account_identifier()
            || !a.duplicate_criteria(b).0
        {
            return Err(AppError::StaleAccount);
        }
        let live = vault
            .backup_sets()
            .iter()
            .find(|s| s.account_id() == self.account.id());
        let same = match (live, &self.set) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                a.id() == b.id()
                    && a.state() == b.state()
                    && a.codes().len() == b.codes().len()
                    && a.codes().iter().zip(b.codes()).all(|(a, b)| {
                        a.id() == b.id()
                            && a.state() == b.state()
                            && a.used_at() == b.used_at()
                            && a.expose_value() == b.expose_value()
                    })
            }
            _ => false,
        };
        if same {
            Ok(())
        } else {
            Err(AppError::StaleBackupCodes)
        }
    }
}

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub struct RecoveryChanges {
    pub retained: usize,
    pub retained_used: usize,
    pub added: usize,
    pub removed: usize,
    pub restored: usize,
    pub restored_used: usize,
    pub kept_in_trash: usize,
}
impl RecoveryChanges {
    pub fn preview(target: &RecoveryTarget, values: &[Zeroizing<String>]) -> Self {
        let mut result = Self::default();
        let original: BTreeMap<_, _> = target
            .set()
            .into_iter()
            .flat_map(|s| s.codes())
            .map(|c| (c.expose_value(), c))
            .collect();
        let selected: BTreeSet<_> = values.iter().map(|v| v.as_str()).collect();
        for value in values {
            match original.get(value.as_str()) {
                Some(c) if c.state().deletion == DeletionState::Active => {
                    result.retained += 1;
                    result.retained_used += usize::from(c.used_at().is_some());
                }
                Some(c) => {
                    result.restored += 1;
                    result.restored_used += usize::from(c.used_at().is_some());
                }
                None => result.added += 1,
            }
        }
        if let Some(set) = target.set() {
            for code in set
                .codes()
                .iter()
                .filter(|c| !selected.contains(c.expose_value()))
            {
                if code.state().deletion == DeletionState::Active {
                    result.removed += 1;
                } else {
                    result.kept_in_trash += 1;
                }
            }
        }
        result
    }
}
impl Vault {
    pub fn save_recovery_codes(
        &mut self,
        target: &RecoveryTarget,
        values: Vec<Zeroizing<String>>,
        at: Timestamp,
    ) -> Result<BackupSetId, AppError> {
        target.check(self)?;
        if target
            .set()
            .is_some_and(|s| s.state().deletion != DeletionState::Active)
        {
            return Err(AppError::StaleBackupCodes);
        }
        let mut unique = BTreeSet::new();
        if values.is_empty() || values.len() > Self::MAX_RECORDS {
            return Err(AppError::InvalidBackupCodes);
        }
        for value in &values {
            if BackupCode::normalized_value(value)? != *value || !unique.insert(value.as_str()) {
                return Err(AppError::InvalidBackupCodes);
            }
        }
        let at = self.mutation_time(at);
        let id = if let Some(original) = target.set() {
            let mut codes = original.codes().to_vec();
            let original_values: BTreeSet<_> =
                original.codes().iter().map(|c| c.expose_value()).collect();
            for code in &mut codes {
                let wanted = unique.contains(code.expose_value());
                if wanted && code.state.deletion != DeletionState::Active {
                    code.state.updated_at = at;
                    code.state.deletion = DeletionState::Active;
                } else if !wanted && code.state.deletion == DeletionState::Active {
                    code.state.updated_at = at;
                    code.state.deletion = DeletionState::Trashed { at };
                }
            }
            for value in &values {
                if !original_values.contains(value.as_str()) {
                    codes.push(BackupCode::new(value, at)?);
                }
            }
            let other_count: usize = self
                .backup_sets
                .iter()
                .filter(|s| s.id != original.id())
                .map(|s| s.codes.len())
                .sum();
            if codes.len() > Self::MAX_RECORDS.saturating_sub(other_count) {
                return Err(AppError::InvalidBackupCodes);
            }
            let set = self
                .backup_sets
                .iter_mut()
                .find(|s| s.id == original.id())
                .ok_or(AppError::StaleBackupCodes)?;
            set.codes = codes;
            set.state.updated_at = at;
            set.id
        } else {
            let codes = values
                .iter()
                .map(|v| BackupCode::new(v, at))
                .collect::<Result<Vec<_>, _>>()?;
            self.add_backup_set(target.account_id(), codes, at)?
        };
        self.accounts
            .iter_mut()
            .find(|a| a.id == target.account_id())
            .ok_or(AppError::StaleAccount)?
            .state
            .updated_at = at;
        self.state.updated_at = at;
        Ok(id)
    }
    /// Reversible set deletion: children and their used history remain encrypted.
    pub fn trash_recovery_set(
        &mut self,
        target: &RecoveryTarget,
        at: Timestamp,
    ) -> Result<(), AppError> {
        target.check(self)?;
        let original = target
            .set()
            .filter(|s| s.state().deletion == DeletionState::Active)
            .ok_or(AppError::StaleBackupCodes)?;
        let at = self.mutation_time(at);
        let set = self
            .backup_sets
            .iter_mut()
            .find(|s| s.id == original.id())
            .ok_or(AppError::StaleBackupCodes)?;
        set.state.updated_at = at;
        set.state.deletion = DeletionState::Trashed { at };
        self.accounts
            .iter_mut()
            .find(|a| a.id == target.account_id())
            .ok_or(AppError::StaleAccount)?
            .state
            .updated_at = at;
        self.state.updated_at = at;
        Ok(())
    }
}
