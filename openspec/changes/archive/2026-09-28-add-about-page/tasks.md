# Tasks

## 1. Dependency Configuration

- [x] 1.1 Add `"about"` feature to the `libcosmic` dependency in `Cargo.toml` and verify compilation with `cargo check`.

## 2. Localization Strings

- [x] 2.1 Add `about = About` and related descriptions to `res/i18n/en/constellation.ftl` and verify with string lookup tests.

## 3. About Metadata Constructor

- [x] 3.1 Implement `about_info()` constructor in `src/view/about.rs` returning `cosmic::widget::about::About` populated with application metadata (name, version, author, license, and project links) and verify with unit tests.

## 4. Navigation & State Integration

- [x] 4.1 Add `About` variant to `SettingsPanel` and `MenuAct` in `src/constellation/mod.rs`, map `MenuAct::About` to `Message::OpenSettings(SettingsPanel::About)`, and verify with existing settings panel tests.
- [x] 4.2 Add "About" option to the header user dropdown menu in `src/constellation/app.rs` and verify menu item presence in view tests.
- [x] 4.3 Add "About" category row to the App Settings overview in `src/settings/app/view.rs` and verify overview rendering tests in `src/settings/app/tests.rs`.

## 5. Context Drawer & End-to-End Verification

- [x] 5.1 Wire `SettingsPanel::About` in `src/constellation/app.rs` `context_drawer()` to render `cosmic::app::context_drawer::about` with `Message::OpenUrl` link callbacks and back navigation.
- [x] 5.2 Run the full test suite with `cargo test` and verify code quality with `cargo clippy --all-targets`.
