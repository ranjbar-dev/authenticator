//! Explicit per-user Windows startup registration. No shell, arguments or network.
use std::{fmt, sync::Arc};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Missing,
    Registered,
    Conflict,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Unavailable,
    Location,
    Conflict,
    Changed,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unavailable => "Windows startup registration could not be accessed. Retry the check or change after Windows permits access.",
            Self::Location => "Run the packaged Authenticator.exe from a local folder with a shorter path to use Auto Start.",
            Self::Conflict => "The startup name belongs to a different command or copy. It was kept unchanged. Manage the older copy in Windows before retrying.",
            Self::Changed => "Startup registration changed during this action. Check it again before retrying.",
        })
    }
}
impl std::error::Error for Error {}
pub(crate) trait Backend: Send + Sync {
    fn inspect(&self) -> Result<Status, Error>;
    fn change(&self, enabled: bool) -> Result<Status, Error>;
}
#[derive(Clone)]
pub struct Startup(Arc<dyn Backend>);
impl fmt::Debug for Startup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Startup([REDACTED])")
    }
}
impl Default for Startup {
    fn default() -> Self {
        #[cfg(all(windows, not(test)))]
        return Self(Arc::new(native::Registration::system()));
        #[cfg(test)]
        return testing::fake().0;
        #[cfg(not(any(windows, test)))]
        return Self(Arc::new(Unsupported));
    }
}
impl Startup {
    pub fn inspect(&self) -> Result<Status, Error> {
        self.0.inspect()
    }
    /// Caller must hold an explicit current action and cancellation admission.
    pub(crate) fn change(&self, enabled: bool) -> Result<Status, Error> {
        self.0.change(enabled)
    }
}
#[cfg(not(any(windows, test)))]
struct Unsupported;
#[cfg(not(any(windows, test)))]
impl Backend for Unsupported {
    fn inspect(&self) -> Result<Status, Error> {
        Err(Error::Unavailable)
    }
    fn change(&self, _: bool) -> Result<Status, Error> {
        Err(Error::Unavailable)
    }
}

#[cfg(windows)]
mod native {
    use super::*;
    use std::{
        os::windows::ffi::OsStrExt,
        path::{Component, Path, Prefix},
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS},
        System::Registry::*,
    };

    const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE: &str = "Authenticator.ranjbar-dev";
    const MAX_COMMAND: usize = 260;
    // Fixed bounded buffer: foreign values are classified, never printed/retained.
    const MAX_VALUE_UNITS: usize = MAX_COMMAND + 2;

    pub(super) struct Registration {
        key: Vec<u16>,
        value: Vec<u16>,
        command: Result<Vec<u16>, Error>,
    }
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }
    fn command(path: &Path) -> Result<Vec<u16>, Error> {
        let units: Vec<_> = path.as_os_str().encode_wide().collect();
        let mut components = path.components();
        if !matches!(components.next(), Some(Component::Prefix(p)) if matches!(p.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
            || !matches!(components.next(), Some(Component::RootDir))
            || components.any(|c| !matches!(c, Component::Normal(_)))
            || !path
                .file_name()
                .is_some_and(|n| n.eq_ignore_ascii_case("Authenticator.exe"))
            || units
                .iter()
                .any(|c| *c < 32 || *c == 127 || *c == u16::from(b'"'))
            || units.len() + 2 > MAX_COMMAND
        {
            return Err(Error::Location);
        }
        Ok(std::iter::once(u16::from(b'"'))
            .chain(units)
            .chain([u16::from(b'"'), 0])
            .collect())
    }
    impl Registration {
        pub(super) fn system() -> Self {
            Self {
                key: wide(RUN),
                value: wide(VALUE),
                command: std::env::current_exe()
                    .map_err(|_| Error::Location)
                    .and_then(|p| command(&p)),
            }
        }
        fn open(&self, write: bool, create: bool) -> Result<Option<Key>, Error> {
            let mut key = ptr::null_mut();
            let access = KEY_QUERY_VALUE | if write { KEY_SET_VALUE } else { 0 };
            // SAFETY: terminated strings, valid out pointer, HKCU-only fixed scope.
            let code = unsafe {
                if create {
                    RegCreateKeyExW(
                        HKEY_CURRENT_USER,
                        self.key.as_ptr(),
                        0,
                        ptr::null(),
                        REG_OPTION_NON_VOLATILE,
                        access,
                        ptr::null(),
                        &mut key,
                        ptr::null_mut(),
                    )
                } else {
                    RegOpenKeyExW(HKEY_CURRENT_USER, self.key.as_ptr(), 0, access, &mut key)
                }
            };
            if code == ERROR_FILE_NOT_FOUND {
                return Ok(None);
            }
            if code != ERROR_SUCCESS {
                return Err(Error::Unavailable);
            }
            Ok(Some(Key(key)))
        }
        fn status(&self, key: &Key) -> Result<Status, Error> {
            let expected = self.command.as_ref().map_err(|e| *e)?;
            let mut buffer = [0u16; MAX_VALUE_UNITS];
            let mut kind = 0;
            let mut bytes = std::mem::size_of_val(&buffer) as u32;
            // SAFETY: buffer byte count matches allocation; result always bounded.
            let code = unsafe {
                RegQueryValueExW(
                    key.0,
                    self.value.as_ptr(),
                    ptr::null(),
                    &mut kind,
                    buffer.as_mut_ptr().cast(),
                    &mut bytes,
                )
            };
            if code == ERROR_FILE_NOT_FOUND {
                return Ok(Status::Missing);
            }
            if code == windows_sys::Win32::Foundation::ERROR_MORE_DATA {
                return Ok(Status::Conflict);
            }
            if code != ERROR_SUCCESS {
                return Err(Error::Unavailable);
            }
            let count = bytes as usize / 2;
            if kind != REG_SZ
                || !bytes.is_multiple_of(2)
                || count > buffer.len()
                || count != expected.len()
                || buffer[..count] != expected[..]
            {
                return Ok(Status::Conflict);
            }
            Ok(Status::Registered)
        }
    }
    struct Key(HKEY);
    impl Drop for Key {
        fn drop(&mut self) {
            unsafe {
                RegCloseKey(self.0);
            }
        }
    }
    impl Backend for Registration {
        fn inspect(&self) -> Result<Status, Error> {
            self.command.as_ref().map_err(|e| *e)?;
            self.open(false, false)?
                .map_or(Ok(Status::Missing), |k| self.status(&k))
        }
        fn change(&self, enabled: bool) -> Result<Status, Error> {
            let expected = self.command.as_ref().map_err(|e| *e)?;
            let Some(key) = self.open(true, enabled)? else {
                return Ok(Status::Missing);
            };
            let before = self.status(&key)?;
            if before == Status::Conflict {
                return Err(Error::Conflict);
            }
            // Never delete the key or any other value. Existing exact state is idempotent.
            let code = if enabled && before == Status::Missing {
                unsafe {
                    RegSetValueExW(
                        key.0,
                        self.value.as_ptr(),
                        0,
                        REG_SZ,
                        expected.as_ptr().cast(),
                        (expected.len() * 2) as u32,
                    )
                }
            } else if !enabled && before == Status::Registered {
                unsafe { RegDeleteValueW(key.0, self.value.as_ptr()) }
            } else {
                ERROR_SUCCESS
            };
            if code != ERROR_SUCCESS {
                return Err(Error::Unavailable);
            }
            let after = self.status(&key)?;
            if after
                != if enabled {
                    Status::Registered
                } else {
                    Status::Missing
                }
            {
                return Err(Error::Changed);
            }
            Ok(after)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn startup_command_quotes_disk_path_and_rejects_unc_arguments_and_long_paths() {
            // Test harness is not the packaged executable; construction/inspection
            // refuses it before reading the user's actual Run value.
            assert_eq!(Registration::system().inspect(), Err(Error::Location));
            assert_eq!(
                command(Path::new(r"C:\Public Folder\Authenticator.exe")).ok(),
                Some(wide("\"C:\\Public Folder\\Authenticator.exe\""))
            );
            for path in [
                r"\\server\share\Authenticator.exe",
                r"\\?\UNC\server\share\Authenticator.exe",
                r"C:\Public\other.exe",
                r"C:\Public\Authenticator.exe\..\Authenticator.exe",
                "C:\\Public\"bad\\Authenticator.exe",
                "C:\\bad\n\\Authenticator.exe",
            ] {
                assert_eq!(command(Path::new(path)), Err(Error::Location));
            }
            assert_eq!(
                command(Path::new(&format!(
                    "C:\\{}\\Authenticator.exe",
                    "x".repeat(260)
                ))),
                Err(Error::Location)
            );
        }
        #[test]
        fn startup_native_isolated_registry_changes_only_owned_value_and_preserves_conflicts()
        -> Result<(), Box<dyn std::error::Error>> {
            let name = format!(
                r"Software\Authenticator-Task19-Test-{}",
                uuid::Uuid::new_v4()
            );
            let registration = Registration {
                key: wide(&name),
                value: wide("Owned"),
                command: Ok(wide("\"C:\\Public Folder\\Authenticator.exe\"")),
            };
            struct Cleanup(Vec<u16>);
            impl Drop for Cleanup {
                fn drop(&mut self) {
                    unsafe {
                        RegDeleteKeyW(HKEY_CURRENT_USER, self.0.as_ptr());
                    }
                }
            }
            let cleanup = Cleanup(registration.key.clone());
            assert_eq!(registration.inspect(), Ok(Status::Missing));
            assert_eq!(registration.change(false), Ok(Status::Missing));
            assert_eq!(registration.change(true), Ok(Status::Registered));
            assert_eq!(registration.change(true), Ok(Status::Registered));
            let key = registration.open(true, false)?.ok_or("missing test key")?;
            let neighbor = wide("Neighbor");
            let foreign = wide("public-foreign-command");
            unsafe {
                assert_eq!(
                    RegSetValueExW(
                        key.0,
                        neighbor.as_ptr(),
                        0,
                        REG_SZ,
                        foreign.as_ptr().cast(),
                        (foreign.len() * 2) as u32
                    ),
                    ERROR_SUCCESS
                );
            }
            assert_eq!(registration.change(false), Ok(Status::Missing));
            unsafe {
                assert_eq!(
                    RegSetValueExW(
                        key.0,
                        registration.value.as_ptr(),
                        0,
                        REG_SZ,
                        foreign.as_ptr().cast(),
                        (foreign.len() * 2) as u32
                    ),
                    ERROR_SUCCESS
                );
            }
            assert_eq!(registration.inspect(), Ok(Status::Conflict));
            assert_eq!(registration.change(true), Err(Error::Conflict));
            assert_eq!(registration.change(false), Err(Error::Conflict));
            let no_terminator: Vec<u8> = expected_without_nul(&registration);
            for (kind, data) in [
                (REG_BINARY, vec![1u8; 17]),
                (REG_SZ, no_terminator),
                (REG_SZ, vec![65u8; 4096]),
            ] {
                unsafe {
                    assert_eq!(
                        RegSetValueExW(
                            key.0,
                            registration.value.as_ptr(),
                            0,
                            kind,
                            data.as_ptr(),
                            data.len() as u32
                        ),
                        ERROR_SUCCESS
                    );
                }
                assert_eq!(registration.inspect(), Ok(Status::Conflict));
                assert_eq!(registration.change(true), Err(Error::Conflict));
                assert_eq!(registration.change(false), Err(Error::Conflict));
            }
            let mut data = [0u16; 128];
            let mut bytes = std::mem::size_of_val(&data) as u32;
            unsafe {
                assert_eq!(
                    RegQueryValueExW(
                        key.0,
                        neighbor.as_ptr(),
                        ptr::null(),
                        ptr::null_mut(),
                        data.as_mut_ptr().cast(),
                        &mut bytes
                    ),
                    ERROR_SUCCESS
                );
            }
            assert_eq!(&data[..foreign.len()], &foreign);
            unsafe {
                RegDeleteValueW(key.0, neighbor.as_ptr());
                RegDeleteValueW(key.0, registration.value.as_ptr());
            }
            drop(key);
            drop(cleanup);
            assert!(registration.open(false, false)?.is_none());
            Ok(())
        }
        fn expected_without_nul(registration: &Registration) -> Vec<u8> {
            registration
                .command
                .as_ref()
                .map(|s| {
                    s[..s.len() - 1]
                        .iter()
                        .flat_map(|n| n.to_le_bytes())
                        .collect()
                })
                .unwrap_or_default()
        }
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use super::*;
    use std::sync::Mutex;
    pub(crate) struct Fake {
        pub status: Mutex<Result<Status, Error>>,
        pub changes: Mutex<Vec<bool>>,
        pub fail_change: Mutex<bool>,
    }
    impl Backend for Fake {
        fn inspect(&self) -> Result<Status, Error> {
            *self.status.lock().unwrap_or_else(|e| e.into_inner())
        }
        fn change(&self, enabled: bool) -> Result<Status, Error> {
            self.changes
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(enabled);
            if *self.fail_change.lock().unwrap_or_else(|e| e.into_inner()) {
                return Err(Error::Unavailable);
            }
            let mut state = self.status.lock().unwrap_or_else(|e| e.into_inner());
            if *state == Ok(Status::Conflict) {
                return Err(Error::Conflict);
            }
            *state = Ok(if enabled {
                Status::Registered
            } else {
                Status::Missing
            });
            *state
        }
    }
    pub(crate) fn fake() -> (Startup, Arc<Fake>) {
        let fake = Arc::new(Fake {
            status: Mutex::new(Ok(Status::Missing)),
            changes: Mutex::new(Vec::new()),
            fail_change: Mutex::new(false),
        });
        (Startup(fake.clone()), fake)
    }
}
