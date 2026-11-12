//! Ephemeral Account/QR editors. No source image, payload, path or logging.
use crate::{
    domain::{
        otp::{Algorithm, Digits, OtpSettings, Period, Secret},
        records::{Account, AccountDraft, AccountId, DeletionState, RecordState, Vault},
        recovery_input::RecoveryInput,
    },
    error::AppError,
    qr::{ReviewCandidate, ScanReview},
};
use zeroize::{Zeroize, Zeroizing};

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
    pub advanced: bool,
    pub replace_secret: bool,
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
            advanced: false,
            replace_secret: true,
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
            advanced: false,
            replace_secret: true,
        }
    }
    pub fn validate(&self) -> Result<AccountDraft, AppError> {
        self.validate_with_secret(Secret::from_base32(&self.secret)?)
    }
    fn validate_with_secret(&self, secret: Secret) -> Result<AccountDraft, AppError> {
        AccountDraft::new(
            &self.service,
            Some(&self.website),
            Some(&self.account),
            secret,
            OtpSettings {
                algorithm: Algorithm::parse(&self.algorithm)?,
                digits: Digits::parse(&self.digits)?,
                period: Period::parse(&self.period)?,
            },
        )
    }
    /// Shared Account/review field tree. Values/labels/focus come from the same
    /// full-role controls; advanced selections never coerce scanned settings.
    pub fn show(
        &mut self,
        ui: &mut eframe::egui::Ui,
        focus: &mut bool,
        editing: bool,
        show_errors: bool,
    ) -> bool {
        use super::{components as c, theme};
        let error = if show_errors {
            let metadata =
                crate::domain::validation::text(&self.service, 256, AppError::MissingService)
                    .and_then(|_| crate::domain::validation::website(Some(&self.website)))
                    .and_then(|_| {
                        crate::domain::validation::optional_text(
                            Some(&self.account),
                            320,
                            AppError::InvalidAccount,
                        )
                    });
            metadata.err().or_else(|| {
                if self.replace_secret {
                    Secret::from_base32(&self.secret).err()
                } else {
                    None
                }
            })
        } else {
            None
        };
        let field_error = |kind| {
            if error == Some(kind) {
                Some(kind.to_string())
            } else {
                None
            }
        };
        let response = c::labeled_field(
            ui,
            "review-service",
            "Service / website name",
            &mut self.service,
            "Required. Use a recognizable service name.",
            field_error(AppError::MissingService).as_deref(),
        );
        let mut changed = response.changed();
        if *focus {
            response.request_focus();
            *focus = false;
        }
        changed |= c::labeled_field(
            ui,
            "review-website",
            "Website address (optional)",
            &mut self.website,
            "An http or https address. Stored locally; never opened here.",
            field_error(AppError::InvalidWebsite).as_deref(),
        )
        .changed();
        changed |= c::labeled_field(
            ui,
            "review-account",
            "Account identifier (optional)",
            &mut self.account,
            "The service username or email identifying this entry.",
            field_error(AppError::InvalidAccount).as_deref(),
        )
        .changed();
        if editing && !self.replace_secret {
            c::secondary(ui, "Secret key · existing key is kept");
            if c::button(ui, "Change secret key", c::ButtonStyle::Secondary).clicked() {
                self.replace_secret = true;
                self.reveal = false;
                changed = true;
            }
        } else {
            if self.reveal {
                changed |= c::labeled_revealed_secret_field(
                    ui,
                    "review-secret",
                    "Secret key · visible",
                    &mut self.secret,
                    "Keep this key private.",
                    field_error(AppError::InvalidSecret).as_deref(),
                )
                .changed();
            } else {
                changed |= c::labeled_secret_field(
                    ui,
                    "review-secret",
                    "Secret key",
                    &mut self.secret,
                    "Base32 secret from the service. Concealed by default.",
                    field_error(AppError::InvalidSecret).as_deref(),
                )
                .changed();
            }
            if c::button(
                ui,
                if self.reveal {
                    "Conceal secret"
                } else {
                    "Reveal secret"
                },
                c::ButtonStyle::Secondary,
            )
            .clicked()
            {
                self.reveal = !self.reveal;
                c::clear_sensitive_widget_memory(ui.ctx());
            }
            if editing && c::button(ui, "Keep existing secret", c::ButtonStyle::Secondary).clicked()
            {
                self.secret.zeroize();
                self.replace_secret = false;
                self.reveal = false;
                changed = true;
                c::clear_sensitive_widget_memory(ui.ctx());
            }
        }
        c::secondary(
            ui,
            &format!(
                "{} · {} digits · {} seconds",
                *self.algorithm, *self.digits, *self.period
            ),
        );
        if c::button(
            ui,
            if self.advanced {
                "Advanced options · expanded"
            } else {
                "Advanced options · collapsed"
            },
            c::ButtonStyle::Secondary,
        )
        .clicked()
        {
            self.advanced = !self.advanced;
        }
        if self.advanced {
            changed |= c::labeled_choice(
                ui,
                "review-algorithm",
                "Algorithm",
                &mut self.algorithm,
                &["SHA1", "SHA256", "SHA512"],
            );
            changed |= c::labeled_choice(
                ui,
                "review-digits",
                "Code digits",
                &mut self.digits,
                &["6", "8"],
            );
            changed |= c::labeled_choice(
                ui,
                "review-period",
                "Period (seconds)",
                &mut self.period,
                &["15", "30", "60", "120"],
            );
        }
        ui.add_space(theme::SMALL_GAP);
        changed
    }
}
#[derive(Clone, Copy)]
pub enum ReviewMode {
    Import,
    Create,
    Edit {
        id: AccountId,
        expected: RecordState,
    },
}
impl ReviewMode {
    pub fn is_import(self) -> bool {
        matches!(self, Self::Import)
    }
    pub fn is_edit(self) -> bool {
        matches!(self, Self::Edit { .. })
    }
    pub fn exclude(self) -> Option<AccountId> {
        if let Self::Edit { id, .. } = self {
            Some(id)
        } else {
            None
        }
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
    pub mode: ReviewMode,
    pub backup_enabled: bool,
    pub backup_text: Zeroizing<String>,
    pub backup_reveal: bool,
    pub deduplicate: bool,
    pub discard_requested: bool,
    pub backup_review: Option<Result<(usize, usize), AppError>>,
    original: Option<AccountDraft>,
    acknowledged_backup_text: Option<Zeroizing<String>>,
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
            mode: ReviewMode::Import,
            backup_enabled: false,
            backup_text: Zeroizing::new(String::new()),
            backup_reveal: false,
            deduplicate: false,
            discard_requested: false,
            backup_review: None,
            original: None,
            acknowledged_backup_text: None,
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
            mode: ReviewMode::Create,
            backup_enabled: false,
            backup_text: Zeroizing::new(String::new()),
            backup_reveal: false,
            deduplicate: false,
            discard_requested: false,
            backup_review: None,
            original: None,
            acknowledged_backup_text: None,
        }
    }
    pub fn edit(account: &Account) -> Result<Self, AppError> {
        if account.state().deletion != DeletionState::Active {
            return Err(AppError::StaleAccount);
        }
        let d = account.credential();
        let mut review = Self::manual();
        let mut editor = Editor::blank();
        editor.service.push_str(d.service());
        editor.website.push_str(d.website().unwrap_or_default());
        editor
            .account
            .push_str(d.account_identifier().unwrap_or_default());
        *editor.algorithm = match d.settings().algorithm {
            Algorithm::Sha1 => "SHA1",
            Algorithm::Sha256 => "SHA256",
            Algorithm::Sha512 => "SHA512",
        }
        .into();
        *editor.digits = d.settings().digits.count().to_string();
        *editor.period = d.settings().period.seconds().to_string();
        editor.replace_secret = false;
        review.editors = vec![editor];
        review.mode = ReviewMode::Edit {
            id: account.id(),
            expected: account.state(),
        };
        review.original = Some(d.clone());
        Ok(review)
    }
    pub fn drafts(&self, vault: &Vault) -> Result<Vec<AccountDraft>, AppError> {
        if let ReviewMode::Edit { id, expected } = self.mode {
            let account = vault
                .accounts()
                .iter()
                .find(|a| a.id() == id)
                .ok_or(AppError::StaleAccount)?;
            if account.state() != expected || account.state().deletion != DeletionState::Active {
                return Err(AppError::StaleAccount);
            }
            let editor = &self.editors[0];
            Ok(vec![if editor.replace_secret {
                editor.validate()?
            } else {
                editor.validate_with_secret(account.credential().secret().clone())?
            }])
        } else {
            self.approved()
        }
    }
    pub fn save_intent(&self, vault: &Vault) -> Result<super::state::SaveIntent, AppError> {
        let mut drafts = self.drafts(vault)?;
        Ok(match self.mode {
            ReviewMode::Import => super::state::SaveIntent::Import(drafts),
            ReviewMode::Create => super::state::SaveIntent::Create {
                draft: drafts.remove(0),
                codes: if self.backup_enabled {
                    RecoveryInput::parse(&self.backup_text)?
                        .approved(self.backup_duplicates_acknowledged())?
                } else {
                    Vec::new()
                },
            },
            ReviewMode::Edit { id, expected } => super::state::SaveIntent::Edit {
                id,
                expected,
                original: self
                    .original
                    .as_ref()
                    .ok_or(AppError::StaleAccount)?
                    .clone(),
                draft: drafts.remove(0),
                replace_secret: self.editors[0].replace_secret,
            },
        })
    }
    pub fn backup_duplicates_acknowledged(&self) -> bool {
        self.deduplicate
            && self
                .acknowledged_backup_text
                .as_ref()
                .is_some_and(|text| **text == *self.backup_text)
    }
    pub fn acknowledge_backup_duplicates(&mut self, acknowledge: bool) {
        self.deduplicate = acknowledge;
        self.acknowledged_backup_text = acknowledge.then(|| self.backup_text.clone());
        if acknowledge {
            self.clear_backup_error();
        }
    }
    pub fn set_backup_enabled(&mut self, enabled: bool) {
        self.backup_enabled = enabled;
        self.backup_reveal = false;
        if !enabled {
            self.clear_backup_error();
        }
    }
    fn clear_backup_error(&mut self) {
        if self.error.as_ref().is_some_and(|message| {
            [
                AppError::ReviewBackupDuplicates,
                AppError::InvalidBackupCodes,
            ]
            .iter()
            .any(|error| message.starts_with(&error.to_string()))
        }) {
            self.error = None;
            self.show_error = false;
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
    fn duplicate_acknowledgement_and_toggle_clear_only_corrected_backup_errors()
    -> Result<(), AppError> {
        let vault = Vault::new(crate::domain::records::Timestamp(1));
        let mut review = ImportReview::manual();
        review.editors[0].service.push_str("Disposable");
        review.editors[0].secret.push_str("MY");
        review.set_backup_enabled(true);
        review.backup_text.push_str("one\none");
        let failure = review
            .save_intent(&vault)
            .err()
            .ok_or(AppError::InvalidRecord)?;
        assert_eq!(failure, AppError::ReviewBackupDuplicates);
        review.error = Some(format!(
            "{failure} Check every selected account; your edits are kept."
        ));
        review.show_error = true;
        review.acknowledge_backup_duplicates(true);
        assert!(review.error.is_none());
        assert!(!review.show_error);
        assert!(review.save_intent(&vault).is_ok());
        assert_eq!(&**review.backup_text, "one\none");
        assert_eq!(&**review.editors[0].service, "Disposable");
        review.backup_text.push_str("\ntwo\ntwo");
        assert!(!review.backup_duplicates_acknowledged());
        assert!(matches!(
            review.save_intent(&vault),
            Err(AppError::ReviewBackupDuplicates)
        ));
        let metadata = AppError::MissingService.to_string();
        review.editors[0].service.clear();
        review.error = Some(metadata.clone());
        review.acknowledge_backup_duplicates(true);
        assert_eq!(review.error.as_deref(), Some(metadata.as_str()));
        assert!(matches!(
            review.save_intent(&vault),
            Err(AppError::MissingService)
        ));
        review.set_backup_enabled(false);
        assert_eq!(review.error.as_deref(), Some(metadata.as_str()));
        assert!(matches!(
            review.save_intent(&vault),
            Err(AppError::MissingService)
        ));
        review.error = Some(AppError::InvalidBackupCodes.to_string());
        review.set_backup_enabled(false);
        assert!(review.error.is_none());
        Ok(())
    }
    #[test]
    fn scanned_nondefault_settings_remain_visible_with_advanced_collapsed() -> Result<(), AppError>
    {
        let draft = crate::domain::uri::parse_totp_uri(
            "otpauth://totp/Public:fixture?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Public&algorithm=SHA512&digits=8&period=120",
        )?;
        let mut editor = Editor::from_draft(draft, true);
        assert!(!editor.advanced);
        let ctx = eframe::egui::Context::default();
        super::super::theme::apply(&ctx);
        ctx.enable_accesskit();
        let mut output = ctx.run_ui(eframe::egui::RawInput::default(), |root| {
            eframe::egui::CentralPanel::default().show(root, |ui| {
                editor.show(ui, &mut false, false, false);
            });
        });
        output.textures_delta.clear();
        let tree = output
            .platform_output
            .accesskit_update
            .as_ref()
            .ok_or(AppError::InvalidRecord)?;
        assert!(
            tree.nodes
                .iter()
                .any(|(_, node)| node.value() == Some("SHA512 · 8 digits · 120 seconds"))
        );
        assert!(
            tree.nodes
                .iter()
                .all(|(_, node)| node.role() != eframe::egui::Role::ComboBox
                    && node.value().is_none_or(|v| !v.contains("GEZDGNBV")))
        );
        assert_eq!(
            editor.validate()?.settings(),
            OtpSettings {
                algorithm: Algorithm::Sha512,
                digits: Digits::Eight,
                period: Period::Seconds120
            }
        );
        Ok(())
    }
    #[test]
    fn backup_duplicate_acknowledgement_is_bound_to_exact_current_input() -> Result<(), AppError> {
        let vault = Vault::new(crate::domain::records::Timestamp(1));
        let mut review = ImportReview::manual();
        review.editors[0].service.push_str("Disposable");
        review.editors[0].secret.push_str("MY");
        review.backup_enabled = true;
        review.backup_text.push_str("one\none");
        assert!(matches!(
            review.save_intent(&vault),
            Err(AppError::ReviewBackupDuplicates)
        ));
        review.acknowledge_backup_duplicates(true);
        assert!(review.save_intent(&vault).is_ok());
        review.backup_text.push_str("\ntwo\ntwo");
        assert!(!review.backup_duplicates_acknowledged());
        assert!(matches!(
            review.save_intent(&vault),
            Err(AppError::ReviewBackupDuplicates)
        ));
        review.acknowledge_backup_duplicates(true);
        assert!(review.save_intent(&vault).is_ok());
        review.backup_enabled = false;
        assert!(
            matches!(review.save_intent(&vault)?, super::super::state::SaveIntent::Create { codes, .. } if codes.is_empty())
        );
        assert_eq!(&**review.backup_text, "one\none\ntwo\ntwo");
        Ok(())
    }
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
