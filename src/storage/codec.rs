//! Fixed schema codec. Every variable length is checked before copying/allocation.
//! No generic deserializer or data-dependent capacity reservation.
use crate::{
    domain::{otp::*, records::*},
    error::AppError,
};
use uuid::Uuid;
use zeroize::Zeroizing;

pub(crate) const MAX_PLAINTEXT: usize = 64 * 1024 * 1024;
type Result<T> = std::result::Result<T, AppError>;

struct Writer(Zeroizing<Vec<u8>>);
impl Writer {
    fn bytes(&mut self, b: &[u8]) -> Result<()> {
        if b.len() > MAX_PLAINTEXT.saturating_sub(self.0.len()) {
            return Err(AppError::InvalidRecord);
        }
        self.0.extend_from_slice(b);
        Ok(())
    }
    fn u8(&mut self, v: u8) -> Result<()> {
        self.bytes(&[v])
    }
    fn u32(&mut self, v: u32) -> Result<()> {
        self.bytes(&v.to_le_bytes())
    }
    fn u64(&mut self, v: u64) -> Result<()> {
        self.bytes(&v.to_le_bytes())
    }
    fn blob(&mut self, b: &[u8]) -> Result<()> {
        self.u32(u32::try_from(b.len()).map_err(|_| AppError::InvalidRecord)?)?;
        self.bytes(b)
    }
    fn text(&mut self, s: &str) -> Result<()> {
        self.blob(s.as_bytes())
    }
    fn optional(&mut self, s: Option<&str>) -> Result<()> {
        self.u8(u8::from(s.is_some()))?;
        if let Some(s) = s {
            self.text(s)?;
        }
        Ok(())
    }
    fn state(&mut self, s: RecordState) -> Result<()> {
        self.u64(s.created_at.0)?;
        self.u64(s.updated_at.0)?;
        match s.deletion {
            DeletionState::Active => self.u8(0),
            DeletionState::Trashed { at } => {
                self.u8(1)?;
                self.u64(at.0)
            }
        }
    }
    fn identity(&mut self, id: Uuid, state: RecordState) -> Result<()> {
        self.bytes(id.as_bytes())?;
        self.state(state)
    }
    fn count(&mut self, n: usize) -> Result<()> {
        self.u32(u32::try_from(n).map_err(|_| AppError::InvalidRecord)?)
    }
}
pub(crate) fn encode(v: &Vault) -> Result<Zeroizing<Vec<u8>>> {
    v.validate()?;
    // Reserve once including AEAD tag space: growth must not leave abandoned
    // plaintext allocations. This is a fixed bounded 64MiB session operation.
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(MAX_PLAINTEXT + 16)
        .map_err(|_| AppError::StorageUnavailable)?;
    let mut w = Writer(Zeroizing::new(buffer));
    w.u32(v.schema_version)?;
    w.identity(v.id.as_uuid(), v.state)?;
    let p = &v.preferences;
    w.identity(p.id.as_uuid(), p.state)?;
    w.u8(p.sort as u8)?;
    w.u8(p.view as u8)?;
    w.u8(u8::from(p.auto_start))?;
    w.u32(p.lock_timeout_seconds)?;
    w.u32(p.clipboard_timeout_seconds)?;
    w.text(&p.query)?;
    match p.folder {
        FolderFilter::All => w.u8(0)?,
        FolderFilter::Unfiled => w.u8(1)?,
        FolderFilter::Folder(id) => {
            w.u8(2)?;
            w.bytes(id.as_uuid().as_bytes())?;
        }
    }
    w.count(v.accounts.len())?;
    for a in &v.accounts {
        w.identity(a.id.as_uuid(), a.state)?;
        let c = &a.credential;
        w.text(c.service())?;
        w.optional(c.website())?;
        w.optional(c.account_identifier())?;
        w.blob(c.secret().expose_bytes())?;
        w.u8(c.settings().algorithm as u8)?;
        w.u8(c.settings().digits as u8)?;
        w.u8(c.settings().period as u8)?;
        w.u8(a.icon as u8)?;
    }
    w.count(v.folders.len())?;
    for f in &v.folders {
        w.identity(f.id.as_uuid(), f.state)?;
        w.text(&f.name)?;
        w.u8(f.color as u8)?;
    }
    w.count(v.memberships.len())?;
    for m in &v.memberships {
        w.identity(m.id.as_uuid(), m.state)?;
        w.bytes(m.folder_id.as_uuid().as_bytes())?;
        w.bytes(m.account_id.as_uuid().as_bytes())?;
    }
    w.count(v.backup_sets.len())?;
    for s in &v.backup_sets {
        w.identity(s.id.as_uuid(), s.state)?;
        w.bytes(s.account_id.as_uuid().as_bytes())?;
        w.count(s.codes.len())?;
        for c in &s.codes {
            w.identity(c.id.as_uuid(), c.state)?;
            w.text(c.expose_value())?;
            w.u8(u8::from(c.used_at.is_some()))?;
            if let Some(at) = c.used_at {
                w.u64(at.0)?;
            }
        }
    }
    Ok(w.0)
}

struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if n > self.0.len() {
            return Err(AppError::InvalidVault);
        }
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        Ok(a)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        self.take(N)?.try_into().map_err(|_| AppError::InvalidVault)
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.array::<1>()?[0])
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.array()?))
    }
    fn boolean(&mut self) -> Result<bool> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(AppError::InvalidVault),
        }
    }
    fn blob(&mut self, max: usize) -> Result<&'a [u8]> {
        let n = self.u32()? as usize;
        if n > max {
            return Err(AppError::InvalidVault);
        }
        self.take(n)
    }
    fn text(&mut self, max: usize) -> Result<&'a str> {
        std::str::from_utf8(self.blob(max)?).map_err(|_| AppError::InvalidVault)
    }
    fn optional(&mut self, max: usize) -> Result<Option<&'a str>> {
        if self.boolean()? {
            Ok(Some(self.text(max)?))
        } else {
            Ok(None)
        }
    }
    fn id(&mut self) -> Result<Uuid> {
        let id = Uuid::from_bytes(self.array()?);
        if id.is_nil() {
            return Err(AppError::InvalidVault);
        }
        Ok(id)
    }
    fn state(&mut self) -> Result<RecordState> {
        let created_at = Timestamp(self.u64()?);
        let updated_at = Timestamp(self.u64()?);
        let deletion = if self.boolean()? {
            DeletionState::Trashed {
                at: Timestamp(self.u64()?),
            }
        } else {
            DeletionState::Active
        };
        let s = RecordState {
            created_at,
            updated_at,
            deletion,
        };
        s.validate()?;
        Ok(s)
    }
    fn count(&mut self, min_bytes: usize) -> Result<usize> {
        let n = self.u32()? as usize;
        if n > Vault::MAX_RECORDS || n > self.0.len() / min_bytes {
            return Err(AppError::InvalidVault);
        }
        Ok(n)
    }
}
pub(crate) fn decode(bytes: &[u8]) -> Result<Vault> {
    if bytes.len() > MAX_PLAINTEXT {
        return Err(AppError::InvalidVault);
    }
    let mut r = Reader(bytes);
    // Schema 1 is the first released schema. There is no legitimate schema 0.
    // Future migrations must transform a detached candidate and validate before commit.
    let schema_version = r.u32()?;
    if !(1..=Vault::SCHEMA_VERSION).contains(&schema_version) {
        return Err(AppError::UnsupportedVault);
    }
    let id = VaultId::from_uuid(r.id()?)?;
    let state = r.state()?;
    let pid = PreferencesId::from_uuid(r.id()?)?;
    let ps = r.state()?;
    let sort = match r.u8()? {
        0 => SortOrder::LatestFirst,
        1 => SortOrder::EarliestFirst,
        2 => SortOrder::ServiceAscending,
        3 => SortOrder::ServiceDescending,
        4 => SortOrder::AccountAscending,
        5 => SortOrder::AccountDescending,
        _ => return Err(AppError::InvalidVault),
    };
    let view = match r.u8()? {
        0 => ViewMode::List,
        1 => ViewMode::Grid,
        _ => return Err(AppError::InvalidVault),
    };
    let preferences = Preferences {
        id: pid,
        state: ps,
        sort,
        view,
        auto_start: r.boolean()?,
        lock_timeout_seconds: r.u32()?,
        clipboard_timeout_seconds: r.u32()?,
        query: if schema_version >= 2 {
            Zeroizing::new(r.text(CodesPreferences::MAX_QUERY_BYTES)?.to_owned())
        } else {
            Zeroizing::new(String::new())
        },
        folder: if schema_version >= 2 {
            match r.u8()? {
                0 => FolderFilter::All,
                1 => FolderFilter::Unfiled,
                2 => FolderFilter::Folder(FolderId::from_uuid(r.id()?)?),
                _ => return Err(AppError::InvalidVault),
            }
        } else {
            FolderFilter::All
        },
    };
    let mut v = Vault {
        id,
        schema_version: Vault::SCHEMA_VERSION,
        state,
        preferences,
        accounts: Vec::new(),
        folders: Vec::new(),
        memberships: Vec::new(),
        backup_sets: Vec::new(),
    };
    for _ in 0..r.count(48)? {
        let id = AccountId::from_uuid(r.id()?)?;
        let state = r.state()?;
        let service = r.text(256)?;
        let website = r.optional(2048)?;
        let identifier = r.optional(320)?;
        let secret =
            Secret::from_bytes(Zeroizing::new(r.blob(Secret::MAX_DECODED_BYTES)?.to_vec()))?;
        let algorithm = match r.u8()? {
            0 => Algorithm::Sha1,
            1 => Algorithm::Sha256,
            2 => Algorithm::Sha512,
            _ => return Err(AppError::InvalidVault),
        };
        let digits = match r.u8()? {
            0 => Digits::Six,
            1 => Digits::Eight,
            _ => return Err(AppError::InvalidVault),
        };
        let period = match r.u8()? {
            0 => Period::Seconds15,
            1 => Period::Seconds30,
            2 => Period::Seconds60,
            3 => Period::Seconds120,
            _ => return Err(AppError::InvalidVault),
        };
        if r.u8()? != 0 {
            return Err(AppError::InvalidVault);
        }
        let credential = AccountDraft::new(
            service,
            website,
            identifier,
            secret,
            OtpSettings {
                algorithm,
                digits,
                period,
            },
        )?;
        // Constructors normalize manual input; persisted data must already be canonical.
        if credential.service() != service
            || credential.website() != website
            || credential.account_identifier() != identifier
        {
            return Err(AppError::InvalidVault);
        }
        v.accounts.push(Account {
            id,
            state,
            credential,
            icon: LocalIcon::Initials,
        });
    }
    for _ in 0..r.count(38)? {
        let id = FolderId::from_uuid(r.id()?)?;
        let state = r.state()?;
        let name = r.text(256)?.to_owned();
        let color = match r.u8()? {
            0 => FolderColor::Blue,
            1 => FolderColor::Purple,
            2 => FolderColor::Pink,
            3 => FolderColor::Red,
            4 => FolderColor::Orange,
            5 => FolderColor::Green,
            _ => return Err(AppError::InvalidVault),
        };
        v.folders.push(Folder {
            id,
            state,
            name,
            color,
        });
    }
    for _ in 0..r.count(65)? {
        let id = MembershipId::from_uuid(r.id()?)?;
        let state = r.state()?;
        let folder_id = FolderId::from_uuid(r.id()?)?;
        let account_id = AccountId::from_uuid(r.id()?)?;
        v.memberships.push(Membership {
            id,
            state,
            folder_id,
            account_id,
        });
    }
    let mut total_codes = 0usize;
    for _ in 0..r.count(53)? {
        let id = BackupSetId::from_uuid(r.id()?)?;
        let state = r.state()?;
        let account_id = AccountId::from_uuid(r.id()?)?;
        let count = r.count(38)?;
        total_codes = total_codes
            .checked_add(count)
            .ok_or(AppError::InvalidVault)?;
        if total_codes > Vault::MAX_RECORDS {
            return Err(AppError::InvalidVault);
        }
        let mut codes = Vec::new();
        for _ in 0..count {
            let id = BackupCodeId::from_uuid(r.id()?)?;
            let state = r.state()?;
            let value = Zeroizing::new(r.text(4096)?.to_owned());
            let used_at = if r.boolean()? {
                Some(Timestamp(r.u64()?))
            } else {
                None
            };
            codes.push(BackupCode {
                id,
                state,
                value,
                used_at,
            });
        }
        v.backup_sets.push(BackupCodeSet {
            id,
            state,
            account_id,
            codes,
        });
    }
    if !r.0.is_empty() {
        return Err(AppError::InvalidVault);
    }
    v.validate().map_err(|_| AppError::InvalidVault)?;
    Ok(v)
}
