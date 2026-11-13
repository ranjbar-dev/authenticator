//! Pure domain layer: no UI, files, clipboard or networking.
pub mod clock;
pub mod folder_edit;
#[cfg(test)]
mod folder_tests;
pub mod otp;
pub mod records;
pub mod recovery_code;
pub mod recovery_edit;
pub mod recovery_input;
pub mod uri;
pub mod validation;
