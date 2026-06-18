import AVFoundation
import XCTest
@testable import VaraCore

/// Proves the local media normalizer: ANY audio/video file → 16 kHz mono, the
/// format every Vara engine accepts.
///
/// Audio resample/downmix is exercised with tones generated in-process. The
/// headline video case uses committed fixtures (`Fixtures/clip-with-audio.mov`,
/// an H.264 + AAC .mov, and a video-only `clip-no-audio.mov`) rather than
/// encoding video in-process — deterministic, fast, and it confirms that
/// `AVAudioFile` (today's path) can't open the container while the normalizer can.
final class MediaNormalizerTests: XCTestCase {
    private var tmpDir: URL!

    override func setUpWithError() throws {
        tmpDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("media-normalizer-tests-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: tmpDir, withIntermediateDirectories: true)
    }

    override func tearDownWithError() throws {
        if let tmpDir { try? FileManager.default.removeItem(at: tmpDir) }
    }

    /// Locates a committed fixture relative to this source file (no resource
    /// bundling needed; the test repo is present at test time).
    private func fixture(_ name: String, _ ext: String) throws -> URL {
        let dir = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .appendingPathComponent("Fixtures")
        let url = dir.appendingPathComponent(name).appendingPathExtension(ext)
        guard FileManager.default.fileExists(atPath: url.path) else {
            throw XCTSkip("missing test fixture \(name).\(ext) at \(url.path)")
        }
        return url
    }

    // MARK: - Audio: resample + downmix

    func testResamplesStereo48kAudioToMono16kWAV() async throws {
        let src = tmpDir.appendingPathComponent("src.wav")
        try makeToneWAV(at: src, sampleRate: 48_000, channels: 2, seconds: 1.0)

        let out = try await MediaNormalizer.normalize(input: src, to: .wav, outputDirectory: tmpDir)
        let f = try AVAudioFile(forReading: out)

        XCTAssertEqual(f.fileFormat.sampleRate, 16_000, accuracy: 1)
        XCTAssertEqual(f.fileFormat.channelCount, 1)
        XCTAssertGreaterThan(f.length, 0)
        // ~1s at 16 kHz ≈ 16k frames (generous slack for resampler edges).
        XCTAssertEqual(Double(f.length), 16_000, accuracy: 2_000)
    }

    func testResamplesToMono16kAACAndCompresses() async throws {
        let src = tmpDir.appendingPathComponent("src.wav")
        try makeToneWAV(at: src, sampleRate: 48_000, channels: 2, seconds: 1.0)

        let aac = try await MediaNormalizer.normalize(input: src, to: .aacM4A, outputDirectory: tmpDir)
        let f = try AVAudioFile(forReading: aac)

        XCTAssertEqual(f.fileFormat.sampleRate, 16_000, accuracy: 1)
        XCTAssertEqual(f.fileFormat.channelCount, 1)
        XCTAssertGreaterThan(f.length, 0)

        // AAC is smaller than the equivalent 16 kHz mono WAV. (For a 1-second clip
        // the m4a's fixed container overhead keeps the margin modest; at real
        // recording length it amortizes to ~14 MB/hr vs ~115 MB/hr.)
        let wav = try await MediaNormalizer.normalize(input: src, to: .wav, outputDirectory: tmpDir)
        let aacSize = try fileSize(aac)
        let wavSize = try fileSize(wav)
        XCTAssertLessThan(aacSize, wavSize, "AAC (\(aacSize)B) should be smaller than WAV (\(wavSize)B)")
    }

    // MARK: - Video: extract the audio track

    func testExtractsAudioFromVideoContainer() async throws {
        let mov = try fixture("clip-with-audio", "mov")

        // The point: extract the audio track from a VIDEO container and deliver it
        // as 16 kHz mono, regardless of the source container/codec.
        let out = try await MediaNormalizer.normalize(input: mov, to: .wav, outputDirectory: tmpDir)
        let f = try AVAudioFile(forReading: out)

        XCTAssertEqual(f.fileFormat.sampleRate, 16_000, accuracy: 1)
        XCTAssertEqual(f.fileFormat.channelCount, 1)
        XCTAssertGreaterThan(f.length, 0)
    }

    func testExtractsAudioFromVideoToCompactAAC() async throws {
        let mov = try fixture("clip-with-audio", "mov")
        let aac = try await MediaNormalizer.normalize(input: mov, to: .aacM4A, outputDirectory: tmpDir)
        let f = try AVAudioFile(forReading: aac)

        XCTAssertEqual(f.fileFormat.sampleRate, 16_000, accuracy: 1)
        XCTAssertEqual(f.fileFormat.channelCount, 1)
        XCTAssertGreaterThan(f.length, 0)
    }

    func testThrowsWhenFileHasNoAudioTrack() async throws {
        let mov = try fixture("clip-no-audio", "mov")
        do {
            _ = try await MediaNormalizer.normalize(input: mov, to: .wav, outputDirectory: tmpDir)
            XCTFail("expected noAudioTrack error")
        } catch let error as MediaNormalizerError {
            XCTAssertEqual(error, .noAudioTrack)
        }
    }

    // MARK: - Container selection

    func testContainerChoiceFollowsBackendLocality() {
        XCTAssertEqual(NormalizedAudioContainer.forLocalBackend(true), .wav)
        XCTAssertEqual(NormalizedAudioContainer.forLocalBackend(false), .aacM4A)
    }

    // MARK: - Fixtures

    private func fileSize(_ url: URL) throws -> Int {
        let attrs = try FileManager.default.attributesOfItem(atPath: url.path)
        return (attrs[.size] as? Int) ?? 0
    }

    /// Writes a sine-tone WAV (float32 PCM) at an arbitrary rate/channel count.
    private func makeToneWAV(
        at url: URL,
        sampleRate: Double,
        channels: AVAudioChannelCount,
        seconds: Double,
        frequency: Double = 440
    ) throws {
        try? FileManager.default.removeItem(at: url)
        guard let format = AVAudioFormat(commonFormat: .pcmFormatFloat32,
                                         sampleRate: sampleRate,
                                         channels: channels,
                                         interleaved: false) else {
            throw XCTSkip("cannot build source format")
        }
        let file = try AVAudioFile(forWriting: url, settings: format.settings)
        let frames = AVAudioFrameCount(sampleRate * seconds)
        guard let buffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: frames) else {
            throw XCTSkip("cannot build source buffer")
        }
        buffer.frameLength = frames
        for ch in 0..<Int(channels) {
            let p = buffer.floatChannelData![ch]
            for i in 0..<Int(frames) {
                p[i] = Float(sin(2.0 * Double.pi * frequency * Double(i) / sampleRate)) * 0.5
            }
        }
        try file.write(from: buffer)
    }
}
