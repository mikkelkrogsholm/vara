import Foundation
import FoundationModels
import VaraCore

/// On-device LLM cleanup via Apple's Foundation Models (macOS 26 + an
/// Apple-Intelligence-capable Mac). No API key, no network — the system model
/// powers Vara's `Ren tekst` / `Professionel` / `Spørg AI` modes fully locally.
///
/// Lives in the App target (not VaraCore) so the reusable core + VaraCLI stay
/// free of the macOS-26-only `FoundationModels` dependency. Maps Vara's
/// system/user split to the framework's instructions/prompt. On any failure —
/// model unavailable, the ~4k-token context window exceeded, or a guardrail
/// refusal — it throws, and `DictationModeProcessor` falls back to the raw
/// transcript (dictation never loses the speaker's words).
@available(macOS 26, *)
struct FoundationModelsChatClient: ChatClient {
    let id = "apple-foundation-models"

    func complete(system: String, user: String) async throws -> String {
        switch SystemLanguageModel.default.availability {
        case .available:
            break
        case .unavailable(let reason):
            throw FoundationModelsChatError.unavailable(reason)
        }

        // Fresh session per call: instructions = the mode's system prompt, prompt
        // = the framed transcript. Not reused, so stale turns never eat the window.
        let session = LanguageModelSession(instructions: system)
        let response = try await session.respond(to: user)

        let text = response.content.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { throw FoundationModelsChatError.emptyCompletion }
        return text
    }
}

/// Errors surfaced to the modes layer (which logs + falls back to raw). None
/// carry the transcript — Foundation Models errors describe the reason, not the
/// input — so they are safe to log.
@available(macOS 26, *)
enum FoundationModelsChatError: Error, CustomStringConvertible {
    case unavailable(SystemLanguageModel.Availability.UnavailableReason)
    case emptyCompletion

    var description: String {
        switch self {
        case .unavailable(let reason):
            switch reason {
            case .deviceNotEligible:
                return "Apple Foundation Models: this Mac isn't eligible for Apple Intelligence."
            case .appleIntelligenceNotEnabled:
                return "Apple Foundation Models: turn on Apple Intelligence in System Settings."
            case .modelNotReady:
                return "Apple Foundation Models: the model is still downloading — try again shortly."
            @unknown default:
                return "Apple Foundation Models unavailable."
            }
        case .emptyCompletion:
            return "Apple Foundation Models returned no text."
        }
    }
}
