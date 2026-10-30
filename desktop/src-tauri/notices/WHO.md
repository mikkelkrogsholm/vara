# Who by Desert Ant Labs

Optional local speaker diarization, adapted desert-ant-core SDK 4.2.1,
revision 1a5efed67f44731b5eb5433aa9be3eebae05e269, model v0.3.4.
Vara adds a build-time lane-export patch for optional known-count grouping.

Source: https://github.com/Desert-Ant-Labs/desert-ant-core
Model: https://huggingface.co/desert-ant-labs/who
Provider / attribution: https://desertant.com
License: https://license.desertant.com/1.0

This is source-available software under the Desert Ant Labs Source-Available
License 1.0, not an MIT/Apache model. Models are downloaded by the SDK
on explicit user request and are not included in the Vara application bundle.
The official SDK's license/usage reporting is retained. It sends an installation
identifier and model/app/platform metadata to Desert Ant Labs; no audio or text.
The UI discloses this before download and use. No Vara account or payment service
is added. The free-tier threshold and all other conditions are in the license.

The app's Rust core uses the SDK's stable C ABI in an isolated, cancellable child
process. No former Vara Swift application code is reintroduced.
