import AVFoundation
import Foundation
import Speech

/// On-device speech-to-text via Apple's `SpeechAnalyzer` / `SpeechTranscriber`
/// (macOS 26+). Fully local, no API key, strong Danish (`da_DK` is first-class),
/// and — unlike WhisperKit — the model is a *system-managed* per-locale asset
/// (downloaded once via `AssetInventory`, shared across apps, not counted in the
/// bundle). This is Vara's cleanest "fully offline" engine.
///
/// HARD-GATED to macOS 26: the whole type is `@available(macOS 26, *)` and must
/// only be instantiated behind `if #available(macOS 26, *)`. It needs NO Apple
/// Intelligence — that's a separate subsystem from the Foundation Models LLM.
@available(macOS 26, *)
public struct AppleSpeechTranscriberBackend: SpeechToTextBackend {
    public let id = "apple-speech"
    public let displayName = "Apple Speech (on-device)"
    public var shortName: String { "Apple Speech" }
    public let capabilities = SpeechToTextBackendCapabilities(
        supportsFileTranscription: true,
        supportsStreamingTranscription: true,
        supportsLanguageHints: true,
        supportsTimestamps: true,
        runsLocally: true,
        // Steady-state truth: transcription is fully local. Only the one-time
        // per-locale asset download touches the network (handled below).
        requiresNetwork: false,
        requiredCredentialKeys: [],
        // We convert to the analyzer's preferred format anyway, but the mic path
        // feeds 16 kHz like every other Vara backend.
        supportedSampleRates: [16_000]
    )

    private let diagnosticLog: (@Sendable (String) async -> Void)?

    public init(diagnosticLog: (@Sendable (String) async -> Void)? = nil) {
        self.diagnosticLog = diagnosticLog
    }

    /// Whether SpeechTranscriber can handle the given language on this OS build
    /// (used by the UI to decide whether to even offer the engine for the user's
    /// dictation language). Matches by language code, not full `Locale` equality.
    public static func isLanguageSupported(_ languageCode: String?) async -> Bool {
        await resolveLocale(for: languageCode) != nil
    }

    /// The 2-letter language codes SpeechTranscriber supports on this OS build
    /// (e.g. `["en", "fr", "de", …]`) — empty on failure. The UI uses this to hide
    /// the engine for languages Apple can't do yet (notably Danish on macOS 26.2),
    /// and it auto-recovers if a future macOS adds them.
    public static func supportedLanguageCodes() async -> Set<String> {
        let supported = await SpeechTranscriber.supportedLocales
        return Set(supported.compactMap { $0.language.languageCode?.identifier })
    }

    public func transcribe(_ request: SpeechToTextRequest) async throws -> TranscriptionResult {
        let startedAt = Date()

        guard let locale = await Self.resolveLocale(for: request.languageCode) else {
            throw SpeechToTextBackendError.unsupportedAudio(
                backendID: id,
                reason: "Apple Speech has no on-device model for language '\(request.languageCode ?? "auto")'."
            )
        }

        // No volatile results for the batch path: every emitted result is final,
        // so we can append them all without the volatile/final dedup dance. (The
        // live-HUD streaming feature will opt into .volatileResults later.)
        let transcriber = SpeechTranscriber(
            locale: locale,
            transcriptionOptions: [],
            reportingOptions: [],
            attributeOptions: []
        )
        let analyzer = SpeechAnalyzer(modules: [transcriber])

        try await ensureModelInstalled(for: transcriber, locale: locale)

        // Consume finalized results concurrently while we feed audio.
        let resultsTask = Task {
            var text = ""
            for try await result in transcriber.results {
                text += String(result.text.characters)
            }
            return text
        }

        let durationSeconds: Double
        do {
            switch request.audio {
            case .file(let url):
                // Let the analyzer decode/resample the file itself — no manual
                // conversion needed for the file path.
                let file = try AVAudioFile(forReading: url)
                durationSeconds = Double(file.length) / file.fileFormat.sampleRate
                try await analyzer.start(inputAudioFile: file, finishAfterFile: true)
                try await analyzer.finalizeAndFinishThroughEndOfInput()

            case .pcm16Stream(let stream, let sampleRate):
                guard let analyzerFormat = await SpeechAnalyzer.bestAvailableAudioFormat(compatibleWith: [transcriber]) else {
                    throw SpeechToTextBackendError.unsupportedAudio(backendID: id, reason: "no compatible analyzer audio format")
                }
                guard let sourceFormat = AVAudioFormat(commonFormat: .pcmFormatInt16, sampleRate: Double(sampleRate), channels: 1, interleaved: true),
                      let converter = AVAudioConverter(from: sourceFormat, to: analyzerFormat) else {
                    throw SpeechToTextBackendError.unsupportedAudio(backendID: id, reason: "could not build audio converter for \(sampleRate) Hz")
                }

                let (inputSequence, inputContinuation) = AsyncStream<AnalyzerInput>.makeStream()
                try await analyzer.start(inputSequence: inputSequence)

                var totalBytes = 0
                for await chunk in stream {
                    totalBytes += chunk.count
                    guard let source = Self.makeInt16Buffer(chunk, format: sourceFormat),
                          let converted = Self.resample(source, with: converter, to: analyzerFormat) else { continue }
                    inputContinuation.yield(AnalyzerInput(buffer: converted))
                }
                inputContinuation.finish()
                try await analyzer.finalizeAndFinishThroughEndOfInput()
                durationSeconds = Double(totalBytes / 2) / Double(sampleRate)
            }
        } catch {
            resultsTask.cancel()
            throw error
        }

        let text = try await resultsTask.value
        await diagnosticLog?("Apple Speech completed. locale=\(locale.identifier), chars=\(text.count), elapsed=\(String(format: "%.2f", Date().timeIntervalSince(startedAt)))s")

        return TranscriptionResult(
            text: text.trimmingCharacters(in: .whitespacesAndNewlines),
            detectedLanguage: locale.language.languageCode?.identifier,
            languageConfidence: nil,
            audioDurationSeconds: durationSeconds,
            elapsedSeconds: Date().timeIntervalSince(startedAt),
            backend: displayName,
            rawJSON: nil
        )
    }

    // MARK: - Locale resolution

    /// The best supported `Locale` for a Vara language code ("da", "da-DK"), or
    /// nil if SpeechTranscriber has no model for that language. Prefers an exact
    /// region match, else the first supported locale for the language.
    static func resolveLocale(for languageCode: String?) async -> Locale? {
        let supported = await SpeechTranscriber.supportedLocales
        guard !supported.isEmpty else { return nil }

        let raw = (languageCode?.isEmpty == false ? languageCode! : "en")
        let requested = Locale(identifier: raw.replacingOccurrences(of: "-", with: "_"))
        if let exact = supported.first(where: { $0.identifier(.bcp47) == requested.identifier(.bcp47) }) {
            return exact
        }
        let lang = String(raw.prefix(2)).lowercased()
        return supported.first { $0.language.languageCode?.identifier == lang }
    }

    // MARK: - Asset installation

    /// Ensures the per-locale on-device model is installed (downloads on first
    /// use). The asset is system-managed and shared across apps; first use of a
    /// new language needs connectivity.
    private func ensureModelInstalled(for transcriber: SpeechTranscriber, locale: Locale) async throws {
        let installed = await SpeechTranscriber.installedLocales
        if installed.contains(where: { $0.identifier(.bcp47) == locale.identifier(.bcp47) }) { return }

        await diagnosticLog?("Apple Speech: installing on-device model for \(locale.identifier) (one-time)…")
        if let request = try await AssetInventory.assetInstallationRequest(supporting: [transcriber]) {
            try await request.downloadAndInstall()
        }
        // Reserve the active locale so the system keeps it ready (best-effort —
        // an over-cap reservation just means it isn't pinned).
        _ = try? await AssetInventory.reserve(locale: locale)
        await diagnosticLog?("Apple Speech: model ready for \(locale.identifier).")
    }

    // MARK: - Audio bridging (Vara's Int16 stream -> analyzer format)

    /// Wrap a chunk of interleaved Int16 PCM `Data` into an `AVAudioPCMBuffer`.
    static func makeInt16Buffer(_ data: Data, format: AVAudioFormat) -> AVAudioPCMBuffer? {
        let frames = AVAudioFrameCount(data.count / 2)
        guard frames > 0,
              let buffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: frames),
              let channel = buffer.int16ChannelData else { return nil }
        buffer.frameLength = frames
        data.withUnsafeBytes { raw in
            if let base = raw.baseAddress {
                channel[0].update(from: base.assumingMemoryBound(to: Int16.self), count: Int(frames))
            }
        }
        return buffer
    }

    /// Convert one buffer to the analyzer's format (handles sample-rate change).
    /// The single `converter` is reused across chunks so resampling stays
    /// continuous at chunk boundaries. The one-shot feed state lives in a
    /// reference type so the `@Sendable` input block captures no mutable var or
    /// non-Sendable buffer directly (the conversion runs synchronously here).
    static func resample(_ input: AVAudioPCMBuffer, with converter: AVAudioConverter, to format: AVAudioFormat) -> AVAudioPCMBuffer? {
        let ratio = format.sampleRate / input.format.sampleRate
        let capacity = AVAudioFrameCount(Double(input.frameLength) * ratio) + 1024
        guard let output = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: capacity) else { return nil }

        var conversionError: NSError?
        let status = converter.convert(to: output, error: &conversionError, withInputFrom: ConverterFeed(buffer: input).next)
        guard status != .error, output.frameLength > 0 else { return nil }
        return output
    }
}

/// One-shot input provider for `AVAudioConverter.convert(to:error:withInputFrom:)`:
/// hands the single buffer over once, then reports `.noDataNow`. `@unchecked
/// Sendable` because the converter calls `next` synchronously on the same thread
/// during `convert`, so the wrapped non-Sendable buffer never actually escapes.
@available(macOS 26, *)
private final class ConverterFeed: @unchecked Sendable {
    private let buffer: AVAudioPCMBuffer
    private var consumed = false

    init(buffer: AVAudioPCMBuffer) { self.buffer = buffer }

    func next(_ packetCount: AVAudioPacketCount, _ status: UnsafeMutablePointer<AVAudioConverterInputStatus>) -> AVAudioPCMBuffer? {
        if consumed {
            status.pointee = .noDataNow
            return nil
        }
        consumed = true
        status.pointee = .haveData
        return buffer
    }
}
