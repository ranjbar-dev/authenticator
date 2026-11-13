use super::{
    folder_edit::*,
    otp::{OtpSettings, Secret},
    records::*,
};
use crate::{error::AppError, ui::codes};
use std::collections::BTreeSet;

fn fixture() -> Result<Vault, AppError> {
    let mut v = Vault::new(Timestamp(100));
    for i in 0..3 {
        let id = v.add_account(
            AccountDraft::new(
                &format!("Public folder account {i}"),
                None,
                Some("public@fixture.invalid"),
                Secret::from_base32("MY")?,
                OtpSettings::default(),
            )?,
            Timestamp(100),
        )?;
        let mut used = BackupCode::new(&format!("public-used-{i}"), Timestamp(100))?;
        used.set_used(true, Timestamp(100))?;
        v.add_backup_set(id, vec![used], Timestamp(100))?;
    }
    Ok(v)
}
fn save(
    v: &mut Vault,
    target: &FolderTarget,
    name: &str,
    color: FolderColor,
    selected: &[AccountId],
    consent: bool,
) -> Result<FolderId, AppError> {
    v.save_folder(
        target,
        FolderUpdate {
            name,
            color,
            selected: &selected.iter().copied().collect(),
            duplicate_consent: consent,
        },
        Timestamp(1),
    )
}
fn member(v: &Vault, folder: FolderId, account: AccountId) -> Result<&Membership, AppError> {
    v.memberships()
        .iter()
        .find(|m| m.folder_id() == folder && m.account_id() == account)
        .ok_or(AppError::InvalidRecord)
}
#[test]
fn folder_names_byte_boundaries_six_colors_exact_duplicates_and_trash() -> Result<(), AppError> {
    for invalid in [
        "",
        "   ",
        "a\nb",
        "a\0b",
        &"a".repeat(257),
        &"é".repeat(129),
    ] {
        assert_eq!(normalized_name(invalid), Err(AppError::InvalidFolderName));
    }
    assert_eq!(normalized_name(&"é".repeat(128))?, "é".repeat(128));
    assert_eq!(normalized_name("  é folder  ")?, "é folder");
    let mut v = fixture()?;
    for (i, color) in crate::ui::folders::COLORS.into_iter().enumerate() {
        let target = FolderTarget::capture(&v, None)?;
        let id = save(&mut v, &target, &format!(" Public {i} "), color, &[], false)?;
        let f = v
            .folders()
            .iter()
            .find(|f| f.id() == id)
            .ok_or(AppError::InvalidRecord)?;
        assert_eq!(f.name(), format!("Public {i}"));
        assert_eq!(f.color(), color);
        assert_eq!(f.state().created_at, Timestamp(100));
    }
    let target = FolderTarget::capture(&v, None)?;
    assert_eq!(
        save(&mut v, &target, "Public 0", FolderColor::Pink, &[], false),
        Err(AppError::ReviewFolderDuplicates)
    );
    target.check(&v)?;
    let duplicate = save(&mut v, &target, "Public 0", FolderColor::Pink, &[], true)?;
    let target = FolderTarget::capture(&v, Some(duplicate))?;
    assert_eq!(
        save(&mut v, &target, "Public 1", FolderColor::Blue, &[], false),
        Err(AppError::ReviewFolderDuplicates)
    );
    v.trash_folder(&target, Timestamp(100))?;
    let target = FolderTarget::capture(&v, None)?;
    assert_eq!(duplicate_names(&v, "Public 0", None).len(), 2);
    save(&mut v, &target, "public 0", FolderColor::Green, &[], false)?;
    v.validate()
}
#[test]
fn folder_many_to_many_reuses_links_preserves_hidden_trash_and_all_credentials()
-> Result<(), AppError> {
    let mut v = fixture()?;
    let ids: Vec<_> = v.accounts().iter().map(Account::id).collect();
    let before = v.clone(); // pure generated fixture oracle, never a UI/store cache
    let target = FolderTarget::capture(&v, None)?;
    let f = save(
        &mut v,
        &target,
        "Work",
        FolderColor::Blue,
        &[ids[0], ids[1]],
        false,
    )?;
    let target = FolderTarget::capture(&v, None)?;
    let g = save(
        &mut v,
        &target,
        "Home",
        FolderColor::Green,
        &[ids[0]],
        false,
    )?;
    let original_link = member(&v, f, ids[0])?.id();
    v.trash_account(ids[1], v.accounts()[1].state(), Timestamp(100))?;
    let hidden = member(&v, f, ids[1])?.state();
    let target = FolderTarget::capture(&v, Some(f))?;
    assert_eq!(target.hidden_links(), 1);
    assert_eq!(target.selected(), BTreeSet::from([ids[0]]));
    save(&mut v, &target, "Renamed", FolderColor::Orange, &[], false)?;
    assert_eq!(member(&v, f, ids[0])?.id(), original_link);
    assert!(matches!(
        member(&v, f, ids[0])?.state().deletion,
        DeletionState::Trashed { .. }
    ));
    assert_eq!(member(&v, f, ids[1])?.state(), hidden);
    let target = FolderTarget::capture(&v, Some(f))?;
    save(
        &mut v,
        &target,
        "Renamed",
        FolderColor::Red,
        &[ids[0], ids[2]],
        false,
    )?;
    assert_eq!(member(&v, f, ids[0])?.id(), original_link);
    assert_eq!(
        member(&v, f, ids[0])?.state().deletion,
        DeletionState::Active
    );
    assert_eq!(v.memberships().len(), 4);
    let mut p = v.preferences().codes();
    p.folder = FolderFilter::Folder(g);
    assert_eq!(codes::ordered(&v, &p), vec![ids[0]]);
    p.folder = FolderFilter::Folder(f);
    assert_eq!(codes::ordered(&v, &p).len(), 2);
    // No-op does not allocate pairs or change retained link state.
    let retained = member(&v, f, ids[0])?.state();
    let target = FolderTarget::capture(&v, Some(f))?;
    save(
        &mut v,
        &target,
        "Renamed",
        FolderColor::Red,
        &[ids[0], ids[2]],
        false,
    )?;
    assert_eq!(v.memberships().len(), 4);
    assert_eq!(member(&v, f, ids[0])?.state(), retained);
    p.folder = FolderFilter::Folder(f);
    v.set_codes_preferences(p, Timestamp(100))?;
    let links: Vec<_> = v
        .memberships()
        .iter()
        .map(|m| (m.id(), m.state(), m.account_id(), m.folder_id()))
        .collect();
    let target = FolderTarget::capture(&v, Some(f))?;
    v.trash_folder(&target, Timestamp(1))?;
    assert_eq!(v.preferences().codes().folder, FolderFilter::All);
    assert_eq!(
        links,
        v.memberships()
            .iter()
            .map(|m| (m.id(), m.state(), m.account_id(), m.folder_id()))
            .collect::<Vec<_>>()
    );
    for (a, old) in v.accounts().iter().zip(before.accounts()) {
        assert_eq!(a.id(), old.id());
        assert_eq!(a.credential().service(), old.credential().service());
        assert_eq!(
            a.credential().account_identifier(),
            old.credential().account_identifier()
        );
        assert!(a.credential().duplicate_criteria(old.credential()).0);
    }
    for (s, old) in v.backup_sets().iter().zip(before.backup_sets()) {
        assert_eq!(s.id(), old.id());
        assert_eq!(s.state(), old.state());
        assert_eq!(s.account_id(), old.account_id());
        for (c, old) in s.codes().iter().zip(old.codes()) {
            assert_eq!(
                (c.id(), c.state(), c.used_at(), c.expose_value()),
                (old.id(), old.state(), old.used_at(), old.expose_value())
            );
        }
    }
    v.validate()
}
#[test]
fn folder_filters_respect_link_and_parent_trash_and_surviving_restoration() -> Result<(), AppError>
{
    let mut v = fixture()?;
    let ids: Vec<_> = v.accounts().iter().map(Account::id).collect();
    let f = v.add_folder("Public", FolderColor::Blue, Timestamp(100))?;
    v.add_membership(f, ids[0], Timestamp(100))?;
    v.add_membership(f, ids[1], Timestamp(100))?;
    v.memberships[1].state.deletion = DeletionState::Trashed { at: Timestamp(100) };
    let mut p = v.preferences().codes();
    let assert_scope =
        |v: &Vault, p: &mut CodesPreferences, expected: &[AccountId], unfiled: &[AccountId]| {
            p.folder = FolderFilter::Folder(f);
            assert_eq!(
                codes::ordered(v, p).into_iter().collect::<BTreeSet<_>>(),
                expected.iter().copied().collect()
            );
            p.folder = FolderFilter::Unfiled;
            assert_eq!(
                codes::ordered(v, p).into_iter().collect::<BTreeSet<_>>(),
                unfiled.iter().copied().collect()
            );
        };
    assert_scope(&v, &mut p, &[ids[0]], &[ids[1], ids[2]]);
    v.folders[0].state.deletion = DeletionState::Trashed { at: Timestamp(100) };
    assert_scope(&v, &mut p, &[], &ids);
    v.folders[0].state.deletion = DeletionState::Active;
    v.trash_account(ids[0], v.accounts()[0].state(), Timestamp(100))?;
    assert_scope(&v, &mut p, &[], &[ids[1], ids[2]]);
    // Backend fixture only: Task18 owns actual restore UI/transaction.
    v.accounts[0].state.deletion = DeletionState::Active;
    assert_scope(&v, &mut p, &[ids[0]], &[ids[1], ids[2]]);
    v.memberships[1].state.deletion = DeletionState::Active;
    assert_scope(&v, &mut p, &[ids[0], ids[1]], &[ids[2]]);
    p.folder = FolderFilter::All;
    assert_eq!(codes::ordered(&v, &p).len(), 3);
    v.validate()
}
#[test]
fn folder_equal_time_guards_reject_content_links_and_reviewed_account_changes()
-> Result<(), AppError> {
    let mut v = fixture()?;
    let id = v.accounts()[0].id();
    let f = v.add_folder("Public", FolderColor::Blue, Timestamp(100))?;
    v.add_membership(f, id, Timestamp(100))?;
    let target = FolderTarget::capture(&v, Some(f))?;
    v.folders[0].color = FolderColor::Pink;
    assert_eq!(target.check(&v), Err(AppError::StaleFolder));
    v.folders[0].color = FolderColor::Blue;
    v.memberships[0].state.deletion = DeletionState::Trashed { at: Timestamp(100) };
    assert_eq!(target.check(&v), Err(AppError::StaleFolder));
    v.memberships[0].state.deletion = DeletionState::Active;
    let credential = AccountDraft::new(
        "Changed at same time",
        None,
        None,
        Secret::from_base32("MY")?,
        OtpSettings::default(),
    )?;
    v.edit_account(id, credential, Timestamp(100))?;
    assert_eq!(target.check(&v), Err(AppError::StaleFolder));
    let target = FolderTarget::capture(&v, Some(f))?;
    assert_eq!(
        save(
            &mut v,
            &target,
            "Public",
            FolderColor::Blue,
            &[AccountId::new()],
            false
        ),
        Err(AppError::StaleFolder)
    );
    target.check(&v)?;
    v.validate()
}
#[test]
fn folder_limits_include_retained_trash_and_reactivation_needs_no_capacity() -> Result<(), AppError>
{
    let mut v = fixture()?;
    let id = v.accounts()[0].id();
    let f = v.add_folder("Public", FolderColor::Blue, Timestamp(100))?;
    v.add_membership(f, id, Timestamp(100))?;
    v.memberships[0].state.deletion = DeletionState::Trashed { at: Timestamp(100) };
    for i in 1..Vault::MAX_RECORDS {
        let folder = v.add_folder(&format!("Public {i}"), FolderColor::Green, Timestamp(100))?;
        v.add_membership(folder, id, Timestamp(100))?;
    }
    let target = FolderTarget::capture(&v, Some(f))?;
    save(&mut v, &target, "Retained", FolderColor::Red, &[id], false)?;
    let target = FolderTarget::capture(&v, Some(f))?;
    let other = v.accounts()[1].id();
    assert_eq!(
        save(
            &mut v,
            &target,
            "Rejected",
            FolderColor::Pink,
            &[id, other],
            false
        ),
        Err(AppError::FolderLimit)
    );
    target.check(&v)?;
    let create = FolderTarget::capture(&v, None)?;
    assert_eq!(
        save(
            &mut v,
            &create,
            "No capacity",
            FolderColor::Blue,
            &[],
            false
        ),
        Err(AppError::FolderLimit)
    );
    assert_eq!(v.folders()[0].name(), "Retained");
    v.validate()
}
