# Tasks

## 1. Create Settings Widgets Submodule

- [x] 1.1 Create `src/settings/widgets/navigation.rs` implementing `category_row`, `header_card`, and `avatar_box`, and verify via compilation.
- [x] 1.2 Create `src/settings/widgets/feedback.rs` implementing `view_error` and `save_button`, and verify via compilation.
- [x] 1.3 Create `src/settings/widgets/notifications.rs` implementing `NotificationModeSelector` with `new`, `set_mode`, `control`, `Clone`, and `Default`, and verify via compilation.
- [x] 1.4 Wire `src/settings/widgets/mod.rs` to re-export the widgets and register `pub mod widgets;` in `src/settings/mod.rs`, and verify with `cargo check`.
- [x] 1.5 Add comprehensive unit tests in `src/settings/widgets/tests.rs` covering category row rendering, save button states, error banner display, and `NotificationModeSelector` entity syncing and cloning, and verify with `cargo test settings::widgets`.

## 2. Migrate User Settings

- [x] 2.1 Refactor `src/settings/user/state.rs` to replace manual `SingleSelectModel` fields with `dm_notification_selector` and `group_notification_selector`, delete `create_notification_mode_model`, and replace the manual `impl Clone for State` with `#[derive(Clone)]`, verifying with `cargo check`.
- [x] 2.2 Update `src/settings/user/update.rs` to synchronize notification modes through the selectors, verifying with `cargo check`.
- [x] 2.3 Refactor `src/settings/user/view.rs` to consume `category_row`, `header_card`, `view_error`, and `selector.control()`, deleting local closures and duplicate card construction, and verify with `cargo check`.
- [x] 2.4 Update and verify `src/settings/user/tests.rs` to ensure all existing user settings unit and smoke tests pass via `cargo test settings::user`.

## 3. Migrate Room Settings

- [x] 3.1 Refactor `src/settings/room/state.rs` to replace manual `SingleSelectModel` fields with `notification_selector`, delete `create_notification_mode_model`, and replace the manual `impl Clone for State` with `#[derive(Clone)]`, verifying with `cargo check`.
- [x] 3.2 Update `src/settings/room/update.rs` to synchronize notification mode through `notification_selector.set_mode()`, verifying with `cargo check`.
- [x] 3.3 Refactor `src/settings/room/view.rs` to consume `category_row`, `header_card`, `view_error`, and `notification_selector.control()`, deleting local closures and duplicate card construction, and verify with `cargo check`.
- [x] 3.4 Update and verify `src/settings/room/tests.rs` to ensure all existing room settings unit and smoke tests pass via `cargo test settings::room`.

## 4. Migrate Space Settings

- [x] 4.1 Refactor `src/settings/space/view.rs` to consume `view_error`, `save_button`, and `avatar_box`, eliminating duplicate error section, save button dirty-checking, and avatar placeholder code, and verify with `cargo check`.
- [x] 4.2 Verify space settings tests pass via `cargo test settings::space`.

## 5. Verification and Linting

- [x] 5.1 Run full workspace test suite `cargo test` and verify all tests pass without errors or regressions.
- [x] 5.2 Run `cargo clippy --all-targets` and `cargo fmt -- --check` to ensure lint and formatting compliance with zero warnings.
