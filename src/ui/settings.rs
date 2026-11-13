//! Settings metadata and local contextual copy. No credential cache or networking.
use super::components as c;
use crate::{
    domain::records::*,
    platform::startup::{self, Startup},
};
use eframe::egui;
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
    time::Instant,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Counts {
    pub accounts: usize,
    pub sets: usize,
    pub available: usize,
    pub used: usize,
    pub trash: usize,
}
impl Counts {
    pub fn new(vault: &Vault) -> Self {
        let active: BTreeSet<_> = vault
            .accounts()
            .iter()
            .filter(|a| a.state().deletion == DeletionState::Active)
            .map(Account::id)
            .collect();
        let mut result = Self {
            accounts: active.len(),
            trash: vault
                .accounts()
                .iter()
                .filter(|a| a.state().deletion != DeletionState::Active)
                .count()
                + vault
                    .folders()
                    .iter()
                    .filter(|f| f.state().deletion != DeletionState::Active)
                    .count()
                + vault
                    .memberships()
                    .iter()
                    .filter(|m| m.state().deletion != DeletionState::Active)
                    .count()
                + vault
                    .backup_sets()
                    .iter()
                    .filter(|s| s.state().deletion != DeletionState::Active)
                    .count()
                + vault
                    .backup_sets()
                    .iter()
                    .flat_map(|s| s.codes())
                    .filter(|c| c.state().deletion != DeletionState::Active)
                    .count(),
            ..Self::default()
        };
        for set in vault.backup_sets().iter().filter(|s| {
            s.state().deletion == DeletionState::Active && active.contains(&s.account_id())
        }) {
            result.sets += 1;
            for code in set
                .codes()
                .iter()
                .filter(|c| c.state().deletion == DeletionState::Active)
            {
                if code.used_at().is_some() {
                    result.used += 1;
                } else {
                    result.available += 1;
                }
            }
        }
        result
    }
}

type ProbeResult = Arc<Mutex<Option<Result<startup::Status, startup::Error>>>>;
struct Probe {
    generation: u64,
    result: ProbeResult,
}
#[derive(Default)]
pub struct Session {
    pub service: Startup,
    probe: Option<Probe>,
    pub actual: Option<Result<startup::Status, startup::Error>>,
    pub feedback: Option<(&'static str, bool)>,
    pub pending_startup: bool,
    pub pending_settings: bool,
    pub submitted_dirty: Option<Option<Instant>>,
    pub focus: bool,
    pub return_focus: Option<egui::Id>,
}
impl Session {
    pub fn reset(&mut self) {
        self.probe = None;
        self.actual = None;
        self.feedback = None;
        self.pending_startup = false;
        self.pending_settings = false;
        self.submitted_dirty = None;
        self.focus = false;
        self.return_focus = None;
    }
    pub fn refresh(&mut self) {
        self.probe = None;
        self.actual = None;
    }
    /// Read-only OS observation may run without moving/cloning the vault. It has
    /// no side effects and carries only enum status. Stale sessions drop it.
    pub fn poll(&mut self, generation: u64, repaint: impl Fn() + Send + 'static) {
        if let Some(probe) = &self.probe {
            let done = probe
                .result
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .take();
            if let Some(done) = done {
                if probe.generation == generation {
                    self.actual = Some(done);
                }
                self.probe = None;
            }
        }
        if self.actual.is_none() && self.probe.is_none() && !self.pending_startup {
            let service = self.service.clone();
            let result: ProbeResult = Arc::new(Mutex::new(None));
            self.probe = Some(Probe {
                generation,
                result: result.clone(),
            });
            std::thread::spawn(move || {
                let state =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| service.inspect()))
                        .unwrap_or(Err(startup::Error::Unavailable));
                *result.lock().unwrap_or_else(|e| e.into_inner()) = Some(state);
                repaint();
            });
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HelpTopic {
    Overview,
    Capture,
    BackupCodes,
    Backups,
    Clock,
    Trash,
    Lock,
}
impl HelpTopic {
    pub fn title(self) -> &'static str {
        match self {
            Self::Overview => "Using Authenticator",
            Self::Capture => "Scan or enter an account",
            Self::BackupCodes => "Service recovery codes",
            Self::Backups => "Encrypted backup and recovery",
            Self::Clock => "Windows time and codes",
            Self::Trash => "Trash and permanent deletion",
            Self::Lock => "Local passphrase and locking",
        }
    }
}
pub fn help(ui: &mut egui::Ui, topic: HelpTopic) {
    c::heading(ui, topic.title(), 2);
    match topic {
        HelpTopic::Overview => {
            c::body(
                ui,
                "Authenticator keeps your vault on this device. Use 2FA Codes, Backup Codes and Settings without an app account or internet connection.",
            );
            c::secondary(
                ui,
                "Keyboard: Tab and Shift+Tab move focus. Enter or Space activates buttons. Ctrl+L locks the vault. Escape closes dialogs or cancels active work and locks. Settings → Export & Import creates or restores a local encrypted backup.",
            );
            c::secondary(
                ui,
                "Your vault passphrase cannot be reset. Keep an independent encrypted backup and its separate passphrase. A compromised unlocked device, Windows clipboard history or other clipboard tools can expose copied data. No complete memory or secure-erasure guarantee is offered.",
            );
        }
        HelpTopic::Capture => {
            c::body(
                ui,
                "Display the service's setup QR in another window. Choose Screenshot to Scan, drag a region around the QR and release. Escape or Cancel returns without importing. Multiple supported codes require an explicit selection. Review the account and choose Save before anything is stored.",
            );
            c::secondary(
                ui,
                "If capture fails or the QR is missing, retry with the whole QR visible, use Import from Photos for a local PNG/JPEG, or choose Manually Input. Images stay local; captures and QR payloads are not saved. Choose only disposable data when sharing screenshots of this app.",
            );
            c::secondary(
                ui,
                "Supported: otpauth TOTP, SHA-1/256/512, 6/8 digits and 15/30/60/120-second periods. Defaults: SHA-1, 6 digits, 30 seconds. HOTP and proprietary migration codes are unsupported. PNG/JPEG input is limited to 8 MiB, 4096 pixels per side and 4,194,304 pixels; at most 7 candidates with 2048-byte payloads. Oversized or ambiguous input is rejected, never silently truncated.",
            );
        }
        HelpTopic::BackupCodes => {
            c::body(
                ui,
                "Backup codes are issued by each service and may work only once. Keep an independent copy somewhere safe in case you lose this vault. Authenticator does not create service recovery codes.",
            );
            c::secondary(
                ui,
                "Add Backup Codes selects an existing 2FA account. Check the service and account identifier before continuing. Existing sets, including sets in Trash, are identified. Enter one code per line or read a local UTF-8 text file (optional BOM, 1 MiB maximum), then Preview changes before Save. Blank lines are ignored; exact duplicates require review.",
            );
            c::secondary(
                ui,
                "Editing keeps exact retained used history; omitted codes move to Trash. Detail has separate Reveal/Conceal, Copy, Mark used/available and one-step Undo actions. Copy never marks a code used. Available and used totals include active codes under active accounts and sets. Restore deleted records from Settings → Trash.",
            );
        }
        HelpTopic::Backups => {
            c::body(
                ui,
                "Settings → Export & Import → Export encrypted backup saves all accounts, OTP parameters, folders, recovery-code used states, Trash, metadata and preferences. Choose a local destination and a separate backup passphrase with confirmation. Encryption is mandatory. Existing files require explicit overwrite approval.",
            );
            c::secondary(
                ui,
                "Import backup accepts native .authpbk backups. Enter the backup passphrase, review counts and conflicts, then choose Merge or explicitly confirm Replace. Default Merge keeps local accounts; separate copies require deliberate choices. Wrong passphrase, invalid data or cancellation before commit cannot partially import. Once an atomic save is admitted, it may finish after cancellation; the app locks and ignores late success.",
            );
            c::secondary(
                ui,
                "Portable backups do not need the original Windows user or device keys. Keep the backup file and passphrase separately and test recovery. Neither passphrase can be reset. Import retains local startup registration and does not silently change Windows settings. Reference-app .vault files are unsupported unless compatibility is independently proven.",
            );
        }
        HelpTopic::Clock => {
            c::body(
                ui,
                "Codes use the current Windows clock and are recomputed at each period boundary and on the next frame after resuming. A detected wall-clock discontinuity shows guidance. This is not an accuracy check against the service.",
            );
            c::secondary(
                ui,
                "If a service rejects a code, check Windows Settings → Time & language → Date & time. Confirm the date, time and time zone, then try a fresh code. Authenticator does not change the system clock, contact a time server or compensate with a hidden offset. A large clock error can make valid credentials produce rejected codes.",
            );
        }
        HelpTopic::Trash => {
            c::body(
                ui,
                "Trash has no automatic expiry. Restore reactivates the selected record and preserves its original seed, OTP settings and used-code history. Records deleted separately remain in Trash. Restore a deleted parent before an individual child. Matching records require explicit separate-record consent and never silently overwrite active data.",
            );
            c::secondary(
                ui,
                "Permanently delete and Empty trash require a scope review and confirmation. Account deletion includes its recovery sets, codes and folder links; folder deletion preserves accounts. Removing the final stored code also removes its empty set, as disclosed. These operations cannot be undone here. Filesystem/SSD/memory and old-backup secure erasure is not guaranteed; existing backups remain unchanged.",
            );
        }
        HelpTopic::Lock => {
            c::body(
                ui,
                "The local vault passphrase creates and unlocks the encrypted vault; it is not an app login. Default idle lock is 5 minutes. Lock vault or Ctrl+L locks immediately; Windows session lock, suspend and relaunch also require unlocking again.",
            );
            c::secondary(
                ui,
                "There is no passphrase reset or backdoor. Recovery requires an independently encrypted backup and its backup passphrase. Lock conceals credential UI and requests app-owned clipboard cleanup. An already admitted encrypted write may finish; active worker/OS memory copies cannot be forcibly erased before work returns. Keep independent service recovery methods.",
            );
        }
    }
}
