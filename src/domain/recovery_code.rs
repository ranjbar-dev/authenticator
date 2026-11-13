//! Small, exact per-code transaction guards. No recovery-set value cache.
use super::records::*;
use crate::error::AppError;

#[derive(Clone)]
pub struct CodeTarget {
    account: Account,
    set: BackupSetId,
    set_state: RecordState,
    code: BackupCode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodeChange {
    MarkUsed(bool),
    RestoreUsed(Option<Timestamp>),
    Trash,
}

impl CodeTarget {
    pub fn capture(
        vault: &Vault,
        account: AccountId,
        set: BackupSetId,
        code: BackupCodeId,
    ) -> Result<Self, AppError> {
        let account = vault
            .accounts()
            .iter()
            .find(|a| a.id() == account && a.state().deletion == DeletionState::Active)
            .ok_or(AppError::StaleAccount)?;
        let set = vault
            .backup_sets()
            .iter()
            .find(|s| {
                s.id() == set
                    && s.account_id() == account.id()
                    && s.state().deletion == DeletionState::Active
            })
            .ok_or(AppError::StaleBackupCodes)?;
        let code = set
            .codes()
            .iter()
            .find(|c| c.id() == code && c.state().deletion == DeletionState::Active)
            .ok_or(AppError::StaleBackupCodes)?;
        Ok(Self {
            account: account.clone(),
            set: set.id(),
            set_state: set.state(),
            code: code.clone(),
        })
    }
    pub fn account_id(&self) -> AccountId {
        self.account.id()
    }
    pub fn set_id(&self) -> BackupSetId {
        self.set
    }
    pub fn code_id(&self) -> BackupCodeId {
        self.code.id()
    }
    pub fn used_at(&self) -> Option<Timestamp> {
        self.code.used_at()
    }
    pub fn check(&self, vault: &Vault) -> Result<(), AppError> {
        let live = Self::capture(vault, self.account_id(), self.set, self.code.id())?;
        let a = self.account.credential();
        let b = live.account.credential();
        if self.account.state() != live.account.state()
            || a.service() != b.service()
            || a.website() != b.website()
            || a.account_identifier() != b.account_identifier()
            || !a.duplicate_criteria(b).0
        {
            return Err(AppError::StaleAccount);
        }
        if live.set_state != self.set_state
            || live.code.state() != self.code.state()
            || live.code.used_at() != self.code.used_at()
            || live.code.expose_value() != self.code.expose_value()
        {
            return Err(AppError::StaleBackupCodes);
        }
        Ok(())
    }
}

impl Vault {
    /// Called inside the sole encrypted-store transaction. A stale selection
    /// fails before any field changes, including when timestamps are equal.
    pub fn change_recovery_code(
        &mut self,
        target: &CodeTarget,
        change: CodeChange,
        at: Timestamp,
    ) -> Result<(), AppError> {
        target.check(self)?;
        let at = self.mutation_time(at);
        let used_at = match change {
            CodeChange::MarkUsed(used) => used.then_some(at),
            CodeChange::RestoreUsed(previous) => {
                if previous.is_some_and(|t| t < target.code.state().created_at || t > at) {
                    return Err(AppError::InvalidRecord);
                }
                previous
            }
            CodeChange::Trash => target.used_at(),
        };
        // References were checked above; all following changes are infallible.
        let set = self
            .backup_sets
            .iter_mut()
            .find(|s| s.id == target.set)
            .ok_or(AppError::StaleBackupCodes)?;
        let code = set
            .codes
            .iter_mut()
            .find(|c| c.id == target.code.id())
            .ok_or(AppError::StaleBackupCodes)?;
        code.state.updated_at = at;
        code.used_at = used_at;
        if change == CodeChange::Trash {
            code.state.deletion = DeletionState::Trashed { at };
        }
        set.state.updated_at = at;
        self.accounts
            .iter_mut()
            .find(|a| a.id() == target.account_id())
            .ok_or(AppError::StaleAccount)?
            .state
            .updated_at = at;
        self.state.updated_at = at;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::otp::{OtpSettings, Secret};
    fn fixture() -> Result<(Vault, AccountId, BackupSetId, BackupCodeId), AppError> {
        let mut v = Vault::new(Timestamp(100));
        let a = v.add_account(
            AccountDraft::new(
                "Public detail",
                None,
                None,
                Secret::from_base32("MY")?,
                OtpSettings::default(),
            )?,
            Timestamp(100),
        )?;
        let code = BackupCode::new("public-exact-code", Timestamp(100))?;
        let id = code.id();
        let s = v.add_backup_set(
            a,
            vec![code, BackupCode::new("public-other", Timestamp(100))?],
            Timestamp(100),
        )?;
        Ok((v, a, s, id))
    }
    #[test]
    fn used_undo_touch_aggregate_parents_and_keep_exact_original_history() -> Result<(), AppError> {
        let (mut v, a, s, c) = fixture()?;
        let t = CodeTarget::capture(&v, a, s, c)?;
        v.change_recovery_code(&t, CodeChange::MarkUsed(true), Timestamp(50))?;
        assert_eq!(
            v.backup_sets()[0].codes()[0].used_at(),
            Some(Timestamp(100))
        );
        assert!(t.check(&v).is_err());
        let t = CodeTarget::capture(&v, a, s, c)?;
        v.change_recovery_code(&t, CodeChange::MarkUsed(false), Timestamp(200))?;
        let t = CodeTarget::capture(&v, a, s, c)?;
        v.change_recovery_code(
            &t,
            CodeChange::RestoreUsed(Some(Timestamp(100))),
            Timestamp(150),
        )?;
        assert_eq!(
            v.backup_sets()[0].codes()[0].used_at(),
            Some(Timestamp(100))
        );
        for state in [
            v.state(),
            v.accounts()[0].state(),
            v.backup_sets()[0].state(),
            v.backup_sets()[0].codes()[0].state(),
        ] {
            assert_eq!(state.updated_at, Timestamp(200));
        }
        let t = CodeTarget::capture(&v, a, s, c)?;
        assert_eq!(
            v.change_recovery_code(
                &t,
                CodeChange::RestoreUsed(Some(Timestamp(1))),
                Timestamp(201)
            ),
            Err(AppError::InvalidRecord)
        );
        t.check(&v)?;
        v.validate()
    }
    #[test]
    fn exact_guards_reject_same_timestamp_value_parent_changes_and_hidden_records()
    -> Result<(), AppError> {
        let (mut v, a, s, c) = fixture()?;
        let t = CodeTarget::capture(&v, a, s, c)?;
        v.backup_sets[0].codes[0].value = zeroize::Zeroizing::new("public-replaced-value".into());
        assert_eq!(
            v.change_recovery_code(&t, CodeChange::Trash, Timestamp(100)),
            Err(AppError::StaleBackupCodes)
        );
        assert_eq!(
            v.backup_sets()[0].codes()[0].state().deletion,
            DeletionState::Active
        );
        let t = CodeTarget::capture(&v, a, s, c)?;
        v.edit_account(
            a,
            AccountDraft::new(
                "Public changed",
                None,
                None,
                Secret::from_base32("MY")?,
                OtpSettings::default(),
            )?,
            Timestamp(100),
        )?;
        assert_eq!(t.check(&v), Err(AppError::StaleAccount));
        let t = CodeTarget::capture(&v, a, s, c)?;
        v.trash_account(a, v.accounts()[0].state(), Timestamp(100))?;
        assert_eq!(t.check(&v), Err(AppError::StaleAccount));
        // Generated backend fixture simulates Task18 restore; no restore UI claim.
        v.accounts[0].state.deletion = DeletionState::Active;
        t.check(&v)?;
        v.change_recovery_code(&t, CodeChange::Trash, Timestamp(101))?;
        assert!(CodeTarget::capture(&v, a, s, c).is_err());
        assert_eq!(v.backup_sets()[0].codes().len(), 2);
        v.validate()?;
        v.accounts.clear();
        assert_eq!(v.validate(), Err(AppError::InvalidRecord));
        v.backup_sets.clear();
        v.validate()
    }
}
