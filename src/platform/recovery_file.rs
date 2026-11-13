//! Bounded read-only local plaintext import. No raw copy, persistence or logging.
use crate::{domain::recovery_input::MAX_TEXT_BYTES, error::AppError};
use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::{Component, Path},
};
use zeroize::Zeroizing;

pub fn read(path: &Path) -> Result<Zeroizing<String>, AppError> {
    if !path.is_absolute()
        || path.file_name().is_none()
        || path.components().any(|c| matches!(c, Component::ParentDir))
    {
        return Err(AppError::BackupFileUnavailable);
    }
    #[cfg(windows)]
    let (_parents, file) = {
        use std::{
            os::windows::fs::{MetadataExt, OpenOptionsExt},
            path::Prefix,
        };
        use windows_sys::Win32::Storage::FileSystem::*;
        const DRIVE_REMOVABLE: u32 = 2;
        const DRIVE_FIXED: u32 = 3;
        const DRIVE_RAMDISK: u32 = 6;
        let drive = match path.components().next() {
            Some(Component::Prefix(p)) => match p.kind() {
                Prefix::Disk(d) => d,
                _ => return Err(AppError::BackupFileUnavailable),
            },
            _ => return Err(AppError::BackupFileUnavailable),
        };
        let root = [u16::from(drive), u16::from(b':'), u16::from(b'\\'), 0];
        // SAFETY: the root buffer is a live null-terminated drive root.
        if !matches!(
            unsafe { GetDriveTypeW(root.as_ptr()) },
            DRIVE_FIXED | DRIVE_REMOVABLE | DRIVE_RAMDISK
        ) {
            return Err(AppError::BackupFileUnavailable);
        }
        for component in path.components() {
            if let Component::Normal(name) = component {
                let name = name.to_str().ok_or(AppError::BackupFileUnavailable)?;
                let stem = name
                    .split('.')
                    .next()
                    .unwrap_or_default()
                    .to_ascii_uppercase();
                if name.contains(':')
                    || name.ends_with([' ', '.'])
                    || name.contains('\0')
                    || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                    || (stem.len() == 4
                        && (stem.starts_with("COM") || stem.starts_with("LPT"))
                        && stem.as_bytes()[3].is_ascii_digit())
                {
                    return Err(AppError::BackupFileUnavailable);
                }
            }
        }
        // Open root-to-leaf without following reparse points. Denying write and
        // delete sharing pins each checked directory until the bounded read ends,
        // closing the ancestor replacement race of metadata-only preflight.
        let mut parents = Vec::<File>::new();
        for ancestor in path
            .ancestors()
            .skip(1)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            let directory = OpenOptions::new()
                .access_mode(FILE_READ_ATTRIBUTES)
                .share_mode(FILE_SHARE_READ)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
                .open(ancestor)
                .map_err(|_| AppError::BackupFileUnavailable)?;
            let metadata = directory
                .metadata()
                .map_err(|_| AppError::BackupFileUnavailable)?;
            if !metadata.is_dir() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
            {
                return Err(AppError::BackupFileUnavailable);
            }
            parents.push(directory);
        }
        let file = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|_| AppError::BackupFileUnavailable)?;
        (parents, file)
    };
    #[cfg(not(windows))]
    let file = {
        for ancestor in path.ancestors() {
            if std::fs::symlink_metadata(ancestor)
                .map_err(|_| AppError::BackupFileUnavailable)?
                .file_type()
                .is_symlink()
            {
                return Err(AppError::BackupFileUnavailable);
            }
        }
        OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|_| AppError::BackupFileUnavailable)?
    };
    let metadata = file
        .metadata()
        .map_err(|_| AppError::BackupFileUnavailable)?;
    if !metadata.is_file() {
        return Err(AppError::BackupFileUnavailable);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        // The file was opened with OPEN_REPARSE_POINT, so these are handle
        // attributes of the selected object rather than a followed link target.
        if metadata.file_attributes()
            & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
            != 0
        {
            return Err(AppError::BackupFileUnavailable);
        }
    }
    let mut bytes = Zeroizing::new(Vec::new());
    file.take((MAX_TEXT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| AppError::BackupFileUnavailable)?;
    if bytes.len() > MAX_TEXT_BYTES {
        return Err(AppError::BackupFileTooLarge);
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| AppError::InvalidBackupEncoding)?;
    Ok(Zeroizing::new(text.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::recovery_input::RecoveryInput;
    #[test]
    fn generated_utf8_file_matches_text_and_source_is_unchanged()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("public-generated-only.txt");
        let input = "\u{feff} public-alpha \r\n\r\nβeta\npublic-alpha\nPublic-alpha\n";
        std::fs::write(&path, input)?;
        let from_file = RecoveryInput::parse(&read(&path)?)?;
        let from_text = RecoveryInput::parse(input)?;
        assert_eq!((from_file.blanks, from_file.duplicates), (1, 1));
        assert_eq!(from_file.values, from_text.values);
        assert_eq!(std::fs::read(&path)?, input.as_bytes());
        assert_eq!(std::fs::read_dir(dir.path())?.count(), 1);
        Ok(())
    }
    #[test]
    fn invalid_encoding_size_and_nonfiles_fail_with_fixed_messages()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("public-bounded-only.txt");
        std::fs::write(&path, [0xff, 0xfe, b'x', 0])?;
        assert_eq!(read(&path).err(), Some(AppError::InvalidBackupEncoding));
        std::fs::write(&path, vec![b' '; MAX_TEXT_BYTES])?;
        assert_eq!(read(&path)?.len(), MAX_TEXT_BYTES);
        std::fs::write(&path, vec![b' '; MAX_TEXT_BYTES + 1])?;
        assert_eq!(read(&path).err(), Some(AppError::BackupFileTooLarge));
        assert_eq!(
            read(dir.path()).err(),
            Some(AppError::BackupFileUnavailable)
        );
        assert_eq!(
            read(Path::new("relative-plaintext-sentinel")).err(),
            Some(AppError::BackupFileUnavailable)
        );
        assert!(
            !AppError::BackupFileUnavailable
                .to_string()
                .contains("public-bounded")
        );
        Ok(())
    }
    #[cfg(windows)]
    #[test]
    fn rejects_links_ancestors_ads_devices_and_traversal_before_reading()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let target = dir.path().join("public-source.txt");
        std::fs::write(&target, b"public-generated")?;
        let link = dir.path().join("public-link.txt");
        std::os::windows::fs::symlink_file(&target, &link)?;
        assert_eq!(read(&link).err(), Some(AppError::BackupFileUnavailable));
        let nested = dir.path().join("nested");
        std::fs::create_dir(&nested)?;
        std::fs::write(nested.join("codes.txt"), b"public-generated")?;
        let linked_directory = dir.path().join("linked-directory");
        std::os::windows::fs::symlink_dir(&nested, &linked_directory)?;
        assert_eq!(
            read(&linked_directory.join("codes.txt")).err(),
            Some(AppError::BackupFileUnavailable)
        );
        for path in [
            dir.path().join("public-source.txt:stream"),
            dir.path().join("NUL.txt"),
            dir.path().join("nested/../public-source.txt"),
            std::path::PathBuf::from(r"\\public.invalid\share\codes.txt"),
        ] {
            assert_eq!(read(&path).err(), Some(AppError::BackupFileUnavailable));
        }
        Ok(())
    }
}
