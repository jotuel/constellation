# Tasks

## 1. Data Model, Enums, and Routing

- [x] 1.1 Add `SpaceProfile` and `SpaceAccess` variants to `SettingsPanel` in `src/constellation/mod.rs` and verify with `cargo check`.
- [x] 1.2 Add `Message::OpenPanel(crate::SettingsPanel)` to `src/settings/space/message.rs` and route it to `self.handle_open_settings(panel)` in `src/constellation/handlers/update.rs`, and verify with `cargo check`.
- [x] 1.3 Update `handle_open_settings` in `src/constellation/handlers/rooms.rs` and search filter synchronization in `src/constellation/handlers/search.rs` to support space subpages, and verify with `cargo check`.
- [x] 1.4 Wire context drawer titles and subpage view dispatch for `SpaceProfile`, `SpaceAccess`, and `ManageSpaceRooms` in `src/constellation/app.rs`, and verify with `cargo check`.

## 2. Localization

- [x] 2.1 Add localization strings for space subpage titles, header card subtitles, and category row live summaries in `res/i18n/en/constellation.ftl`, and verify via string key presence.

## 3. Space Settings View Decomposition

- [x] 3.1 Implement `view_overview(&self)` in `src/settings/space/view.rs` using `header_card` and `category_row` from `crate::settings::widgets`, displaying live summaries for profile, discovery, and room count, and verify with `cargo check`.
- [x] 3.2 Implement `view_profile_page(&self)` in `src/settings/space/view.rs` isolating space profile editing, avatar upload, and save action, and verify with `cargo check`.
- [x] 3.3 Implement `view_access_page(&self)` in `src/settings/space/view.rs` isolating discovery toggles and save action, and verify with `cargo check`.
- [x] 3.4 Wire `view(&self)` to forward to `self.view_overview()` and verify `view_manage(&self)` renders cleanly as the hierarchy subpage, verifying with `cargo check`.

## 4. Testing and Verification

- [x] 4.1 Add unit tests in `src/settings/space/tests.rs` verifying subpage smoke rendering, `OpenPanel` message dispatch, and overview category row generation, and verify with `cargo test settings::space`.
- [x] 4.2 Add integration tests in `src/constellation/tests.rs` for space settings navigation stack push, pop, escape key handling, and deep linking, and verify with `cargo test constellation::tests::test_space_settings`.
- [x] 4.3 Run full workspace test suite via `cargo test` and verify zero errors or regressions.
- [x] 4.4 Run `cargo clippy --all-targets` and `cargo fmt -- --check` to ensure lint and formatting compliance with zero warnings.
