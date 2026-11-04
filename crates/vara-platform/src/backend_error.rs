//! Classified system diagnostics; never serialize device names or credential data.
use vara_core::UserMessage as M;
pub(crate) fn hound(error: &hound::Error) -> M {
    match error {
        hound::Error::IoError(e) => M::io_cause(e),
        hound::Error::Unsupported => M::new("media.cause_unsupported"),
        _ => M::new("media.cause_format"),
    }
}
pub(crate) fn json(error: &serde_json::Error) -> M {
    M::new("json.parse")
        .number("line", error.line() as i64)
        .number("column", error.column() as i64)
}
pub(crate) fn config(error: &cpal::DefaultStreamConfigError) -> M {
    match error {
        cpal::DefaultStreamConfigError::DeviceNotAvailable => M::new("audio.device_unavailable"),
        cpal::DefaultStreamConfigError::StreamTypeNotSupported => {
            M::new("audio.config_unsupported")
        }
        cpal::DefaultStreamConfigError::BackendSpecific { .. } => M::new("audio.backend"),
    }
}
pub(crate) fn build(error: &cpal::BuildStreamError) -> M {
    match error {
        cpal::BuildStreamError::DeviceNotAvailable => M::new("audio.device_unavailable"),
        cpal::BuildStreamError::StreamConfigNotSupported => M::new("audio.config_unsupported"),
        cpal::BuildStreamError::InvalidArgument => M::new("audio.invalid_argument"),
        cpal::BuildStreamError::StreamIdOverflow => M::new("audio.stream_limit"),
        cpal::BuildStreamError::BackendSpecific { .. } => M::new("audio.backend"),
    }
}
pub(crate) fn play(error: &cpal::PlayStreamError) -> M {
    match error {
        cpal::PlayStreamError::DeviceNotAvailable => M::new("audio.device_unavailable"),
        cpal::PlayStreamError::BackendSpecific { .. } => M::new("audio.backend"),
    }
}
pub(crate) fn stream(error: &cpal::StreamError) -> M {
    match error {
        cpal::StreamError::DeviceNotAvailable => M::new("audio.device_unavailable"),
        cpal::StreamError::BackendSpecific { .. } => M::new("audio.backend"),
    }
}
#[cfg(any(test, not(target_os = "macos")))]
pub(crate) fn keyring(error: &keyring::Error) -> M {
    use keyring::Error as E;
    let mut message = match error {
        E::PlatformFailure(_) => M::new("keyring.platform"),
        E::NoStorageAccess(_) => M::new("keyring.access"),
        E::NoEntry => M::new("keyring.missing"),
        E::BadEncoding(_) => M::new("keyring.encoding"),
        E::TooLong(_, limit) => M::new("keyring.too_long").number("limit", (*limit).into()),
        E::Invalid(..) => M::new("keyring.invalid"),
        E::Ambiguous(_) => M::new("keyring.ambiguous"),
        _ => M::new("keyring.platform"),
    };
    let mut source = std::error::Error::source(error);
    while let Some(e) = source {
        if let Some(io) = e.downcast_ref::<std::io::Error>() {
            message = message.caused_by(M::io_cause(io));
            break;
        }
        source = e.source();
    }
    message
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_platform_error_variants_remain_distinct_without_payload() {
        assert_ne!(
            build(&cpal::BuildStreamError::DeviceNotAvailable).code,
            build(&cpal::BuildStreamError::InvalidArgument).code
        );
        let backend = cpal::BuildStreamError::BackendSpecific {
            err: cpal::BackendSpecificError {
                description: "secret microphone /private/path".into(),
            },
        };
        assert_eq!(build(&backend).code, "audio.backend");
        assert!(!build(&backend).wire().contains("secret"));
        let err = keyring::Error::BadEncoding(b"private-key".to_vec());
        assert_eq!(keyring(&err).code, "keyring.encoding");
        assert!(!keyring(&err).wire().contains("private-key"));
        let err = keyring::Error::NoStorageAccess(Box::new(std::io::Error::from_raw_os_error(13)));
        assert_eq!(keyring(&err).cause.unwrap().code, "io.permission_denied");
        assert_eq!(
            hound(&hound::Error::IoError(std::io::Error::from_raw_os_error(2))).code,
            "io.not_found"
        );
    }
}
