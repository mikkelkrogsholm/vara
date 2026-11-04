# Contributing to Vara

Contributions are welcome, including documentation, bug fixes, UI translations and testing. Small, focused pull requests are easiest to review. Project communication and documentation are in English; the app's Danish locale remains supported alongside English.

## Get started

1. Fork the repository on GitHub, then clone your fork and create a branch:
   ```sh
   git clone https://github.com/YOUR-USERNAME/vara.git
   cd vara
   git switch -c your-change
   ```
2. Install the pinned Rust toolchain, CMake, a native C/C++ toolchain and the Tauri 2 platform prerequisites. On macOS, install Xcode command-line tools; the Who runtime requires Swift 6.2+. For the desktop app, install the Tauri CLI and run:
   ```sh
   cargo install tauri-cli --locked
   cargo tauri dev
   ```
   See the [desktop guide](desktop/README.md) for models and permissions. Agent tools are not required. An Apple Developer ID certificate is needed for the signed shipping build, not ordinary development.
3. Make your change and run the checks relevant to it. The lightweight version checks require Python 3.11 or newer and do not build the app:
   ```sh
   python3 scripts/check-version.py
   python3 -m unittest discover -s scripts/tests -p 'test_version.py'
   cargo test --workspace --locked
   ```
4. Commit and push your branch, then open a pull request from your fork to the upstream repository's `main` branch:
   ```sh
   git add <changed-files>
   git commit -m "Describe the change"
   git push -u origin your-change
   ```
   Explain the change and how you tested it; add screenshots or a recording for visible UI changes. Maintainers may ask for adjustments.

For bugs, use the issue form and include steps, expected and actual results, app version, operating system and relevant model. Feature ideas are welcome through the feature form. Please do not include private audio, transcripts, API keys, or other sensitive data in issues, logs, screenshots or pull requests; use synthetic or public examples.

macOS is the active tested desktop platform; Windows and Linux integration and release testing remain open. Public packaging and notarization are not ready. See [RELEASING.md](RELEASING.md) for versions and shipping, and [LICENSE](LICENSE) for licensing.
