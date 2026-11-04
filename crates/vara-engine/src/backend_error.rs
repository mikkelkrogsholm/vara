//! Safe typed diagnostics: never serialize backend Display strings, paths or model data.
use vara_core::UserMessage as M;

pub(crate) fn json(error: &serde_json::Error) -> M {
    M::new("json.parse")
        .number("line", error.line() as i64)
        .number("column", error.column() as i64)
}
pub(crate) fn whisper(error: &whisper_rs::WhisperError) -> M {
    use whisper_rs::WhisperError::*;
    let code = match error {
        InitError => "whisper.init",
        FailedToCreateState => "whisper.state",
        SpectrogramNotInitialized => "whisper.spectrogram_missing",
        UnableToCalculateSpectrogram => "whisper.spectrogram_failed",
        EncodeNotComplete => "whisper.encode_incomplete",
        FailedToEncode => "whisper.encode",
        DecodeNotComplete => "whisper.decode_incomplete",
        FailedToDecode => "whisper.decode",
        UnableToCalculateEvaluation => "whisper.evaluation",
        InvalidMelBands => "whisper.mel_bands",
        InvalidThreadCount => "whisper.thread_count",
        InvalidText => "whisper.invalid_text",
        InvalidUtf8 { .. } => "whisper.utf8",
        NullByteInString { .. } => "whisper.null_byte",
        NullPointer => "whisper.null_pointer",
        NoSamples => "whisper.no_samples",
        InputOutputLengthMismatch { .. } => "whisper.sample_length",
        HalfSampleMissing(_) => "whisper.half_sample",
        GenericError(code) => return M::new("whisper.generic").number("code", (*code).into()),
    };
    M::new(code)
}
macro_rules! tensor_classifier {
    ($name:ident, $error:path $(, $start:pat)?) => {
        fn $name(error: &$error) -> M {
            use $error as E;
            match error {
                E::IoError(e) => M::io_cause(e),
                E::InvalidHeaderDeserialization(e) | E::JsonError(e) => M::new("tensor.header").caused_by(json(e)),
                E::InvalidHeader(_) | E::HeaderTooSmall | E::InvalidHeaderLength $(| $start)? => M::new("tensor.header"),
                E::HeaderTooLarge | E::ValidationOverflow => M::new("tensor.limit"),
                E::TensorNotFound(_) => M::new("tensor.missing"),
                E::InvalidOffset(_) | E::MetadataIncompleteBuffer => M::new("tensor.offset"),
                E::TensorInvalidInfo | E::InvalidTensorView(..) | E::MisalignedSlice => M::new("tensor.shape"),
            }
        }
    };
}
tensor_classifier!(
    tensor06,
    safetensors06::SafeTensorError,
    E::InvalidHeaderStart
);
tensor_classifier!(tensor08, safetensors08::SafeTensorError);
pub(crate) fn voxrt(error: &voxrt::Error) -> M {
    match error {
        voxrt::Error::Io(e) => M::io_cause(e),
        voxrt::Error::Json(e) => M::new("voxrt.json").caused_by(json(e)),
        voxrt::Error::SafeTensors(e) => M::new("voxrt.safetensors").caused_by(tensor06(e)),
        voxrt::Error::Format(_) => M::new("voxrt.format"),
        voxrt::Error::Audio(_) => M::new("voxrt.audio"),
    }
}
pub(crate) fn candle(error: &candle_core::Error) -> M {
    use candle_core::Error as E;
    match error {
        E::Io(e) => M::io_cause(e),
        E::Context { inner, .. } | E::WithPath { inner, .. } | E::WithBacktrace { inner, .. } => {
            candle(inner)
        }
        E::UnexpectedDType { .. }
        | E::DTypeMismatchBinaryOp { .. }
        | E::UnsupportedDTypeForOp(..)
        | E::UnsupportedSafeTensorDtype(_) => M::new("candle.dtype"),
        E::DimOutOfRange { .. }
        | E::DuplicateDimIndex { .. }
        | E::UnexpectedNumberOfDims { .. }
        | E::UnexpectedShape { .. }
        | E::ShapeMismatch { .. }
        | E::ShapeMismatchBinaryOp { .. }
        | E::ShapeMismatchCat { .. }
        | E::ShapeMismatchSplit { .. }
        | E::BroadcastIncompatibleShapes { .. } => M::new("candle.shape"),
        E::DeviceMismatchBinaryOp { .. } | E::Metal(_) => M::new("candle.device"),
        E::Cuda(error) => {
            M::new("candle.device").caused_by(boxed(error.as_ref(), "candle.backend"))
        }
        E::NotCompiledWithCudaSupport => M::new("candle.cuda_unavailable"),
        E::NotCompiledWithMetalSupport => M::new("candle.metal_unavailable"),
        E::CannotFindTensor { .. } => M::new("candle.missing_tensor"),
        E::SafeTensor(e) => M::new("candle.format").caused_by(tensor08(e)),
        E::Npy(_) | E::Zip(_) | E::ParseInt(_) | E::FromUtf8(_) => M::new("candle.format"),
        E::WrappedContext { wrapped, .. } => {
            M::new("candle.backend").caused_by(boxed(wrapped.as_ref(), "candle.backend"))
        }
        E::Wrapped(_) | E::Msg(_) | E::UnwrapNone => M::new("candle.backend"),
        _ => M::new("candle.tensor"),
    }
}
/// Only inspect typed sources; Display may contain input text, paths or secrets.
pub(crate) fn boxed(error: &(dyn std::error::Error + 'static), fallback: &'static str) -> M {
    let mut current = Some(error);
    while let Some(e) = current {
        if let Some(io) = e.downcast_ref::<std::io::Error>() {
            return M::io_cause(io);
        }
        if let Some(json_error) = e.downcast_ref::<serde_json::Error>() {
            return json(json_error);
        }
        if let Some(err) = e.downcast_ref::<tokenizers::models::bpe::Error>() {
            use tokenizers::models::bpe::Error as B;
            match err {
                B::BadVocabulary => return M::new("tokenizer.vocabulary"),
                B::BadMerges(_) => return M::new("tokenizer.merges"),
                B::MergeTokenOutOfVocabulary(_) | B::UnkTokenOutOfVocabulary(_) => {
                    return M::new("tokenizer.missing_token");
                }
                B::InvalidDropout => return M::new("tokenizer.config"),
                B::Io(_) | B::JsonError(_) => (),
            }
        }
        if e.is::<tokenizers::models::wordpiece::Error>() {
            return M::new("tokenizer.missing_token");
        }
        if let Some(err) = e.downcast_ref::<tokenizers::models::wordlevel::Error>() {
            return match err {
                tokenizers::models::wordlevel::Error::MissingUnkToken => {
                    M::new("tokenizer.missing_token")
                }
                tokenizers::models::wordlevel::Error::BadVocabulary => {
                    M::new("tokenizer.vocabulary")
                }
            };
        }
        current = e.source();
    }
    M::new(fallback)
}
pub(crate) fn loader(error: &libloading::Error) -> M {
    use libloading::Error as E;
    let code = match error {
        E::DlOpen { .. }
        | E::DlOpenUnknown
        | E::LoadLibraryExW { .. }
        | E::LoadLibraryExWUnknown
        | E::GetModuleHandleExW { .. }
        | E::GetModuleHandleExWUnknown => "loader.open",
        E::DlSym { .. } | E::DlSymUnknown | E::GetProcAddress { .. } | E::GetProcAddressUnknown => {
            "loader.symbol"
        }
        _ => "loader.other",
    };
    let mut message = M::new(code);
    if let Some(source) = std::error::Error::source(error) {
        if let Some(io) = source.downcast_ref::<std::io::Error>() {
            message = message.caused_by(M::io_cause(io));
        }
    }
    message
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_backend_variants_preserve_categories_without_private_values() {
        let generic = whisper(&whisper_rs::WhisperError::GenericError(-17));
        assert_eq!(generic.params["code"], -17);
        assert_ne!(
            whisper(&whisper_rs::WhisperError::FailedToDecode).code,
            whisper(&whisper_rs::WhisperError::FailedToEncode).code
        );
        let e = voxrt::Error::Io(std::io::Error::from_raw_os_error(13));
        assert_eq!(voxrt(&e).code, "io.permission_denied");
        let format = voxrt::Error::Format("secret /private/path".into());
        assert_eq!(voxrt(&format).code, "voxrt.format");
        assert!(!voxrt(&format).wire().contains("secret"));
        let c = candle_core::Error::WithPath {
            inner: Box::new(candle_core::Error::Io(std::io::Error::from_raw_os_error(2))),
            path: "/secret".into(),
        };
        assert_eq!(candle(&c).code, "io.not_found");
        assert!(!candle(&c).wire().contains("secret"));
        let st = voxrt::Error::SafeTensors(safetensors06::SafeTensorError::TensorNotFound(
            "private-tensor".into(),
        ));
        let message = voxrt(&st);
        assert_eq!(message.cause.as_ref().unwrap().code, "tensor.missing");
        assert!(!message.wire().contains("private-tensor"));
        let archive = candle_core::Error::SafeTensor(
            safetensors08::SafeTensorError::TensorNotFound("private tensor".into()),
        );
        assert_eq!(
            candle(&archive).cause.as_ref().unwrap().code,
            "tensor.missing"
        );
        assert!(!candle(&archive).wire().contains("private tensor"));
        let token =
            tokenizers::models::bpe::Error::MergeTokenOutOfVocabulary("private text".into());
        assert_eq!(
            boxed(&token, "backend.tokenizer").code,
            "tokenizer.missing_token"
        );
        assert!(
            !boxed(&token, "backend.tokenizer")
                .wire()
                .contains("private text")
        );
        assert_ne!(
            loader(&libloading::Error::DlOpenUnknown).code,
            loader(&libloading::Error::DlSymUnknown).code
        );
    }
}
