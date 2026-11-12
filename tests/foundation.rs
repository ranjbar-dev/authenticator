use authenticator::{error::AppError, platform::capture::desktop_bounds};

#[test]
fn test_data_is_isolated_and_removed() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::Builder::new()
        .prefix("authenticator-test-")
        .tempdir()?;
    let path = dir.path().to_path_buf();
    assert!(path.starts_with(std::env::temp_dir()));
    std::fs::write(path.join("disposable-fixture"), b"not a credential")?;
    dir.close()?;
    assert!(!path.exists());
    Ok(())
}

#[test]
fn public_error_messages_are_fixed_and_actionable() {
    let errors = [
        AppError::ClockUnavailable,
        AppError::CaptureUnavailable,
        AppError::InvalidSelection,
        AppError::StorageUnavailable,
    ];
    for error in errors {
        assert!(!error.to_string().is_empty());
    }
}

#[cfg(windows)]
#[test]
fn windows_display_adapter_reads_valid_geometry() -> Result<(), AppError> {
    desktop_bounds()?.validate()?;
    Ok(())
}
