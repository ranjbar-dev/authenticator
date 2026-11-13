use super::{
    otp::{OtpSettings, Secret},
    records::*,
    trash::*,
};
use crate::error::AppError;

#[test]
fn trash_maximum_code_graph_review_guard_and_leaf_scope_are_bounded() -> Result<(), AppError> {
    let mut v = Vault::new(Timestamp(100));
    let a = v.add_account(
        AccountDraft::new(
            "Public maximum graph",
            None,
            None,
            Secret::from_base32("MY")?,
            OtpSettings::default(),
        )?,
        Timestamp(100),
    )?;
    let mut codes = Vec::with_capacity(Vault::MAX_RECORDS);
    for i in 0..Vault::MAX_RECORDS {
        let mut value = format!("public-{i:05}-");
        value.extend(std::iter::repeat_n('x', 4096 - value.len()));
        codes.push(BackupCode::new(&value, Timestamp(100))?);
    }
    let code = codes[0].id();
    trash(&mut codes[0].state);
    let set = v.add_backup_set(a, codes, Timestamp(100))?;
    let started = std::time::Instant::now();
    let review = Review::new(&v, Operation::Delete(ItemId::Code(set, code)))?;
    let captured = started.elapsed();
    assert_eq!(
        review.summary().affected,
        Counts {
            codes: 1,
            ..Default::default()
        }
    );
    let started = std::time::Instant::now();
    review.apply(&mut v, false, Timestamp(100))?;
    eprintln!(
        "maximum Trash guard: 10000 codes x4096bytes; capture={}ms checked-apply={}ms",
        captured.as_millis(),
        started.elapsed().as_millis()
    );
    assert_eq!(v.backup_sets[0].codes.len(), Vault::MAX_RECORDS - 1);
    assert_eq!(v.accounts[0].id(), a);
    v.validate()
}

fn fixture() -> Result<Vault, AppError> {
    let mut v = Vault::new(Timestamp(100));
    let f = v.add_folder("Public folder", FolderColor::Blue, Timestamp(100))?;
    for (name, seed) in [("Public A", "MY"), ("Public B", "M4")] {
        let a = v.add_account(
            AccountDraft::new(
                name,
                None,
                Some("fixture.invalid"),
                Secret::from_base32(seed)?,
                OtpSettings::default(),
            )?,
            Timestamp(100),
        )?;
        v.add_membership(f, a, Timestamp(100))?;
        let mut code = BackupCode::new(&format!("public-{name}-used"), Timestamp(100))?;
        code.set_used(true, Timestamp(100))?;
        v.add_backup_set(
            a,
            vec![
                code,
                BackupCode::new(&format!("public-{name}-available"), Timestamp(100))?,
            ],
            Timestamp(100),
        )?;
    }
    Ok(v)
}
fn trash(state: &mut RecordState) {
    state.deletion = DeletionState::Trashed { at: Timestamp(100) };
}
fn apply(v: &mut Vault, operation: Operation) -> Result<(), AppError> {
    Review::new(v, operation)?.apply(v, true, Timestamp(50))?;
    v.validate()
}

#[test]
fn trash_parent_restore_preserves_seed_settings_used_history_and_independent_child_states()
-> Result<(), AppError> {
    let mut v = fixture()?;
    let a = v.accounts[0].id();
    let f = v.folders[0].id();
    let s = v.backup_sets[0].id();
    let c = v.backup_sets[0].codes[0].id();
    let m = v.memberships[0].id();
    let original = v.accounts[0].credential().clone();
    trash(&mut v.accounts[0].state);
    assert_eq!(
        items(&v).len(),
        1,
        "inherited hidden children are not separate deletions"
    );
    trash(&mut v.backup_sets[0].state);
    trash(&mut v.backup_sets[0].codes[0].state);
    trash(&mut v.memberships[0].state);
    trash(&mut v.folders[0].state);
    assert_eq!(items(&v).len(), 5);
    apply(&mut v, Operation::Restore(ItemId::Account(a)))?;
    assert!(v.accounts[0].credential().duplicate_criteria(&original).0);
    assert_eq!(v.accounts[0].credential().service(), original.service());
    assert_eq!(
        v.accounts[0].credential().account_identifier(),
        original.account_identifier()
    );
    assert_eq!(v.backup_sets[0].codes[0].used_at(), Some(Timestamp(100)));
    for id in [
        ItemId::Folder(f),
        ItemId::Set(s),
        ItemId::Code(s, c),
        ItemId::Membership(m),
    ] {
        assert!(matches!(
            id.state(&v).ok_or(AppError::InvalidRecord)?.deletion,
            DeletionState::Trashed { .. }
        ));
    }
    assert_eq!(items(&v).len(), 4);
    Ok(())
}

#[test]
fn trash_restore_requires_visible_parents_and_never_revives_other_own_trash() -> Result<(), AppError>
{
    let mut v = fixture()?;
    let a = v.accounts[0].id();
    let f = v.folders[0].id();
    let s = v.backup_sets[0].id();
    let c = v.backup_sets[0].codes[0].id();
    let m = v.memberships[0].id();
    trash(&mut v.accounts[0].state);
    trash(&mut v.folders[0].state);
    trash(&mut v.backup_sets[0].state);
    trash(&mut v.backup_sets[0].codes[0].state);
    trash(&mut v.memberships[0].state);
    for id in [ItemId::Membership(m), ItemId::Set(s), ItemId::Code(s, c)] {
        let r = Review::new(&v, Operation::Restore(id))?;
        assert!(!r.summary().parents.is_empty());
        assert_eq!(
            r.apply(&mut v, true, Timestamp(100)),
            Err(AppError::TrashParentHidden)
        );
        r.check(&v)?;
    }
    apply(&mut v, Operation::Restore(ItemId::Account(a)))?;
    apply(&mut v, Operation::Restore(ItemId::Folder(f)))?;
    apply(&mut v, Operation::Restore(ItemId::Membership(m)))?;
    apply(&mut v, Operation::Restore(ItemId::Set(s)))?;
    assert_ne!(
        v.backup_sets[0].codes[0].state().deletion,
        DeletionState::Active
    );
    apply(&mut v, Operation::Restore(ItemId::Code(s, c)))?;
    assert!(items(&v).is_empty());
    assert_eq!(v.backup_sets[0].codes[0].used_at(), Some(Timestamp(100)));
    Ok(())
}

#[test]
fn trash_restore_duplicates_require_separate_record_consent_without_overwrite()
-> Result<(), AppError> {
    let mut v = fixture()?;
    let a = v.accounts[0].id();
    let duplicate = v.accounts[0].credential().clone();
    let other = v.add_account(duplicate, Timestamp(100))?;
    trash(&mut v.accounts[0].state);
    let r = Review::new(&v, Operation::Restore(ItemId::Account(a)))?;
    assert_eq!(r.summary().conflicts, vec![ItemId::Account(other)]);
    assert_eq!(
        r.apply(&mut v, false, Timestamp(100)),
        Err(AppError::ReviewTrashConflicts)
    );
    r.check(&v)?;
    r.apply(&mut v, true, Timestamp(100))?;
    assert_eq!(v.accounts().len(), 3);
    let f = v.folders[0].id();
    let other_f = v.add_folder("Public folder", FolderColor::Green, Timestamp(100))?;
    trash(&mut v.folders[0].state);
    let r = Review::new(&v, Operation::Restore(ItemId::Folder(f)))?;
    assert_eq!(r.summary().conflicts, vec![ItemId::Folder(other_f)]);
    assert_eq!(
        r.apply(&mut v, false, Timestamp(100)),
        Err(AppError::ReviewTrashConflicts)
    );
    r.check(&v)?;
    r.apply(&mut v, true, Timestamp(100))?;
    assert_eq!(v.folders().len(), 2);
    assert_eq!(v.folders[1].color(), FolderColor::Green);
    v.validate()
}

#[test]
fn trash_permanent_account_and_folder_cascades_preserve_unrelated_records() -> Result<(), AppError>
{
    let mut v = fixture()?;
    let a = v.accounts[0].id();
    let b = v.accounts[1].id();
    let f = v.folders[0].id();
    let original_b = v.accounts[1].credential().clone();
    let original_b_set = v.backup_sets[1].id();
    let original_b_code = v.backup_sets[1].codes[0].id();
    trash(&mut v.accounts[0].state);
    let r = Review::new(&v, Operation::Delete(ItemId::Account(a)))?;
    assert_eq!(
        r.summary().affected,
        Counts {
            accounts: 1,
            memberships: 1,
            sets: 1,
            codes: 2,
            ..Default::default()
        }
    );
    assert_eq!(r.summary().active_associations, 4);
    r.apply(&mut v, false, Timestamp(100))?;
    assert_eq!(v.accounts.len(), 1);
    assert_eq!(v.accounts[0].id(), b);
    assert!(v.accounts[0].credential().duplicate_criteria(&original_b).0);
    assert_eq!(v.backup_sets[0].id(), original_b_set);
    assert_eq!(v.backup_sets[0].codes[0].id(), original_b_code);
    assert_eq!(v.backup_sets[0].codes[0].used_at(), Some(Timestamp(100)));
    assert_eq!(v.memberships.len(), 1);
    assert_eq!(v.memberships[0].account_id(), b);
    v.validate()?;
    trash(&mut v.folders[0].state);
    let r = Review::new(&v, Operation::Delete(ItemId::Folder(f)))?;
    assert_eq!(
        r.summary().affected,
        Counts {
            folders: 1,
            memberships: 1,
            ..Default::default()
        }
    );
    r.apply(&mut v, false, Timestamp(100))?;
    assert!(v.folders.is_empty() && v.memberships.is_empty());
    assert_eq!(v.accounts[0].id(), b);
    assert_eq!(v.backup_sets[0].codes[0].used_at(), Some(Timestamp(100)));
    v.validate()
}

#[test]
fn trash_empty_deduplicates_overlapping_cascades_and_last_code_removes_only_its_empty_set()
-> Result<(), AppError> {
    let mut v = fixture()?;
    let a = v.accounts[0].id();
    trash(&mut v.accounts[0].state);
    trash(&mut v.backup_sets[0].state);
    trash(&mut v.backup_sets[0].codes[0].state);
    trash(&mut v.memberships[0].state);
    let r = Review::new(&v, Operation::Empty)?;
    assert_eq!(
        r.summary().affected,
        Counts {
            accounts: 1,
            memberships: 1,
            sets: 1,
            codes: 2,
            ..Default::default()
        }
    );
    r.apply(&mut v, false, Timestamp(100))?;
    assert!(!v.accounts.iter().any(|x| x.id() == a));
    assert!(items(&v).is_empty());
    v.validate()?;
    let set = v.backup_sets[0].id();
    let account = v.accounts[0].id();
    v.backup_sets[0].codes.remove(1);
    let code = v.backup_sets[0].codes[0].id();
    trash(&mut v.backup_sets[0].codes[0].state);
    let r = Review::new(&v, Operation::Delete(ItemId::Code(set, code)))?;
    assert_eq!(r.summary().empty_sets, 1);
    assert_eq!(
        r.summary().affected,
        Counts {
            sets: 1,
            codes: 1,
            ..Default::default()
        }
    );
    assert_eq!(r.summary().active_associations, 1);
    r.apply(&mut v, false, Timestamp(100))?;
    assert!(v.backup_sets.is_empty());
    assert_eq!(v.accounts[0].id(), account);
    v.validate()
}

#[test]
fn trash_stale_binding_covers_equal_time_secret_metadata_relationship_and_preference_changes()
-> Result<(), AppError> {
    let mut base = fixture()?;
    trash(&mut base.accounts[0].state);
    let id = base.accounts[0].id();
    let r = Review::new(&base, Operation::Delete(ItemId::Account(id)))?;
    for change in 0..13 {
        let mut v = base.clone();
        match change {
            0 => {
                v.accounts[0].credential = AccountDraft::new(
                    "Public changed",
                    None,
                    None,
                    Secret::from_base32("ME")?,
                    OtpSettings::default(),
                )?
            }
            1 => v.folders[0].name = "Public changed".into(),
            2 => v.folders[0].color = FolderColor::Pink,
            3 => v.memberships[0].state.deletion = DeletionState::Trashed { at: Timestamp(100) },
            4 => v.memberships[0].account_id = v.accounts[1].id(),
            5 => {
                v.backup_sets[0].codes[0].value =
                    zeroize::Zeroizing::new("public-changed-code".into())
            }
            6 => v.backup_sets[0].codes[0].used_at = None,
            7 => {
                v.backup_sets[0].codes[0].state.deletion =
                    DeletionState::Trashed { at: Timestamp(100) }
            }
            8 => v.preferences.query = zeroize::Zeroizing::new("changed pending preference".into()),
            9 => v.preferences.folder = FolderFilter::Unfiled,
            10 => v.preferences.auto_start = true,
            11 => v.accounts[0].state.deletion = DeletionState::Active,
            _ => v.state.updated_at = Timestamp(101),
        }
        assert_eq!(
            r.check(&v),
            Err(AppError::StaleTrash),
            "field change {change}"
        );
        assert_eq!(
            r.apply(&mut v, true, Timestamp(100)),
            Err(AppError::StaleTrash)
        );
    }
    assert_eq!(format!("{r:?}"), "TrashReview([REDACTED])");
    base.state.updated_at = Timestamp(u64::MAX);
    apply(&mut base, Operation::Restore(ItemId::Account(id)))?;
    assert_eq!(base.accounts[0].state().updated_at, Timestamp(u64::MAX));
    Ok(())
}

#[test]
fn trash_dense_memberships_and_empty_code_set_scope_are_bounded_and_valid() -> Result<(), AppError>
{
    let mut v = fixture()?;
    let a = v.accounts[0].id();
    for i in 1..Vault::MAX_RECORDS - 1 {
        let f = v.add_folder(
            &format!("Public dense {i}"),
            FolderColor::Blue,
            Timestamp(100),
        )?;
        v.add_membership(f, a, Timestamp(100))?;
    }
    assert_eq!(v.memberships.len(), Vault::MAX_RECORDS);
    trash(&mut v.accounts[0].state);
    let r = Review::new(&v, Operation::Delete(ItemId::Account(a)))?;
    assert_eq!(r.summary().affected.memberships, Vault::MAX_RECORDS - 1);
    r.apply(&mut v, false, Timestamp(100))?;
    assert_eq!(v.memberships.len(), 1);
    v.validate()?;
    for c in &mut v.backup_sets[0].codes {
        trash(&mut c.state);
    }
    let r = Review::new(&v, Operation::Empty)?;
    assert_eq!(r.summary().empty_sets, 1);
    assert_eq!(r.summary().affected.codes, 2);
    r.apply(&mut v, false, Timestamp(100))?;
    assert!(v.backup_sets.is_empty());
    v.validate()
}
