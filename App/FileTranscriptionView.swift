import AppKit
import SwiftUI
import UniformTypeIdentifiers
import VaraCore

/// Drives the "transcribe a file" window: pick → normalize+transcribe (via
/// `AppState.transcribeFile`) → show the transcript. Separate from `AppState` so
/// the dictation hot path stays untouched; it just calls into it.
@MainActor
@Observable
final class FileTranscriptionModel {
    enum Phase: Equatable {
        case idle
        case running(fileName: String)
        case done(fileName: String)
        case failed(message: String)
    }

    private(set) var phase: Phase = .idle
    private(set) var transcript: String = ""
    private(set) var engineName: String = ""
    private(set) var elapsedSeconds: Double = 0

    var isRunning: Bool {
        if case .running = phase { return true }
        return false
    }

    private var task: Task<Void, Never>?

    /// Normalize + transcribe `url` through the app's active engine, driving the
    /// window's phase. Cancels any in-flight run first.
    func start(url: URL, appState: AppState) {
        task?.cancel()
        let name = url.lastPathComponent
        transcript = ""
        elapsedSeconds = 0
        engineName = appState.backend?.shortName ?? ""
        phase = .running(fileName: name)

        task = Task { [weak self] in
            do {
                let result = try await appState.transcribeFile(at: url)
                guard let self, !Task.isCancelled else { return }
                self.transcript = result.text
                self.elapsedSeconds = result.elapsedSeconds
                self.engineName = result.backend
                self.phase = .done(fileName: name)
            } catch is CancellationError {
                // Superseded by a newer pick — leave the newer run's state alone.
            } catch {
                guard let self, !Task.isCancelled else { return }
                self.phase = .failed(message: Self.message(for: error))
            }
        }
    }

    func cancel() {
        task?.cancel()
        task = nil
        phase = .idle
    }

    private static func message(for error: Error) -> String {
        if let localized = error as? LocalizedError, let description = localized.errorDescription {
            return description
        }
        return String(describing: error)
    }
}

/// File picker + save helpers for the file-transcription window.
@MainActor
enum MediaFilePicker {
    static func pick() -> URL? {
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.canChooseFiles = true
        panel.allowedContentTypes = [.audio, .movie, .video, .mpeg4Movie, .quickTimeMovie]
        panel.message = String(localized: "Choose an audio or video file to transcribe",
                               comment: "File-transcription open-panel prompt")
        NSApp.activate(ignoringOtherApps: true)
        return panel.runModal() == .OK ? panel.url : nil
    }

    static func save(text: String, suggestedName: String) {
        let panel = NSSavePanel()
        panel.allowedContentTypes = [.plainText]
        panel.nameFieldStringValue = (suggestedName as NSString).deletingPathExtension + ".txt"
        NSApp.activate(ignoringOtherApps: true)
        if panel.runModal() == .OK, let url = panel.url {
            try? text.data(using: .utf8)?.write(to: url, options: .atomic)
        }
    }
}

struct FileTranscriptionView: View {
    let model: FileTranscriptionModel
    let state: AppState

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            header
            Divider()
            content
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        }
        .padding(20)
        .frame(minWidth: 540, minHeight: 440)
    }

    private var header: some View {
        HStack(alignment: .top) {
            VStack(alignment: .leading, spacing: 3) {
                Text("Transcribe a file", comment: "File-transcription window title")
                    .font(.title2.weight(.semibold))
                    .foregroundStyle(BSTheme.textPrimary)
                Text("Audio or video — converted on your Mac, then transcribed by your engine.",
                     comment: "File-transcription window subtitle")
                    .font(.callout)
                    .foregroundStyle(BSTheme.textMuted)
            }
            Spacer()
            Button {
                pickAndStart()
            } label: {
                Label(LocalizedStringKey("Choose File…"), systemImage: "folder")
            }
            .controlSize(.large)
            .disabled(model.isRunning)
        }
    }

    @ViewBuilder
    private var content: some View {
        switch model.phase {
        case .idle:
            placeholder(
                icon: "waveform.and.mic",
                title: String(localized: "Pick a recording to transcribe", comment: "File-transcription idle title"),
                subtitle: String(localized: "Works with audio and video files of any common format.",
                                 comment: "File-transcription idle subtitle")
            )
        case .running(let name):
            VStack(alignment: .leading, spacing: 12) {
                ProgressView().controlSize(.small)
                Text(name).font(.headline).foregroundStyle(BSTheme.textPrimary)
                Text(engineLine).font(.callout).foregroundStyle(BSTheme.textMuted)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.top, 8)
        case .done(let name):
            resultView(fileName: name)
        case .failed(let message):
            failureView(message)
        }
    }

    private func resultView(fileName: String) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text(fileName).font(.headline).foregroundStyle(BSTheme.textPrimary)
                    Text(engineLine).font(.caption).foregroundStyle(BSTheme.textMuted)
                }
                Spacer()
                Button {
                    ClipboardPaster.copyToClipboard(text: model.transcript)
                } label: {
                    Label(LocalizedStringKey("Copy"), systemImage: "doc.on.doc")
                }
                Button {
                    MediaFilePicker.save(text: model.transcript, suggestedName: fileName)
                } label: {
                    Label(LocalizedStringKey("Save…"), systemImage: "square.and.arrow.down")
                }
            }

            ScrollView {
                Text(model.transcript.isEmpty ? "—" : model.transcript)
                    .font(.system(size: 13))
                    .foregroundStyle(BSTheme.textPrimary)
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(12)
            }
            .background(BSTheme.surfaceHover.opacity(0.5), in: RoundedRectangle(cornerRadius: 8))
        }
    }

    private func failureView(_ message: String) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Label(LocalizedStringKey("Couldn't transcribe this file"), systemImage: "exclamationmark.triangle")
                .font(.headline)
                .foregroundStyle(BSTheme.stateError)
            Text(message)
                .font(.callout)
                .foregroundStyle(BSTheme.textMuted)
                .textSelection(.enabled)
            Button {
                pickAndStart()
            } label: {
                Label(LocalizedStringKey("Try another file"), systemImage: "arrow.clockwise")
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.top, 8)
    }

    private func placeholder(icon: String, title: String, subtitle: String) -> some View {
        VStack(spacing: 8) {
            Image(systemName: icon)
                .font(.system(size: 34))
                .foregroundStyle(BSTheme.textMuted)
            Text(title).font(.headline).foregroundStyle(BSTheme.textPrimary)
            Text(subtitle).font(.callout).foregroundStyle(BSTheme.textMuted)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var engineLine: String {
        let engine = engineNameOrFallback
        if case .done = model.phase, model.elapsedSeconds > 0 {
            return String(
                format: String(localized: "%@ · %.1fs", comment: "File-transcription engine + elapsed time"),
                engine, model.elapsedSeconds
            )
        }
        return engine
    }

    private var engineNameOrFallback: String {
        model.engineName.isEmpty
            ? (state.backend?.shortName ?? String(localized: "No engine", comment: "File-transcription: no engine configured"))
            : model.engineName
    }

    private func pickAndStart() {
        guard let url = MediaFilePicker.pick() else { return }
        model.start(url: url, appState: state)
    }
}
