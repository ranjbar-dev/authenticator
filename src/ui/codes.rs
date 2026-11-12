//! Cleared, derived ID ordering only; the Vault remains the record authority.
use crate::domain::{clock::Clock, records::*};
use crate::error::AppError;
use std::{cmp::Ordering, time::Instant};
use zeroize::Zeroizing;

pub const SORTS: [SortOrder; 6] = [
    SortOrder::LatestFirst,
    SortOrder::EarliestFirst,
    SortOrder::ServiceAscending,
    SortOrder::ServiceDescending,
    SortOrder::AccountAscending,
    SortOrder::AccountDescending,
];
pub const SORT_LABELS: [&str; 6] = [
    "Latest First",
    "Earliest First",
    "Service A > Z",
    "Service Z > A",
    "Account A > Z",
    "Account Z > A",
];
pub fn sort_label(sort: SortOrder) -> &'static str {
    SORT_LABELS[SORTS.iter().position(|v| *v == sort).unwrap_or(0)]
}

/// UTC is sampled once per frame. Copy intentionally creates a NEW sample.
#[derive(Clone, Copy)]
pub struct FrameClock(pub Result<u64, AppError>);
impl Clock for FrameClock {
    fn unix_seconds(&self) -> Result<u64, AppError> {
        self.0
    }
}
#[derive(Default)]
pub struct ClockTracker {
    last: Option<(u64, Instant)>,
    pub shifted: bool,
}
impl ClockTracker {
    pub fn observe(&mut self, seconds: Result<u64, AppError>, now: Instant) {
        if let Ok(seconds) = seconds {
            if let Some((previous, instant)) = self.last {
                let elapsed = now.saturating_duration_since(instant).as_secs();
                if seconds.abs_diff(previous.saturating_add(elapsed)) > 5 {
                    self.shifted = true;
                }
            }
            self.last = Some((seconds, now));
        } else {
            self.last = None;
        }
    }
    pub fn clear(&mut self) {
        self.last = None;
        self.shifted = false;
    }
}
pub fn formatted(value: &str) -> Zeroizing<String> {
    let split = value.len() / 2;
    Zeroizing::new(format!("{} {}", &value[..split], &value[split..]))
}
pub fn copy_value(account: &Account, clock: &dyn Clock) -> Result<Zeroizing<String>, AppError> {
    let draft = account.credential();
    let code = crate::domain::otp::current_code(draft.secret(), draft.settings(), clock)?;
    Ok(Zeroizing::new(code.expose_value().to_owned()))
}
pub fn ordered(vault: &Vault, preferences: &CodesPreferences) -> Vec<AccountId> {
    let query = Zeroizing::new(caseless::default_case_fold_str(&preferences.query));
    // Fold metadata once per list change, never once per comparison or HMAC tick.
    let mut values = vault
        .accounts()
        .iter()
        .filter(|a| a.state().deletion == DeletionState::Active)
        .filter(|a| match preferences.folder {
            FolderFilter::All => true,
            FolderFilter::Unfiled => !vault.memberships().iter().any(|m| {
                m.account_id() == a.id()
                    && vault.folders().iter().any(|f| {
                        f.id() == m.folder_id() && f.state().deletion == DeletionState::Active
                    })
            }),
            FolderFilter::Folder(id) => vault
                .memberships()
                .iter()
                .any(|m| m.account_id() == a.id() && m.folder_id() == id),
        })
        .map(|a| {
            (
                a,
                Zeroizing::new(caseless::default_case_fold_str(a.credential().service())),
                Zeroizing::new(caseless::default_case_fold_str(
                    a.credential().account_identifier().unwrap_or_default(),
                )),
            )
        })
        .filter(|(_, service, account)| {
            query.is_empty() || service.contains(&*query) || account.contains(&*query)
        })
        .collect::<Vec<_>>();
    values.sort_by(|(a, service_a, account_a), (b, service_b, account_b)| {
        let primary = match preferences.sort {
            SortOrder::LatestFirst => b.state().created_at.cmp(&a.state().created_at),
            SortOrder::EarliestFirst => a.state().created_at.cmp(&b.state().created_at),
            SortOrder::ServiceAscending => service_a.cmp(service_b),
            SortOrder::ServiceDescending => service_b.cmp(service_a),
            SortOrder::AccountAscending | SortOrder::AccountDescending => {
                // Missing identifiers always follow named identifiers in either direction.
                match (account_a.is_empty(), account_b.is_empty()) {
                    (true, false) => Ordering::Greater,
                    (false, true) => Ordering::Less,
                    _ if preferences.sort == SortOrder::AccountAscending => {
                        account_a.cmp(account_b)
                    }
                    _ => account_b.cmp(account_a),
                }
            }
        };
        primary.then_with(|| a.id().cmp(&b.id()))
    });
    values.into_iter().map(|(a, _, _)| a.id()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::otp::*;
    #[test]
    fn six_sorts_unicode_query_folders_ties_and_trash() -> Result<(), AppError> {
        let mut v = Vault::new(Timestamp(1));
        let draft = |service, identifier| {
            AccountDraft::new(
                service,
                None,
                identifier,
                Secret::from_base32("MY")?,
                OtpSettings::default(),
            )
        };
        let a = v.add_account(draft("Zulu", Some("alpha"))?, Timestamp(2))?;
        let b = v.add_account(draft("Alpha", Some("zebra"))?, Timestamp(3))?;
        let c = v.add_account(draft("ÉCOLE", Some("ÜSER"))?, Timestamp(4))?;
        let d = v.add_account(draft("Alpha", None)?, Timestamp(5))?;
        let mut p = v.preferences().codes();
        for (sort, expected) in [
            (SortOrder::LatestFirst, vec![d, c, b, a]),
            (SortOrder::EarliestFirst, vec![a, b, c, d]),
            (SortOrder::ServiceAscending, {
                let mut tie = vec![b, d];
                tie.sort();
                tie.extend([a, c]);
                tie
            }),
            (SortOrder::ServiceDescending, {
                let mut tie = vec![b, d];
                tie.sort();
                let mut out = vec![c, a];
                out.extend(tie);
                out
            }),
            (SortOrder::AccountAscending, vec![a, b, c, d]),
            (SortOrder::AccountDescending, vec![c, b, a, d]),
        ] {
            p.sort = sort;
            assert_eq!(ordered(&v, &p), expected);
        }
        p.query = Zeroizing::new("école".into());
        assert_eq!(ordered(&v, &p), vec![c]);
        p.query = Zeroizing::new("üser".into());
        assert_eq!(ordered(&v, &p), vec![c]);
        let folded = v.add_account(draft("Straße", Some("Σς"))?, Timestamp(6))?;
        p.query = Zeroizing::new("STRASSE".into());
        assert_eq!(ordered(&v, &p), vec![folded]);
        p.query = Zeroizing::new("σσ".into());
        assert_eq!(ordered(&v, &p), vec![folded]);
        let state = v.accounts().last().ok_or(AppError::InvalidRecord)?.state();
        v.trash_account(folded, state, Timestamp(6))?;
        p.query = Zeroizing::new("absent".into());
        assert!(ordered(&v, &p).is_empty());
        p.query.clear();
        let f = v.add_folder("Work", FolderColor::Blue, Timestamp(6))?;
        v.add_membership(f, a, Timestamp(7))?;
        p.folder = FolderFilter::Folder(f);
        assert_eq!(ordered(&v, &p), vec![a]);
        p.folder = FolderFilter::Unfiled;
        assert_eq!(ordered(&v, &p), vec![c, b, d]);
        let expected = v.accounts()[0].state();
        v.trash_account(a, expected, Timestamp(8))?;
        p.folder = FolderFilter::All;
        assert!(!ordered(&v, &p).contains(&a));
        assert_eq!(v.memberships().len(), 1);
        assert!(v.trash_account(a, expected, Timestamp(9)).is_err());
        Ok(())
    }
    #[test]
    fn consistent_clock_boundaries_resume_jumps_and_unavailable_never_reuse() -> Result<(), AppError>
    {
        let secret = Secret::from_base32("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")?;
        for algorithm in [Algorithm::Sha1, Algorithm::Sha256, Algorithm::Sha512] {
            for digits in [Digits::Six, Digits::Eight] {
                for period in [
                    Period::Seconds15,
                    Period::Seconds30,
                    Period::Seconds60,
                    Period::Seconds120,
                ] {
                    let settings = OtpSettings {
                        algorithm,
                        digits,
                        period,
                    };
                    let mut vault = Vault::new(Timestamp(0));
                    vault.add_account(
                        AccountDraft::new(
                            "Public clock fixture",
                            None,
                            None,
                            secret.clone(),
                            settings,
                        )?,
                        Timestamp(0),
                    )?;
                    let account = &vault.accounts()[0];
                    let boundary = period.seconds() * 100;
                    for second in [
                        boundary - 1,
                        boundary,
                        boundary + 1,
                        boundary + 3600,
                        boundary - 1000,
                    ] {
                        let code = current_code(&secret, settings, &FrameClock(Ok(second)))?;
                        let raw = copy_value(account, &FrameClock(Ok(second)))?;
                        assert_eq!(&*raw, code.expose_value());
                        assert_eq!(raw.len(), usize::from(digits.count()));
                        assert_eq!(code.step, second / period.seconds());
                        assert_eq!(
                            code.remaining_seconds,
                            period.seconds() - second % period.seconds()
                        );
                        assert_eq!(
                            formatted(code.expose_value()).len(),
                            usize::from(digits.count()) + 1
                        );
                    }
                    assert!(
                        current_code(
                            &secret,
                            settings,
                            &FrameClock(Err(AppError::ClockUnavailable))
                        )
                        .is_err()
                    );
                    assert!(
                        copy_value(account, &FrameClock(Err(AppError::ClockUnavailable))).is_err()
                    );
                }
            }
        }
        let now = Instant::now();
        let mut tracker = ClockTracker::default();
        tracker.observe(Ok(1000), now);
        tracker.observe(Ok(4600), now + std::time::Duration::from_secs(3600));
        assert!(!tracker.shifted);
        tracker.observe(Ok(1000), now + std::time::Duration::from_secs(3601));
        assert!(tracker.shifted);
        tracker.clear();
        tracker.observe(Ok(0), now);
        tracker.observe(Ok(1000), now);
        assert!(tracker.shifted);
        Ok(())
    }
}
