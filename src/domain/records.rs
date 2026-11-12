//! Unlocked, in-memory records. Only authenticated ciphertext may persist them.
use super::{
    clock::Clock,
    otp::{OtpSettings, Secret},
    validation,
};
use crate::error::AppError;
use std::{collections::BTreeSet, fmt};
use uuid::Uuid;
use zeroize::{Zeroize, Zeroizing};

macro_rules! ids {
    ($($name:ident),+ $(,)?) => {$(
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(Uuid);
        impl $name {
            pub fn new() -> Self { Self(Uuid::new_v4()) }
            pub fn as_uuid(&self) -> Uuid { self.0 }
            /// Import boundary must reject missing/nil identity, never replace it.
            pub fn from_uuid(value: Uuid) -> Result<Self, AppError> {
                if value.is_nil() { Err(AppError::InvalidRecord) } else { Ok(Self(value)) }
            }
        }
        impl Default for $name { fn default() -> Self { Self::new() } }
    )+};
}
ids!(
    VaultId,
    AccountId,
    FolderId,
    MembershipId,
    BackupSetId,
    BackupCodeId,
    PreferencesId
);

macro_rules! redacted_debug {
    ($($name:ident),+ $(,)?) => {$(
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), "([REDACTED])"))
            }
        }
    )+};
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(pub u64);
impl Timestamp {
    pub fn now(clock: &dyn Clock) -> Result<Self, AppError> {
        clock.unix_seconds().map(Self)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DeletionState {
    #[default]
    Active,
    /// No automatic expiry. Permanent removal is a separate explicit workflow.
    Trashed { at: Timestamp },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordState {
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub deletion: DeletionState,
}
impl RecordState {
    pub fn new(at: Timestamp) -> Self {
        Self {
            created_at: at,
            updated_at: at,
            deletion: DeletionState::Active,
        }
    }
    pub fn validate(self) -> Result<(), AppError> {
        if self.updated_at < self.created_at
            || matches!(self.deletion, DeletionState::Trashed { at } if at < self.created_at || at > self.updated_at)
        {
            return Err(AppError::InvalidRecord);
        }
        Ok(())
    }
    fn touch(&mut self, at: Timestamp) -> Result<(), AppError> {
        if at < self.updated_at {
            return Err(AppError::ClockUnavailable);
        }
        self.updated_at = at;
        Ok(())
    }
}

/// A third-party service credential, never an Authenticator login/profile.
#[derive(Clone)]
pub struct AccountDraft {
    service: String,
    website: Option<String>,
    account_identifier: Option<String>,
    secret: Secret,
    settings: OtpSettings,
}
impl AccountDraft {
    pub fn new(
        service: &str,
        website: Option<&str>,
        account_identifier: Option<&str>,
        secret: Secret,
        settings: OtpSettings,
    ) -> Result<Self, AppError> {
        Ok(Self {
            service: validation::text(service, 256, AppError::MissingService)?,
            website: validation::website(website)?,
            account_identifier: validation::optional_text(
                account_identifier,
                320,
                AppError::InvalidAccount,
            )?,
            secret,
            settings,
        })
    }
    pub fn service(&self) -> &str {
        &self.service
    }
    pub fn website(&self) -> Option<&str> {
        self.website.as_deref()
    }
    pub fn account_identifier(&self) -> Option<&str> {
        self.account_identifier.as_deref()
    }
    pub fn secret(&self) -> &Secret {
        &self.secret
    }
    pub fn settings(&self) -> OtpSettings {
        self.settings
    }
    /// Same policy for existing-vault and reviewed-batch warnings. No provider
    /// alias guessing and no secret text is returned.
    pub fn duplicate_criteria(&self, other: &Self) -> (bool, bool) {
        let service = Zeroizing::new(self.service.to_lowercase());
        let other_service = Zeroizing::new(other.service.to_lowercase());
        let identifier = Zeroizing::new(
            self.account_identifier
                .as_deref()
                .unwrap_or_default()
                .to_lowercase(),
        );
        let other_identifier = Zeroizing::new(
            other
                .account_identifier
                .as_deref()
                .unwrap_or_default()
                .to_lowercase(),
        );
        (
            self.secret.same_seed(&other.secret) && self.settings == other.settings,
            service == other_service && identifier == other_identifier,
        )
    }
    pub fn validate(&self) -> Result<(), AppError> {
        if validation::text(&self.service, 256, AppError::MissingService)? != self.service
            || validation::website(self.website.as_deref())? != self.website
            || validation::optional_text(
                self.account_identifier.as_deref(),
                320,
                AppError::InvalidAccount,
            )? != self.account_identifier
        {
            return Err(AppError::InvalidRecord);
        }
        Ok(())
    }
}
impl Drop for AccountDraft {
    fn drop(&mut self) {
        self.service.zeroize();
        self.website.zeroize();
        self.account_identifier.zeroize();
    }
}

#[derive(Clone)]
pub struct Account {
    pub(crate) id: AccountId,
    pub(crate) state: RecordState,
    pub(crate) credential: AccountDraft,
    // Local bundled icon token only; no path or remote resource.
    pub(crate) icon: LocalIcon,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LocalIcon {
    #[default]
    Initials,
}
impl Account {
    pub fn new(credential: AccountDraft, at: Timestamp) -> Self {
        Self {
            id: AccountId::new(),
            state: RecordState::new(at),
            credential,
            icon: LocalIcon::Initials,
        }
    }
    pub fn id(&self) -> AccountId {
        self.id
    }
    pub fn state(&self) -> RecordState {
        self.state
    }
    pub fn credential(&self) -> &AccountDraft {
        &self.credential
    }
    pub fn icon(&self) -> LocalIcon {
        self.icon
    }
    pub fn validate(&self) -> Result<(), AppError> {
        self.state.validate()?;
        self.credential.validate()
    }
    /// Whole validated replacement. Identity, creation, trash, icon and links stay.
    /// Failed clock validation leaves the original untouched.
    pub fn edit(&mut self, credential: AccountDraft, at: Timestamp) -> Result<(), AppError> {
        self.state.touch(at)?;
        self.credential = credential;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FolderColor {
    Blue,
    Purple,
    Pink,
    Red,
    Orange,
    Green,
}
#[derive(Clone)]
pub struct Folder {
    pub(crate) id: FolderId,
    pub(crate) state: RecordState,
    pub(crate) name: String,
    pub(crate) color: FolderColor,
}
impl Folder {
    pub fn new(name: &str, color: FolderColor, at: Timestamp) -> Result<Self, AppError> {
        Ok(Self {
            id: FolderId::new(),
            state: RecordState::new(at),
            name: validation::text(name, 256, AppError::InvalidRecord)?,
            color,
        })
    }
    pub fn id(&self) -> FolderId {
        self.id
    }
    pub fn state(&self) -> RecordState {
        self.state
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn color(&self) -> FolderColor {
        self.color
    }
    pub fn validate(&self) -> Result<(), AppError> {
        self.state.validate()?;
        if validation::text(&self.name, 256, AppError::InvalidRecord)? != self.name {
            return Err(AppError::InvalidRecord);
        }
        Ok(())
    }
}
impl Drop for Folder {
    fn drop(&mut self) {
        self.name.zeroize();
    }
}

#[derive(Clone)]
pub struct Membership {
    pub(crate) id: MembershipId,
    pub(crate) state: RecordState,
    pub(crate) folder_id: FolderId,
    pub(crate) account_id: AccountId,
}
impl Membership {
    pub fn id(&self) -> MembershipId {
        self.id
    }
    pub fn state(&self) -> RecordState {
        self.state
    }
    pub fn folder_id(&self) -> FolderId {
        self.folder_id
    }
    pub fn account_id(&self) -> AccountId {
        self.account_id
    }
}

#[derive(Clone)]
pub struct BackupCode {
    pub(crate) id: BackupCodeId,
    pub(crate) state: RecordState,
    pub(crate) value: Zeroizing<String>,
    pub(crate) used_at: Option<Timestamp>,
}
impl BackupCode {
    pub fn new(value: &str, at: Timestamp) -> Result<Self, AppError> {
        Ok(Self {
            id: BackupCodeId::new(),
            state: RecordState::new(at),
            value: Zeroizing::new(validation::text(value, 4096, AppError::InvalidRecord)?),
            used_at: None,
        })
    }
    pub fn id(&self) -> BackupCodeId {
        self.id
    }
    pub fn state(&self) -> RecordState {
        self.state
    }
    pub fn expose_value(&self) -> &str {
        &self.value
    }
    pub fn used_at(&self) -> Option<Timestamp> {
        self.used_at
    }
    pub fn validate(&self) -> Result<(), AppError> {
        self.state.validate()?;
        let normalized = Zeroizing::new(validation::text(
            &self.value,
            4096,
            AppError::InvalidRecord,
        )?);
        if normalized != self.value
            || self
                .used_at
                .is_some_and(|at| at < self.state.created_at || at > self.state.updated_at)
        {
            return Err(AppError::InvalidRecord);
        }
        Ok(())
    }
    pub fn set_used(&mut self, used: bool, at: Timestamp) -> Result<(), AppError> {
        self.state.touch(at)?;
        self.used_at = used.then_some(at);
        Ok(())
    }
}
#[derive(Clone)]
pub struct BackupCodeSet {
    pub(crate) id: BackupSetId,
    pub(crate) state: RecordState,
    pub(crate) account_id: AccountId,
    pub(crate) codes: Vec<BackupCode>,
}
impl BackupCodeSet {
    pub fn id(&self) -> BackupSetId {
        self.id
    }
    pub fn state(&self) -> RecordState {
        self.state
    }
    pub fn account_id(&self) -> AccountId {
        self.account_id
    }
    pub fn codes(&self) -> &[BackupCode] {
        &self.codes
    }
    pub fn validate(&self) -> Result<(), AppError> {
        self.state.validate()?;
        if self.codes.is_empty() || self.codes.len() > Vault::MAX_RECORDS {
            return Err(AppError::InvalidRecord);
        }
        let mut ids = BTreeSet::new();
        let mut values = BTreeSet::new();
        for code in &self.codes {
            code.validate()?;
            if !ids.insert(code.id)
                || !values.insert(code.expose_value())
                || code.state.updated_at > self.state.updated_at
            {
                return Err(AppError::InvalidRecord);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SortOrder {
    #[default]
    LatestFirst,
    EarliestFirst,
    ServiceAscending,
    ServiceDescending,
    AccountAscending,
    AccountDescending,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ViewMode {
    #[default]
    List,
    Grid,
}
#[derive(Clone)]
pub struct Preferences {
    pub(crate) id: PreferencesId,
    pub(crate) state: RecordState,
    pub(crate) sort: SortOrder,
    pub(crate) view: ViewMode,
    pub(crate) auto_start: bool,
    pub(crate) lock_timeout_seconds: u32,
    pub(crate) clipboard_timeout_seconds: u32,
}
impl Preferences {
    pub fn new(at: Timestamp) -> Self {
        Self {
            id: PreferencesId::new(),
            state: RecordState::new(at),
            sort: SortOrder::default(),
            view: ViewMode::default(),
            auto_start: false,
            lock_timeout_seconds: 300,
            clipboard_timeout_seconds: 30,
        }
    }
    pub fn id(&self) -> PreferencesId {
        self.id
    }
    pub fn state(&self) -> RecordState {
        self.state
    }
    pub fn sort(&self) -> SortOrder {
        self.sort
    }
    pub fn view(&self) -> ViewMode {
        self.view
    }
    pub fn auto_start(&self) -> bool {
        self.auto_start
    }
    pub fn lock_timeout_seconds(&self) -> u32 {
        self.lock_timeout_seconds
    }
    pub fn clipboard_timeout_seconds(&self) -> u32 {
        self.clipboard_timeout_seconds
    }
    pub fn validate(&self) -> Result<(), AppError> {
        self.state.validate()?;
        if !(1..=86_400).contains(&self.lock_timeout_seconds)
            || !(1..=86_400).contains(&self.clipboard_timeout_seconds)
        {
            return Err(AppError::InvalidRecord);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DuplicateMatch {
    pub account_id: AccountId,
    pub same_seed_and_settings: bool,
    pub same_service_and_identifier: bool,
    pub in_trash: bool,
}

#[derive(Clone)]
pub struct Vault {
    pub(crate) id: VaultId,
    pub(crate) schema_version: u32,
    pub(crate) state: RecordState,
    pub(crate) preferences: Preferences,
    pub(crate) accounts: Vec<Account>,
    pub(crate) folders: Vec<Folder>,
    pub(crate) memberships: Vec<Membership>,
    pub(crate) backup_sets: Vec<BackupCodeSet>,
}
impl Vault {
    pub const SCHEMA_VERSION: u32 = 1;
    /// Bound entity vectors and total recovery-code count at every import boundary.
    /// The encrypted reader must ALSO bound bytes before deserializing allocations.
    pub const MAX_RECORDS: usize = 10_000;
    pub fn new(at: Timestamp) -> Self {
        Self {
            id: VaultId::new(),
            schema_version: Self::SCHEMA_VERSION,
            state: RecordState::new(at),
            preferences: Preferences::new(at),
            accounts: Vec::new(),
            folders: Vec::new(),
            memberships: Vec::new(),
            backup_sets: Vec::new(),
        }
    }
    pub fn id(&self) -> VaultId {
        self.id
    }
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }
    pub fn state(&self) -> RecordState {
        self.state
    }
    pub fn preferences(&self) -> &Preferences {
        &self.preferences
    }
    pub fn set_lock_timeout(&mut self, seconds: u32, at: Timestamp) -> Result<(), AppError> {
        if !(1..=86_400).contains(&seconds)
            || at < self.state.updated_at
            || at < self.preferences.state.updated_at
        {
            return Err(AppError::InvalidRecord);
        }
        self.preferences.lock_timeout_seconds = seconds;
        self.preferences.state.touch(at)?;
        self.state.touch(at)
    }
    pub fn accounts(&self) -> &[Account] {
        &self.accounts
    }
    pub fn folders(&self) -> &[Folder] {
        &self.folders
    }
    pub fn memberships(&self) -> &[Membership] {
        &self.memberships
    }
    pub fn backup_sets(&self) -> &[BackupCodeSet] {
        &self.backup_sets
    }

    /// Warning only. Caller presents it before explicit Save; never auto-overwrite.
    /// Compare decoded seeds only within the unlocked vault. No fingerprint/index.
    /// Metadata uses trim + Unicode lowercase, without email/provider alias rules.
    pub fn duplicates(
        &self,
        draft: &AccountDraft,
        editing: Option<AccountId>,
    ) -> Vec<DuplicateMatch> {
        self.accounts
            .iter()
            .filter(|a| Some(a.id) != editing)
            .filter_map(|a| {
                let c = &a.credential;
                let (same_seed_and_settings, same_service_and_identifier) =
                    c.duplicate_criteria(draft);
                (same_seed_and_settings || same_service_and_identifier).then_some(DuplicateMatch {
                    account_id: a.id,
                    same_seed_and_settings,
                    same_service_and_identifier,
                    in_trash: a.state.deletion != DeletionState::Active,
                })
            })
            .collect()
    }

    pub fn add_account(
        &mut self,
        credential: AccountDraft,
        at: Timestamp,
    ) -> Result<AccountId, AppError> {
        if self.accounts.len() >= Self::MAX_RECORDS {
            return Err(AppError::InvalidRecord);
        }
        self.state.touch(at)?;
        let account = Account::new(credential, at);
        let id = account.id;
        self.accounts.push(account);
        Ok(id)
    }
    pub fn edit_account(
        &mut self,
        id: AccountId,
        credential: AccountDraft,
        at: Timestamp,
    ) -> Result<(), AppError> {
        let account = self
            .accounts
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or(AppError::InvalidRecord)?;
        // Validate all fallible operations before either record changes.
        if at < self.state.updated_at || at < account.state.updated_at {
            return Err(AppError::ClockUnavailable);
        }
        account.edit(credential, at)?;
        self.state.touch(at)
    }
    pub fn add_folder(
        &mut self,
        name: &str,
        color: FolderColor,
        at: Timestamp,
    ) -> Result<FolderId, AppError> {
        if self.folders.len() >= Self::MAX_RECORDS {
            return Err(AppError::InvalidRecord);
        }
        let folder = Folder::new(name, color, at)?;
        self.state.touch(at)?;
        let id = folder.id;
        self.folders.push(folder);
        Ok(id)
    }
    pub fn add_membership(
        &mut self,
        folder_id: FolderId,
        account_id: AccountId,
        at: Timestamp,
    ) -> Result<MembershipId, AppError> {
        if self.memberships.len() >= Self::MAX_RECORDS
            || !self.folders.iter().any(|f| f.id == folder_id)
            || !self.accounts.iter().any(|a| a.id == account_id)
            || self
                .memberships
                .iter()
                .any(|m| m.folder_id == folder_id && m.account_id == account_id)
        {
            return Err(AppError::InvalidRecord);
        }
        self.state.touch(at)?;
        let id = MembershipId::new();
        self.memberships.push(Membership {
            id,
            state: RecordState::new(at),
            folder_id,
            account_id,
        });
        Ok(id)
    }
    pub fn add_backup_set(
        &mut self,
        account_id: AccountId,
        codes: Vec<BackupCode>,
        at: Timestamp,
    ) -> Result<BackupSetId, AppError> {
        if self.backup_sets.len() >= Self::MAX_RECORDS
            || codes.is_empty()
            || codes.len()
                > Self::MAX_RECORDS.saturating_sub(
                    self.backup_sets
                        .iter()
                        .map(|s| s.codes.len())
                        .sum::<usize>(),
                )
            || !self.accounts.iter().any(|a| a.id == account_id)
            || self.backup_sets.iter().any(|s| s.account_id == account_id)
            || codes
                .iter()
                .any(|c| c.state.created_at > at || c.state.updated_at > at)
            || codes.iter().enumerate().any(|(i, c)| {
                codes[..i]
                    .iter()
                    .any(|p| p.value == c.value || p.id == c.id)
            })
        {
            return Err(AppError::InvalidRecord);
        }
        self.state.touch(at)?;
        let id = BackupSetId::new();
        self.backup_sets.push(BackupCodeSet {
            id,
            state: RecordState::new(at),
            account_id,
            codes,
        });
        Ok(id)
    }

    /// Required after authenticated DTO restoration and before mutation/commit.
    /// No serialization derives intentionally bypass these domain invariants.
    pub fn validate(&self) -> Result<(), AppError> {
        self.state.validate()?;
        self.preferences.validate()?;
        if self.schema_version != Self::SCHEMA_VERSION
            || self.preferences.state.updated_at > self.state.updated_at
            || [
                self.accounts.len(),
                self.folders.len(),
                self.memberships.len(),
                self.backup_sets.len(),
            ]
            .into_iter()
            .any(|n| n > Self::MAX_RECORDS)
        {
            return Err(AppError::InvalidRecord);
        }
        let mut accounts = BTreeSet::new();
        for account in &self.accounts {
            account.validate()?;
            if !accounts.insert(account.id) || account.state.updated_at > self.state.updated_at {
                return Err(AppError::InvalidRecord);
            }
        }
        let mut folders = BTreeSet::new();
        for folder in &self.folders {
            folder.validate()?;
            if !folders.insert(folder.id) || folder.state.updated_at > self.state.updated_at {
                return Err(AppError::InvalidRecord);
            }
        }
        let mut memberships = BTreeSet::new();
        let mut pairs = BTreeSet::new();
        for membership in &self.memberships {
            membership.state.validate()?;
            if !memberships.insert(membership.id)
                || !pairs.insert((membership.folder_id, membership.account_id))
                || !folders.contains(&membership.folder_id)
                || !accounts.contains(&membership.account_id)
                || membership.state.updated_at > self.state.updated_at
            {
                return Err(AppError::InvalidRecord);
            }
        }
        let mut sets = BTreeSet::new();
        let mut owners = BTreeSet::new();
        let mut codes = BTreeSet::new();
        for set in &self.backup_sets {
            set.validate()?;
            if !sets.insert(set.id)
                || !owners.insert(set.account_id)
                || !accounts.contains(&set.account_id)
                || set.state.updated_at > self.state.updated_at
            {
                return Err(AppError::InvalidRecord);
            }
            for code in &set.codes {
                if !codes.insert(code.id) || codes.len() > Self::MAX_RECORDS {
                    return Err(AppError::InvalidRecord);
                }
            }
        }
        Ok(())
    }
}
redacted_debug!(
    AccountDraft,
    Account,
    Folder,
    Membership,
    BackupCode,
    BackupCodeSet,
    Preferences,
    Vault
);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_duplicate_identity_policy_matches_batch_and_vault() -> Result<(), AppError> {
        let a = AccountDraft::new(
            "ÉCOLE",
            None,
            Some("ÜSER"),
            Secret::from_base32("MY")?,
            OtpSettings::default(),
        )?;
        let b = AccountDraft::new(
            "école",
            None,
            Some("üser"),
            Secret::from_base32("M4")?,
            OtpSettings::default(),
        )?;
        assert_eq!(a.duplicate_criteria(&b), (false, true));
        let mut vault = Vault::new(Timestamp(1));
        vault.add_account(a, Timestamp(2))?;
        let matches = vault.duplicates(&b, None);
        assert_eq!(matches.len(), 1);
        assert!(!matches[0].same_seed_and_settings);
        assert!(matches[0].same_service_and_identifier);
        Ok(())
    }

    fn sample() -> Result<Vault, AppError> {
        let mut vault = Vault::new(Timestamp(1));
        let credential = AccountDraft::new(
            "Disposable",
            None,
            Some("alice"),
            Secret::from_base32("MY")?,
            OtpSettings::default(),
        )?;
        let account = vault.add_account(credential, Timestamp(2))?;
        let folder = vault.add_folder("Work", FolderColor::Blue, Timestamp(3))?;
        vault.add_membership(folder, account, Timestamp(4))?;
        vault.add_backup_set(
            account,
            vec![BackupCode::new("disposable", Timestamp(5))?],
            Timestamp(5),
        )?;
        Ok(vault)
    }

    #[test]
    fn restored_records_require_valid_references_unique_ids_and_timestamps() -> Result<(), AppError>
    {
        let good = sample()?;
        good.validate()?;
        let mut invalid = good.clone();
        invalid.schema_version += 1;
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.accounts.push(invalid.accounts[0].clone());
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.folders.push(invalid.folders[0].clone());
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.memberships.push(invalid.memberships[0].clone());
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.memberships[0].folder_id = FolderId::new();
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.memberships[0].account_id = AccountId::new();
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.backup_sets[0].account_id = AccountId::new();
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.backup_sets.push(invalid.backup_sets[0].clone());
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        let duplicate_code = invalid.backup_sets[0].codes[0].clone();
        invalid.backup_sets[0].codes.push(duplicate_code);
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.backup_sets[0].codes.clear();
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.accounts[0].state.updated_at = Timestamp(0);
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.accounts[0].state.deletion = DeletionState::Trashed { at: Timestamp(0) };
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.preferences.lock_timeout_seconds = 0;
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.preferences.clipboard_timeout_seconds = 86_401;
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.backup_sets[0].codes[0].used_at = Some(Timestamp(6));
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.accounts[0].credential.service = " invalid ".into();
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.backup_sets[0].codes[0].value = Zeroizing::new(" invalid ".into());
        assert!(invalid.validate().is_err());
        let mut invalid = good.clone();
        invalid.state.updated_at = Timestamp(4);
        assert!(invalid.validate().is_err());
        let mut invalid = good;
        invalid.accounts = vec![invalid.accounts[0].clone(); Vault::MAX_RECORDS + 1];
        assert!(invalid.validate().is_err());
        Ok(())
    }

    #[test]
    fn edit_preserves_trash_and_duplicate_warning_includes_trash() -> Result<(), AppError> {
        let mut vault = sample()?;
        vault.accounts[0].state.deletion = DeletionState::Trashed { at: Timestamp(2) };
        let draft = vault.accounts[0].credential.clone();
        assert!(vault.duplicates(&draft, None)[0].in_trash);
        let id = vault.accounts[0].id;
        vault.edit_account(id, draft, Timestamp(6))?;
        assert_eq!(
            vault.accounts[0].state.deletion,
            DeletionState::Trashed { at: Timestamp(2) }
        );
        vault.validate()?;
        Ok(())
    }
}
