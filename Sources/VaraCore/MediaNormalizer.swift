@preconcurrency import AVFoundation
import Foundation

/// Target container for a normalized audio file.
public enum NormalizedAudioContainer: Sendable {
    /// 16 kHz mono 16-bit linear PCM in a WAV container. Uncompressed → large
    /// (~115 MB/hour), but every on-device engine decodes it trivially and it
    /// matches the format the mic path already produces. Use for LOCAL backends.
    case wav
    /// 16 kHz mono AAC in an .m4a container (~32 kbps ≈ 14 MB/hour). ~10× smaller
    /// than WAV — fits cloud upload-size caps (OpenAI 25 MB) and uploads fast.
    /// Use for CLOUD backends.
    case aacM4A

    var fileExtension: String {
        switch self {
        case .wav: "wav"
        case .aacM4A: "m4a"
        }
    }

    /// The right container for a backend, chosen by where it runs: local engines
    /// want uncompressed WAV, cloud engines want compact AAC to dodge size caps.
    public static func forLocalBackend(_ isLocal: Bool) -> NormalizedAudioContainer {
        isLocal ? .wav : .aacM4A
    }
}

public enum MediaNormalizerError: Error, CustomStringConvertible, Equatable {
    /// The file has no audio track at all (e.g. a silent screen recording, an
    /// image, or a video exported without sound).
    case noAudioTrack
    case readerSetupFailed(String)
    case writerSetupFailed(String)
    case readFailed(String)
    case writeFailed(String)

    public var description: String {
        switch self {
        case .noAudioTrack:
            return "This file has no audio track to transcribe."
        case .readerSetupFailed(let r): return "Could not read the media file: \(r)"
        case .writerSetupFailed(let r): return "Could not prepare audio conversion: \(r)"
        case .readFailed(let r): return "Reading the audio failed: \(r)"
        case .writeFailed(let r): return "Writing the converted audio failed: \(r)"
        }
    }
}

/// Turns ANY media file — video (mp4/mov/mkv…) or audio (m4a/mp3/wav/flac…) —
/// into a clean 16 kHz mono file the speech engines accept.
///
/// It decodes through `AVURLAsset` / `AVAssetReader`, which extracts the audio
/// track from VIDEO containers too — unlike `AVAudioFile` (audio-only), which is
/// why raw video can't be fed to the engines directly. The work streams
/// sample-buffer by sample-buffer, so a multi-hour recording never loads whole
/// into memory.
///
/// Pure system frameworks: no ffmpeg, no bundled encoder, fully on-device.
/// macOS can't encode MP3 natively, so the compact path is AAC/m4a — equal-or-
/// better quality at 16 kHz mono and accepted by every Vara backend.
public enum MediaNormalizer {
    /// Decode `input`'s audio track and re-emit it as a 16 kHz mono file in
    /// `container`. Returns the URL of a freshly written temp file (the caller
    /// owns it and should delete it when done).
    public static func normalize(
        input: URL,
        to container: NormalizedAudioContainer,
        sampleRate: Double = 16_000,
        outputDirectory: URL? = nil
    ) async throws -> URL {
        let asset = AVURLAsset(url: input)
        let audioTracks: [AVAssetTrack]
        do {
            audioTracks = try await asset.loadTracks(withMediaType: .audio)
        } catch {
            throw MediaNormalizerError.readerSetupFailed(error.localizedDescription)
        }
        guard let track = audioTracks.first else {
            throw MediaNormalizerError.noAudioTrack
        }

        let dir = outputDirectory ?? FileManager.default.temporaryDirectory
        let output = dir
            .appendingPathComponent("vara-normalized-\(UUID().uuidString)")
            .appendingPathExtension(container.fileExtension)
        try? FileManager.default.removeItem(at: output)

        switch container {
        case .wav:
            try writeWAV(from: asset, track: track, to: output, sampleRate: sampleRate)
        case .aacM4A:
            try await writeAAC(from: asset, track: track, to: output, sampleRate: sampleRate)
        }
        return output
    }

    // MARK: - WAV (linear PCM) path

    /// Decode → 16 kHz mono Int16 PCM (the reader resamples + downmixes via its
    /// output settings) → stream to a WAV file, patching the header sizes at the
    /// end. No whole-file buffer: data is appended as it's read.
    private static func writeWAV(from asset: AVURLAsset, track: AVAssetTrack, to output: URL, sampleRate: Double) throws {
        let channels = 1
        let bitsPerSample = 16

        let reader: AVAssetReader
        do {
            reader = try AVAssetReader(asset: asset)
        } catch {
            throw MediaNormalizerError.readerSetupFailed(error.localizedDescription)
        }
        let pcmSettings: [String: Any] = [
            AVFormatIDKey: kAudioFormatLinearPCM,
            AVSampleRateKey: sampleRate,
            AVNumberOfChannelsKey: channels,
            AVLinearPCMBitDepthKey: bitsPerSample,
            AVLinearPCMIsFloatKey: false,
            AVLinearPCMIsBigEndianKey: false,
            AVLinearPCMIsNonInterleaved: false,
        ]
        let trackOutput = AVAssetReaderTrackOutput(track: track, outputSettings: pcmSettings)
        trackOutput.alwaysCopiesSampleData = false
        guard reader.canAdd(trackOutput) else {
            throw MediaNormalizerError.readerSetupFailed("cannot add PCM track output")
        }
        reader.add(trackOutput)
        guard reader.startReading() else {
            throw MediaNormalizerError.readFailed(reader.error?.localizedDescription ?? "startReading failed")
        }

        guard FileManager.default.createFile(atPath: output.path, contents: nil),
              let handle = try? FileHandle(forWritingTo: output) else {
            throw MediaNormalizerError.writeFailed("cannot open output for writing")
        }
        defer { try? handle.close() }
        try handle.write(contentsOf: wavHeader(sampleRate: Int(sampleRate), channels: channels, bitsPerSample: bitsPerSample, dataLength: 0))

        var dataLength = 0
        while reader.status == .reading {
            guard let sampleBuffer = trackOutput.copyNextSampleBuffer() else { break }
            defer { CMSampleBufferInvalidate(sampleBuffer) }
            guard let block = CMSampleBufferGetDataBuffer(sampleBuffer) else { continue }
            let length = CMBlockBufferGetDataLength(block)
            guard length > 0 else { continue }
            var bytes = Data(count: length)
            let status = bytes.withUnsafeMutableBytes { ptr in
                CMBlockBufferCopyDataBytes(block, atOffset: 0, dataLength: length, destination: ptr.baseAddress!)
            }
            guard status == kCMBlockBufferNoErr else {
                throw MediaNormalizerError.readFailed("CMBlockBufferCopyDataBytes status \(status)")
            }
            try handle.write(contentsOf: bytes)
            dataLength += length
        }

        if reader.status == .failed {
            throw MediaNormalizerError.readFailed(reader.error?.localizedDescription ?? "reader failed")
        }

        // Patch RIFF chunk size (offset 4) and data subchunk size (offset 40).
        try handle.seek(toOffset: 4)
        try handle.write(contentsOf: le32(UInt32(36 + dataLength)))
        try handle.seek(toOffset: 40)
        try handle.write(contentsOf: le32(UInt32(dataLength)))
    }

    // MARK: - AAC (.m4a) path

    /// Decode → 16 kHz mono Int16 PCM via the reader (resample + downmix), then
    /// encode synchronously to AAC through `AVAudioFile`. No `AVAssetWriter`: its
    /// AAC encoder consumes appended buffers asynchronously, so feeding it
    /// reader buffers (especially with `alwaysCopiesSampleData = false`) races the
    /// buffer's lifetime. `AVAudioFile.write` encodes inline, buffer-by-buffer,
    /// which is both correct and headless-safe.
    private static func writeAAC(from asset: AVURLAsset, track: AVAssetTrack, to output: URL, sampleRate: Double) async throws {
        let aacSettings: [String: Any] = [
            AVFormatIDKey: kAudioFormatMPEG4AAC,
            AVSampleRateKey: sampleRate,
            AVNumberOfChannelsKey: 1,
            AVEncoderBitRateKey: 32_000,
        ]
        let outFile: AVAudioFile
        do {
            outFile = try AVAudioFile(forWriting: output, settings: aacSettings)
        } catch {
            throw MediaNormalizerError.writerSetupFailed(error.localizedDescription)
        }
        // The PCM format `write(from:)` requires (always float32, mono here).
        let pcmFormat = outFile.processingFormat

        let reader: AVAssetReader
        do {
            reader = try AVAssetReader(asset: asset)
        } catch {
            throw MediaNormalizerError.readerSetupFailed(error.localizedDescription)
        }
        // Read float32 mono at the target rate (mono → interleaved == planar, so
        // it maps straight onto `processingFormat`'s single channel buffer).
        let readerSettings: [String: Any] = [
            AVFormatIDKey: kAudioFormatLinearPCM,
            AVSampleRateKey: sampleRate,
            AVNumberOfChannelsKey: 1,
            AVLinearPCMBitDepthKey: 32,
            AVLinearPCMIsFloatKey: true,
            AVLinearPCMIsBigEndianKey: false,
            AVLinearPCMIsNonInterleaved: false,
        ]
        let trackOutput = AVAssetReaderTrackOutput(track: track, outputSettings: readerSettings)
        trackOutput.alwaysCopiesSampleData = true
        guard reader.canAdd(trackOutput) else {
            throw MediaNormalizerError.readerSetupFailed("cannot add PCM track output")
        }
        reader.add(trackOutput)
        guard reader.startReading() else {
            throw MediaNormalizerError.readFailed(reader.error?.localizedDescription ?? "startReading failed")
        }

        let bytesPerFrame = Int(pcmFormat.streamDescription.pointee.mBytesPerFrame)
        while reader.status == .reading {
            guard let sampleBuffer = trackOutput.copyNextSampleBuffer() else { break }
            defer { CMSampleBufferInvalidate(sampleBuffer) }
            guard let block = CMSampleBufferGetDataBuffer(sampleBuffer) else { continue }
            let byteCount = CMBlockBufferGetDataLength(block)
            guard byteCount > 0, bytesPerFrame > 0 else { continue }
            let frames = AVAudioFrameCount(byteCount / bytesPerFrame)
            guard frames > 0,
                  let buffer = AVAudioPCMBuffer(pcmFormat: pcmFormat, frameCapacity: frames),
                  let dst = buffer.floatChannelData?[0] else { continue }
            buffer.frameLength = frames
            let status = CMBlockBufferCopyDataBytes(block, atOffset: 0, dataLength: byteCount, destination: dst)
            guard status == kCMBlockBufferNoErr else {
                throw MediaNormalizerError.readFailed("CMBlockBufferCopyDataBytes status \(status)")
            }
            do {
                try outFile.write(from: buffer)
            } catch {
                throw MediaNormalizerError.writeFailed(error.localizedDescription)
            }
        }

        if reader.status == .failed {
            throw MediaNormalizerError.readFailed(reader.error?.localizedDescription ?? "reader failed")
        }
        // `outFile` finalizes the .m4a on deinit at end of scope.
    }

    // MARK: - Helpers

    private static func wavHeader(sampleRate: Int, channels: Int, bitsPerSample: Int, dataLength: Int) -> Data {
        let byteRate = sampleRate * channels * bitsPerSample / 8
        let blockAlign = channels * bitsPerSample / 8
        var d = Data()
        d.append(contentsOf: Array("RIFF".utf8))
        d.append(le32(UInt32(36 + dataLength)))
        d.append(contentsOf: Array("WAVE".utf8))
        d.append(contentsOf: Array("fmt ".utf8))
        d.append(le32(16))                       // PCM fmt chunk size
        d.append(le16(1))                        // audioFormat = PCM
        d.append(le16(UInt16(channels)))
        d.append(le32(UInt32(sampleRate)))
        d.append(le32(UInt32(byteRate)))
        d.append(le16(UInt16(blockAlign)))
        d.append(le16(UInt16(bitsPerSample)))
        d.append(contentsOf: Array("data".utf8))
        d.append(le32(UInt32(dataLength)))
        return d
    }

    private static func le32(_ v: UInt32) -> Data {
        var le = v.littleEndian
        return withUnsafeBytes(of: &le) { Data($0) }
    }

    private static func le16(_ v: UInt16) -> Data {
        var le = v.littleEndian
        return withUnsafeBytes(of: &le) { Data($0) }
    }
}

