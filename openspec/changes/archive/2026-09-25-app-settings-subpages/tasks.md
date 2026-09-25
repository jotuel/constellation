# Tasks

## 1. Data Model, Enums, and Routing

- [x] 1.1 Add `AppAppearance`, `AppNotifications`, and `AppMaintenance` variants to `SettingsPanel` in `src/constellation/mod.rs` and verify with `cargo check`.
- [x] 1.2 Update `handle_open_settings` in `src/constellation/handlers/rooms.rs` and message routing in `src/constellation/handlers/update.rs` to support App Settings subpages and `OpenPanel` routing, and verify with `cargo check`.
- [x] 1.3 Wire context drawer titles and subpage view dispatch for `AppAppearance`, `AppNotifications`, and `AppMaintenance` in `src/constellation/app.rs`, and verify with `cargo check`.

## 2. App Settings Directory Modularization

- [x] 2.1 Create `src/settings/app/state.rs` extracting `SessionError` and `State`, and `src/settings/app/message.rs` extracting `Message` with `OpenPanel(crate::SettingsPanel)`, and verify with `cargo check`.
- [x] 2.2 Create `src/settings/app/update.rs` extracting `State::update()`, and verify with `cargo check`.
- [x] 2.3 Create `src/settings/app/view.rs` implementing `view_overview`, `view_appearance_page`, `view_notifications_page`, `view_maintenance_page`, and `view()`, using `category_row` from `crate::settings::widgets`, and verify with `cargo check`.
- [x] 2.4 Create `src/settings/app/mod.rs` re-exporting `Message`, `SessionError`, and `State`, and remove old `src/settings/app.rs`, verifying with `cargo check`.

## 3. Localization

- [x] 3.1 Add localization strings for App subpage titles and overview category live summaries in `res/i18n/en/constellation.ftl`, and verify via string key presence.

## 4. Testing and Verification

- [x] 4.1 Create `src/settings/app/tests.rs` migrating existing unit tests and adding view smoke tests for all subpages, and verify with `cargo test settings::app`.
- [x] 4.2 Add integration tests in `src/constellation/tests.rs` for App Settings navigation stack push, pop, escape key handling, and deep linking, and verify with `cargo test constellation::tests::test_app_settings`.
- [x] 4.3 Run full workspace test suite via `cargo test` and verify zero errors or regressions.
- [x] 4.4 Run `cargo clippy --all-targets` and `cargo fmt -- --check` to ensure lint and formatting compliance with zero warnings.
