//! Ephemeral editable import data. No source image, payload, path or logging.
use crate::{
    domain::{
        otp::{Algorithm, Digits, OtpSettings, Period, Secret},
        records::AccountDraft,
    },
    error::AppError,
    qr::{ReviewCandidate, ScanReview},
};
use zeroize::Zeroizing;

pub struct Editor {
    pub service: Zeroizing<String>,
    pub website: Zeroizing<String>,
    pub account: Zeroizing<String>,
    pub secret: Zeroizing<String>,
    pub algorithm: Zeroizing<String>,
    pub digits: Zeroizing<String>,
    pub period: Zeroizing<String>,
    pub reveal: bool,
    pub selected: bool,
}
impl Editor {
    pub fn blank() -> Self {
        Self {
            service: Zeroizing::new(String::new()),
            website: Zeroizing::new(String::new()),
            account: Zeroizing::new(String::new()),
            secret: Zeroizing::new(String::new()),
            algorithm: Zeroizing::new("SHA1".into()),
            digits: Zeroizing::new("6".into()),
            period: Zeroizing::new("30".into()),
            reveal: false,
            selected: true,
        }
    }
    fn from_draft(d: AccountDraft, selected: bool) -> Self {
        Self {
            service: Zeroizing::new(d.service().into()),
            website: Zeroizing::new(d.website().unwrap_or_default().into()),
            account: Zeroizing::new(d.account_identifier().unwrap_or_default().into()),
            secret: d.secret().expose_base32(),
            algorithm: Zeroizing::new(
                match d.settings().algorithm {
                    Algorithm::Sha1 => "SHA1",
                    Algorithm::Sha256 => "SHA256",
                    Algorithm::Sha512 => "SHA512",
                }
                .into(),
            ),
            digits: Zeroizing::new(d.settings().digits.count().to_string()),
            period: Zeroizing::new(d.settings().period.seconds().to_string()),
            reveal: false,
            selected,
        }
    }
    pub fn validate(&self) -> Result<AccountDraft, AppError> {
        AccountDraft::new(
            &self.service,
            Some(&self.website),
            Some(&self.account),
            Secret::from_base32(&self.secret)?,
            OtpSettings {
                algorithm: Algorithm::parse(&self.algorithm)?,
                digits: Digits::parse(&self.digits)?,
                period: Period::parse(&self.period)?,
            },
        )
    }
}
pub struct ImportReview {
    pub editors: Vec<Editor>,
    pub choosing: bool,
    pub current: usize,
    pub rejected: usize,
    pub error: Option<String>,
    pub focus: bool,
    pub show_error: bool,
}
impl ImportReview {
    pub fn from_scan(scan: ScanReview) -> Self {
        let choosing = scan.requires_choice();
        let mut rejected = 0;
        let editors = scan
            .into_candidates()
            .into_iter()
            .filter_map(|candidate| match candidate {
                ReviewCandidate::Supported(d) => Some(Editor::from_draft(d, !choosing)),
                _ => {
                    rejected += 1;
                    None
                }
            })
            .collect();
        Self {
            editors,
            choosing,
            current: 0,
            rejected,
            error: None,
            focus: true,
            show_error: false,
        }
    }
    pub fn manual() -> Self {
        Self {
            editors: vec![Editor::blank()],
            choosing: false,
            current: 0,
            rejected: 0,
            error: None,
            focus: true,
            show_error: false,
        }
    }
    /// Revalidate every selected editor, including those not currently visible.
    pub fn approved(&self) -> Result<Vec<AccountDraft>, AppError> {
        let selected = self
            .editors
            .iter()
            .filter(|e| e.selected)
            .map(Editor::validate)
            .collect::<Result<Vec<_>, _>>()?;
        if selected.is_empty() {
            Err(AppError::InvalidRecord)
        } else {
            Ok(selected)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edits_and_selection_revalidate_whole_batch_without_leaking_errors() -> Result<(), AppError> {
        let mut review = ImportReview::manual();
        review.editors[0].service.push_str("Disposable");
        review.editors[0].secret.push_str("MY");
        review.editors[0].algorithm.clear();
        review.editors[0].algorithm.push_str("SHA512");
        review.editors[0].digits.clear();
        review.editors[0].digits.push('8');
        review.editors[0].period.clear();
        review.editors[0].period.push_str("120");
        let approved = review.approved()?;
        assert_eq!(approved[0].settings().algorithm, Algorithm::Sha512);
        assert_eq!(approved[0].settings().digits, Digits::Eight);
        assert_eq!(approved[0].settings().period, Period::Seconds120);
        review
            .editors
            .push(Editor::from_draft(approved[0].clone(), true));
        review.editors[1].period.push_str("invalid");
        assert_eq!(review.approved().err(), Some(AppError::InvalidPeriod));
        assert_eq!(&**review.editors[1].period, "120invalid");
        review.editors[1].selected = false;
        assert_eq!(review.approved()?.len(), 1);
        review.editors[0].service.clear();
        assert_eq!(review.approved().err(), Some(AppError::MissingService));
        Ok(())
    }
}
