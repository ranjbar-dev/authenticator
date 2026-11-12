//! Published RFC 4226/6238 seeds and disposable data only; no real credentials.
use authenticator::{
    domain::{
        clock::{Clock, FixedClock, unix_seconds_at},
        otp::{Algorithm, Digits, OtpSettings, Period, Secret, current_code},
        records::*,
        uri::parse_totp_uri,
    },
    error::AppError,
};
use std::{
    cell::Cell,
    time::{Duration, UNIX_EPOCH},
};
use zeroize::Zeroizing;

fn published_secret() -> Result<Secret, AppError> {
    Secret::from_bytes(Zeroizing::new(b"12345678901234567890".to_vec()))
}
fn draft(service: &str, identifier: Option<&str>) -> Result<AccountDraft, AppError> {
    AccountDraft::new(
        service,
        None,
        identifier,
        published_secret()?,
        OtpSettings::default(),
    )
}

#[test]
fn rfc6238_all_eighteen_vectors() -> Result<(), AppError> {
    let times = [
        59,
        1_111_111_109,
        1_111_111_111,
        1_234_567_890,
        2_000_000_000,
        20_000_000_000,
    ];
    let vectors = [
        (
            Algorithm::Sha1,
            20,
            [
                "94287082", "07081804", "14050471", "89005924", "69279037", "65353130",
            ],
        ),
        (
            Algorithm::Sha256,
            32,
            [
                "46119246", "68084774", "67062674", "91819424", "90698825", "77737706",
            ],
        ),
        (
            Algorithm::Sha512,
            64,
            [
                "90693936", "25091201", "99943326", "93441116", "38618901", "47863826",
            ],
        ),
    ];
    let published = b"1234567890123456789012345678901234567890123456789012345678901234";
    for (algorithm, length, expected) in vectors {
        let secret = Secret::from_bytes(Zeroizing::new(published[..length].to_vec()))?;
        for (time, expected) in times.into_iter().zip(expected) {
            let settings = OtpSettings {
                algorithm,
                digits: Digits::Eight,
                period: Period::Seconds30,
            };
            assert_eq!(
                current_code(&secret, settings, &FixedClock(time))?.expose_value(),
                expected
            );
            let settings = OtpSettings {
                digits: Digits::Six,
                ..settings
            };
            assert_eq!(
                current_code(&secret, settings, &FixedClock(time))?.expose_value(),
                &expected[2..]
            );
        }
    }
    Ok(())
}

#[test]
fn rfc4226_ten_primitive_vectors_through_totp_counter_mapping() -> Result<(), AppError> {
    // TOTP(timestamp = counter * period) exercises the exact HOTP primitive;
    // the product never exposes event-counter credentials or an HOTP API.
    let expected = [
        "755224", "287082", "359152", "969429", "338314", "254676", "287922", "162583", "399871",
        "520489",
    ];
    let secret = published_secret()?;
    for period in [
        Period::Seconds15,
        Period::Seconds30,
        Period::Seconds60,
        Period::Seconds120,
    ] {
        for (counter, expected) in expected.into_iter().enumerate() {
            let settings = OtpSettings {
                period,
                ..OtpSettings::default()
            };
            let timestamp = counter as u64 * period.seconds();
            let code = current_code(&secret, settings, &FixedClock(timestamp))?;
            assert_eq!(code.expose_value(), expected);
            assert_eq!(code.remaining_seconds, period.seconds());
            assert_eq!(code.next_boundary, Some(timestamp + period.seconds()));
            let before = current_code(
                &secret,
                settings,
                &FixedClock(timestamp + period.seconds() - 1),
            )?;
            assert_eq!(before.expose_value(), expected);
            assert_eq!(before.remaining_seconds, 1);
        }
    }
    Ok(())
}

#[test]
fn every_algorithm_digit_period_combination_and_leading_zeros() -> Result<(), AppError> {
    let secret = published_secret()?;
    for algorithm in [Algorithm::Sha1, Algorithm::Sha256, Algorithm::Sha512] {
        for digits in [Digits::Six, Digits::Eight] {
            let reference = current_code(
                &secret,
                OtpSettings {
                    algorithm,
                    digits,
                    period: Period::Seconds30,
                },
                &FixedClock(300),
            )?;
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
                let code = current_code(&secret, settings, &FixedClock(10 * period.seconds()))?;
                assert_eq!(code.expose_value(), reference.expose_value());
                assert_eq!(code.expose_value().len(), usize::from(digits.count()));
                assert!(code.expose_value().bytes().all(|b| b.is_ascii_digit()));
                assert_eq!(code.step, 10);
            }
        }
    }
    let six = current_code(&secret, OtpSettings::default(), &FixedClock(1_111_111_109))?;
    assert_eq!(six.expose_value(), "081804");
    Ok(())
}

struct MutableClock(Cell<u64>);
impl Clock for MutableClock {
    fn unix_seconds(&self) -> Result<u64, AppError> {
        Ok(self.0.get())
    }
}
struct FailedClock;
impl Clock for FailedClock {
    fn unix_seconds(&self) -> Result<u64, AppError> {
        Err(AppError::ClockUnavailable)
    }
}

#[test]
fn boundary_resume_backward_forward_jumps_and_overflow_are_recomputed() -> Result<(), AppError> {
    let secret = published_secret()?;
    let clock = MutableClock(Cell::new(29));
    let settings = OtpSettings::default();
    assert_eq!(
        current_code(&secret, settings, &clock)?.expose_value(),
        "755224"
    );
    for (now, expected) in [
        (30, "287082"),
        (300, "403154"),
        (0, "755224"),
        (60, "359152"),
    ] {
        clock.0.set(now);
        assert_eq!(
            current_code(&secret, settings, &clock)?.expose_value(),
            expected
        );
    }
    for period in [
        Period::Seconds15,
        Period::Seconds30,
        Period::Seconds60,
        Period::Seconds120,
    ] {
        let code = current_code(
            &secret,
            OtpSettings { period, ..settings },
            &FixedClock(u64::MAX),
        )?;
        assert_eq!(code.step, u64::MAX / period.seconds());
        assert!(code.next_boundary.is_none());
        assert!((1..=period.seconds()).contains(&code.remaining_seconds));
    }
    assert!(matches!(
        current_code(&secret, settings, &FailedClock),
        Err(AppError::ClockUnavailable)
    ));
    Ok(())
}

#[test]
fn utc_conversion_rejects_pre_epoch_and_ignores_local_offset() -> Result<(), AppError> {
    assert_eq!(unix_seconds_at(UNIX_EPOCH)?, 0);
    assert_eq!(
        unix_seconds_at(UNIX_EPOCH + Duration::from_millis(59_999))?,
        59
    );
    assert_eq!(
        unix_seconds_at(UNIX_EPOCH - Duration::from_secs(1)),
        Err(AppError::ClockUnavailable)
    );
    // UTC 00:00:59 == Tehran 03:30:59 == UTC-08 16:00:59 on previous day.
    // The API accepts only the underlying instant, not civil time or offsets.
    let instant = UNIX_EPOCH + Duration::from_secs(59);
    let secret = published_secret()?;
    for offset in [0_i64, 12_600, -28_800] {
        let civil_seconds = 59_i64 + offset;
        let utc_seconds =
            u64::try_from(civil_seconds - offset).map_err(|_| AppError::ClockUnavailable)?;
        assert_eq!(utc_seconds, unix_seconds_at(instant)?);
        assert_eq!(
            current_code(&secret, OtpSettings::default(), &FixedClock(utc_seconds))?.expose_value(),
            "287082"
        );
    }
    Ok(())
}

#[test]
fn base32_normalization_and_strict_padding() -> Result<(), AppError> {
    let canonical = Secret::from_base32("MZXW6YTBOI")?; // RFC 4648 'foobar'.
    for input in ["mzxw6ytboi", "MZXW6YTBOI======", " mzxw\t6ytb\r\noi====== "] {
        let parsed = Secret::from_base32(input)?;
        assert!(parsed.same_seed(&canonical));
        assert_eq!(parsed.expose_base32().as_str(), "MZXW6YTBOI");
    }
    for (encoded, decoded) in [
        ("MY======", "f"),
        ("MZXQ====", "fo"),
        ("MZXW6===", "foo"),
        ("MZXW6YQ=", "foob"),
        ("MZXW6YTB", "fooba"),
    ] {
        assert_eq!(
            Secret::from_base32(encoded)?.expose_bytes(),
            decoded.as_bytes()
        );
    }
    for input in [
        "",
        " \t\r\n",
        "A",
        "AAA",
        "AAAAAA",
        "MZ",
        "MZ======",
        "MY=====",
        "MY=======",
        "MY======A",
        "=MY=====",
        "MY======MY======",
        "M0",
        "M1",
        "M8",
        "M9",
        "M-Y",
        "M_Y",
        "M\0Y",
        "M\u{00a0}Y",
        "M\u{200b}Y",
        "M\x0bY",
        "M\x0cY",
        "MY====é=",
    ] {
        assert!(matches!(
            Secret::from_base32(input),
            Err(AppError::InvalidSecret)
        ));
    }
    assert!(Secret::from_base32(&"A".repeat(4097)).is_err());
    assert!(Secret::from_base32(&data_encoding::BASE32_NOPAD.encode(&vec![0; 1025])).is_err());
    assert!(Secret::from_bytes(Zeroizing::new(Vec::new())).is_err());
    assert!(Secret::from_bytes(Zeroizing::new(vec![0; 1025])).is_err());
    Ok(())
}

#[test]
fn valid_uri_defaults_percent_encoding_and_issuer_rules() -> Result<(), AppError> {
    let base = "otpauth://totp/Example%20Service%3Aalice%2Btest%40example.com?secret=MZXW6YTBOI&issuer=Example%20Service&algorithm=SHA512&digits=8&period=120";
    let imported = parse_totp_uri(base)?;
    assert_eq!(imported.service(), "Example Service");
    assert_eq!(
        imported.account_identifier(),
        Some("alice+test@example.com")
    );
    assert_eq!(
        imported.settings(),
        OtpSettings {
            algorithm: Algorithm::Sha512,
            digits: Digits::Eight,
            period: Period::Seconds120
        }
    );
    assert_eq!(imported.secret().expose_bytes(), b"foobar");
    for (uri, service, account) in [
        ("otpauth://totp/Example?secret=MY", "Example", None),
        (
            "otpauth://totp/Example:alice?secret=MY",
            "Example",
            Some("alice"),
        ),
        (
            "otpauth://totp/alice?issuer=Example&secret=MY",
            "Example",
            Some("alice"),
        ),
        (
            "OTPAUTH://TOTP/Example:%20alice?secret=MY&issuer=Example",
            "Example",
            Some("alice"),
        ),
        (
            "otpauth://totp/%E6%B5%8B%E8%AF%95:alice?secret=MY",
            "测试",
            Some("alice"),
        ),
        (
            "otpauth://totp/A+B:alice+test?secret=MY",
            "A+B",
            Some("alice+test"),
        ),
    ] {
        let parsed = parse_totp_uri(uri)?;
        assert_eq!(parsed.service(), service);
        assert_eq!(parsed.account_identifier(), account);
        assert_eq!(parsed.settings(), OtpSettings::default());
    }
    for algorithm in ["SHA1", "SHA256", "SHA512", "sha256"] {
        for digits in ["6", "8"] {
            for period in ["15", "30", "60", "120"] {
                let uri = format!(
                    "otpauth://totp/Example?secret=MY&algorithm={algorithm}&digits={digits}&period={period}"
                );
                let parsed = parse_totp_uri(&uri)?;
                assert_eq!(
                    parsed.settings(),
                    OtpSettings {
                        algorithm: Algorithm::parse(algorithm)?,
                        digits: Digits::parse(digits)?,
                        period: Period::parse(period)?
                    }
                );
            }
        }
    }
    Ok(())
}

#[test]
fn malformed_and_ambiguous_uri_inputs_never_create_a_draft() {
    for input in [
        "",
        "https://totp/a?secret=MY",
        "otpauth://hotp/a?secret=MY&counter=0",
        "otpauth://steam/a?secret=MY",
        "otpauth:/totp/a?secret=MY",
        "otpauth://totp",
        "otpauth://totp/?secret=MY",
        "otpauth://totp/a/b?secret=MY",
        "otpauth://totp:80/a?secret=MY",
        "otpauth://u@totp/a?secret=MY",
        "otpauth://totp/a?secret=MY#fragment",
        "otpauth://totp/a?secret=MY&",
        "otpauth://totp/a?secret",
        "otpauth://totp/a?=MY",
        "otpauth://totp/a?secret=MY?issuer=a",
        "otpauth://totp/a?issuer=Example",
        "otpauth://totp/a?secret=",
        "otpauth://totp/a?secret=MZ",
        "otpauth://totp/a?secret=M0",
        "otpauth://totp/a?secret=MY&secret=MY",
        "otpauth://totp/a?secret=MY&%73ecret=MY",
        "otpauth://totp/a?secret=MY&SECRET=MY",
        "otpauth://totp/a?secret=MY&issuer=X&issuer=X",
        "otpauth://totp/a?secret=MY&algorithm=SHA1&algorithm=SHA256",
        "otpauth://totp/a?secret=MY&digits=6&digits=8",
        "otpauth://totp/a?secret=MY&period=30&period=60",
        "otpauth://totp/a?secret=MY&counter=0",
        "otpauth://totp/a?secret=MY&image=https%3A%2F%2Fexample.com",
        "otpauth://totp/a?secret=MY&foo=bar",
        "otpauth://totp/a?secret=MY&algorithm=MD5",
        "otpauth://totp/a?secret=MY&algorithm=SHA-1",
        "otpauth://totp/a?secret=MY&algorithm=",
        "otpauth://totp/a?secret=MY&digits=7",
        "otpauth://totp/a?secret=MY&digits=06",
        "otpauth://totp/a?secret=MY&digits=256",
        "otpauth://totp/a?secret=MY&period=0",
        "otpauth://totp/a?secret=MY&period=-30",
        "otpauth://totp/a?secret=MY&period=030",
        "otpauth://totp/a?secret=MY&period=18446744073709551616",
        "otpauth://totp/X:alice?secret=MY&issuer=Y",
        "otpauth://totp/X:alice?secret=MY&issuer=x",
        "otpauth://totp/:alice?secret=MY",
        "otpauth://totp/X:?secret=MY",
        "otpauth://totp/X:a:b?secret=MY",
        "otpauth://totp/a%?secret=MY",
        "otpauth://totp/a%2?secret=MY",
        "otpauth://totp/a%GG?secret=MY",
        "otpauth://totp/%FF?secret=MY",
        "otpauth://totp/a?secret=MY%FF",
        "otpauth://totp/a%00?secret=MY",
        "otpauth://totp/a%0A?secret=MY",
        "otpauth://totp/a?secret=MY&issuer=%09X",
        "otpauth://totp/a?secret=MY\n",
        " otpauth://totp/a?secret=MY",
        "otpauth://totp/A B?secret=MY",
        "otpauth://totp/测试?secret=MY",
        "otpauth://totp/a\\b?secret=MY",
    ] {
        assert!(parse_totp_uri(input).is_err());
    }
    assert!(parse_totp_uri(&format!("otpauth://totp/{}?secret=MY", "a".repeat(8192))).is_err());
}

#[test]
fn validation_errors_are_actionable_fixed_and_seed_safe() -> Result<(), AppError> {
    for (suffix, expected) in [
        ("algorithm=MD5", AppError::InvalidAlgorithm),
        ("digits=7", AppError::InvalidDigits),
        ("period=0", AppError::InvalidPeriod),
        ("secret=MY", AppError::DuplicateParameter),
        ("foo=bar", AppError::UnsupportedParameter),
    ] {
        let result = parse_totp_uri(&format!("otpauth://totp/Example?secret=MY&{suffix}"));
        assert!(matches!(result, Err(error) if error == expected));
    }
    assert!(matches!(draft("", None), Err(AppError::MissingService)));
    assert!(matches!(draft("  ", None), Err(AppError::MissingService)));
    assert!(draft("bad\nname", None).is_err());
    assert!(draft(&"s".repeat(257), None).is_err());
    assert!(draft("Example", Some("bad\0identifier")).is_err());
    assert!(draft("Example", Some(&"x".repeat(321))).is_err());
    for website in [
        "javascript:alert(1)",
        "file:///x",
        "example.com",
        "https:example.com",
        "https:///example.com",
        "https://u:p@example.com",
        "https://example.com/a b",
        "https://example.com/%XX",
        "https://example.com\\x",
        "https://",
        "https://example.com\n",
    ] {
        assert!(matches!(
            AccountDraft::new(
                "Example",
                Some(website),
                None,
                published_secret()?,
                OtpSettings::default()
            ),
            Err(AppError::InvalidWebsite)
        ));
    }
    let valid = AccountDraft::new(
        " Example ",
        Some("https://example.com/path?q=x%20y"),
        Some(" alice "),
        published_secret()?,
        OtpSettings::default(),
    )?;
    assert_eq!(valid.service(), "Example");
    assert_eq!(valid.account_identifier(), Some("alice"));
    assert_eq!(valid.website(), Some("https://example.com/path?q=x%20y"));
    Ok(())
}

#[test]
fn stable_identity_timestamps_associations_and_nonblocking_duplicates() -> Result<(), AppError> {
    let mut vault = Vault::new(Timestamp(100));
    assert_eq!(vault.schema_version(), 2);
    assert_eq!(vault.preferences().lock_timeout_seconds(), 300);
    assert_eq!(vault.preferences().clipboard_timeout_seconds(), 30);
    assert!(!vault.preferences().auto_start());
    let first = vault.add_account(draft("Example", Some("Alice"))?, Timestamp(101))?;
    let duplicate = draft(" example ", Some("alice"))?;
    let warnings = vault.duplicates(&duplicate, None);
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].same_service_and_identifier);
    assert!(warnings[0].same_seed_and_settings);
    assert!(vault.duplicates(&duplicate, Some(first)).is_empty());
    let second = vault.add_account(duplicate, Timestamp(102))?;
    assert_ne!(first, second); // Warning does not silently overwrite or block Save.
    let folder1 = vault.add_folder("Work", FolderColor::Blue, Timestamp(103))?;
    let folder2 = vault.add_folder("Personal", FolderColor::Green, Timestamp(104))?;
    let membership = vault.add_membership(folder1, first, Timestamp(105))?;
    vault.add_membership(folder2, first, Timestamp(106))?;
    assert!(
        vault
            .add_membership(folder1, first, Timestamp(106))
            .is_err()
    );
    assert!(
        vault
            .add_membership(FolderId::new(), first, Timestamp(106))
            .is_err()
    );
    let mut recovery = BackupCode::new("disposable-recovery", Timestamp(106))?;
    let recovery_id = recovery.id();
    recovery.set_used(true, Timestamp(107))?;
    assert_eq!(recovery.used_at(), Some(Timestamp(107)));
    recovery.set_used(false, Timestamp(108))?;
    assert_eq!(recovery.used_at(), None);
    assert_eq!(recovery.id(), recovery_id);
    let set_id = vault.add_backup_set(first, vec![recovery], Timestamp(109))?;
    assert!(
        vault
            .add_backup_set(
                first,
                vec![BackupCode::new("other", Timestamp(109))?],
                Timestamp(109)
            )
            .is_err()
    );
    let replacement = AccountDraft::new(
        "Renamed",
        Some("https://example.com"),
        Some("new-identifier"),
        Secret::from_base32("MZXW6YTBOI")?,
        OtpSettings {
            algorithm: Algorithm::Sha512,
            digits: Digits::Eight,
            period: Period::Seconds120,
        },
    )?;
    vault.edit_account(first, replacement.clone(), Timestamp(110))?;
    let edited = &vault.accounts()[0];
    assert_eq!(edited.id(), first);
    assert_eq!(edited.state().created_at, Timestamp(101));
    assert_eq!(edited.state().updated_at, Timestamp(110));
    assert_eq!(edited.credential().service(), "Renamed");
    assert_eq!(vault.memberships()[0].id(), membership);
    assert_eq!(vault.memberships()[0].account_id(), first);
    assert_eq!(vault.backup_sets()[0].id(), set_id);
    assert_eq!(vault.backup_sets()[0].account_id(), first);
    assert_eq!(vault.backup_sets()[0].codes()[0].id(), recovery_id);
    vault.edit_account(first, draft("Corrected clock", None)?, Timestamp(99))?;
    assert_eq!(
        vault.accounts()[0].credential().service(),
        "Corrected clock"
    );
    assert_eq!(vault.accounts()[0].state().created_at, Timestamp(101));
    assert_eq!(vault.accounts()[0].state().updated_at, Timestamp(110));
    assert_eq!(vault.state().updated_at, Timestamp(110));
    assert!(
        vault
            .edit_account(AccountId::new(), replacement, Timestamp(111))
            .is_err()
    );
    assert!(AccountId::from_uuid(uuid::Uuid::nil()).is_err());
    assert_eq!(AccountId::from_uuid(first.as_uuid())?, first);
    Ok(())
}

#[test]
fn duplicate_reasons_are_independent_and_settings_preserved() -> Result<(), AppError> {
    let mut vault = Vault::new(Timestamp(0));
    vault.add_account(draft("Example", Some("alice"))?, Timestamp(0))?;
    let seed_alias = draft("Different Service", Some("different"))?;
    let matches = vault.duplicates(&seed_alias, None);
    assert!(matches[0].same_seed_and_settings);
    assert!(!matches[0].same_service_and_identifier);
    let metadata = AccountDraft::new(
        "Example",
        None,
        Some("alice"),
        Secret::from_base32("MY")?,
        OtpSettings::default(),
    )?;
    let matches = vault.duplicates(&metadata, None);
    assert!(!matches[0].same_seed_and_settings);
    assert!(matches[0].same_service_and_identifier);
    let other_settings = AccountDraft::new(
        "Different",
        None,
        None,
        published_secret()?,
        OtpSettings {
            period: Period::Seconds60,
            ..OtpSettings::default()
        },
    )?;
    assert!(vault.duplicates(&other_settings, None).is_empty());
    Ok(())
}

#[test]
fn all_sensitive_debug_and_error_formatting_is_redacted() -> Result<(), AppError> {
    let seed = published_secret()?;
    let draft = draft("private-service", Some("private-identifier"))?;
    let account = Account::new(draft.clone(), Timestamp(0));
    let backup = BackupCode::new("private-recovery-code", Timestamp(0))?;
    let mut vault = Vault::new(Timestamp(0));
    let id = vault.add_account(draft.clone(), Timestamp(0))?;
    vault.add_backup_set(id, vec![backup.clone()], Timestamp(0))?;
    let code = current_code(&seed, OtpSettings::default(), &FixedClock(59))?;
    let debug = format!(
        "{seed:?} {draft:?} {account:?} {backup:?} {vault:?} {code:?} {:?}",
        vault.backup_sets()
    );
    for forbidden in [
        "12345678901234567890",
        "GEZDGNBV",
        "private-service",
        "private-identifier",
        "private-recovery-code",
        "287082",
    ] {
        assert!(!debug.contains(forbidden));
    }
    for error in [
        AppError::InvalidSecret,
        AppError::MissingService,
        AppError::InvalidAccount,
        AppError::InvalidWebsite,
        AppError::InvalidUri,
        AppError::UnsupportedOtpType,
        AppError::UnsupportedParameter,
        AppError::DuplicateParameter,
        AppError::ConflictingIssuer,
        AppError::InvalidAlgorithm,
        AppError::InvalidDigits,
        AppError::InvalidPeriod,
        AppError::InvalidRecord,
        AppError::ClockUnavailable,
    ] {
        let formatted = format!("{error} {error:?}");
        assert!(!formatted.contains("private"));
        assert!(!formatted.contains("GEZDGNBV"));
        assert!(!formatted.contains("287082"));
    }
    Ok(())
}
