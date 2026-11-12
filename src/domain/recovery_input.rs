//! Shared bounded text normalization for account creation and recovery editing.
//! Values never occur in errors or Debug; callers own/zeroize raw input.
use super::records::{BackupCode, Vault};
use crate::error::AppError;
use std::collections::BTreeSet;
use zeroize::Zeroizing;

pub const MAX_TEXT_BYTES: usize = 1024 * 1024;
pub struct RecoveryInput {
    pub values: Vec<Zeroizing<String>>,
    pub duplicates: usize,
}
impl RecoveryInput {
    pub fn parse(text: &str) -> Result<Self, AppError> {
        if text.len() > MAX_TEXT_BYTES {
            return Err(AppError::InvalidBackupCodes);
        }
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let mut seen = BTreeSet::new();
        let mut values = Vec::new();
        let mut duplicates = 0;
        let mut count = 0;
        for line in text.split('\n') {
            // Trim BEFORE domain validation: surrounding tabs are whitespace,
            // internal controls are invalid. CRLF and LF have the same result.
            let value = line.trim();
            if value.is_empty() {
                continue;
            }
            count += 1;
            if count > Vault::MAX_RECORDS {
                return Err(AppError::InvalidBackupCodes);
            }
            let normalized =
                BackupCode::normalized_value(value).map_err(|_| AppError::InvalidBackupCodes)?;
            if seen.insert(value) {
                values.push(normalized);
            } else {
                duplicates += 1;
            }
        }
        if values.is_empty() {
            return Err(AppError::InvalidBackupCodes);
        }
        Ok(Self { values, duplicates })
    }
    pub fn approved(self, deduplicate: bool) -> Result<Vec<Zeroizing<String>>, AppError> {
        if self.duplicates > 0 && !deduplicate {
            Err(AppError::ReviewBackupDuplicates)
        } else {
            Ok(self.values)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalization_is_bounded_exact_and_requires_duplicate_review() -> Result<(), AppError> {
        let parsed = RecoveryInput::parse("\u{feff}\talpha \r\n\n beta\t\nalpha\nAlpha\n")?;
        assert_eq!(parsed.values.len(), 3);
        assert_eq!(parsed.duplicates, 1);
        assert_eq!(&**parsed.values[0], "alpha");
        assert_eq!(&**parsed.values[1], "beta");
        assert_eq!(
            parsed.approved(false).err(),
            Some(AppError::ReviewBackupDuplicates)
        );
        assert_eq!(RecoveryInput::parse("a\na")?.approved(true)?.len(), 1);
        for invalid in [
            String::new(),
            "a\tb".into(),
            "a\rb".into(),
            "x".repeat(4097),
            "x".repeat(MAX_TEXT_BYTES + 1),
            "a\n".repeat(Vault::MAX_RECORDS + 1),
        ] {
            assert_eq!(
                RecoveryInput::parse(&invalid).err(),
                Some(AppError::InvalidBackupCodes)
            );
        }
        assert_eq!(RecoveryInput::parse(&"x".repeat(4096))?.values.len(), 1);
        assert_eq!(
            RecoveryInput::parse(&"a\n".repeat(Vault::MAX_RECORDS))?.duplicates,
            Vault::MAX_RECORDS - 1
        );
        Ok(())
    }
}
