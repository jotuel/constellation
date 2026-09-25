# Tasks

## 1. Localization Strings

- [x] 1.1 Add Fluent string identifiers for room subpage titles (`room-profile-title`, `room-notifications-title`, `room-security-title`, `room-packs-title`) and overview category summaries in `res/i18n/en/constellation.ftl` and verify with `cargo check`

## 2. Navigation Stack & Panel Enums

- [x] 2.1 Add flat room subpage variants (`RoomProfile`, `RoomNotifications`, `RoomSecurity`, `RoomPacks`) to `SettingsPanel` enum in `src/constellation/mod.rs` and update match arms across `src/constellation/`
- [x] 2.2 Update `handle_open_settings` in `src/constellation/handlers/rooms.rs` to push `[SettingsPanel::Room, subpage]` onto `settings_stack` when navigating to any room subpage, and truncate stack on `SettingsPanel::Room`
- [x] 2.3 Update `context_drawer()` in `src/constellation/app.rs` to map titles and view dispatch for `RoomProfile`, `RoomNotifications`, `RoomSecurity`, and `RoomPacks`
- [x] 2.4 Add `is_room_settings_open(&self)` helper to `Constellation` in `src/constellation/mod.rs` and verify compilation with `cargo check`

## 3. Room Settings State & Responsive Controls

- [x] 3.1 Update `src/settings/room/state.rs` to maintain a `SingleSelectModel` and entity array for responsive notification mode selection, initializing them in `State::default()` and updating selection on room load and mode changes
- [x] 3.2 Update `src/settings/room/update.rs` to sync the segmented control model when `NotificationModeChanged` or `RoomLoaded` messages are processed, and verify with `cargo check`

## 4. Room Settings View Modularization

- [x] 4.1 Implement `view_overview(&self)` in `src/settings/room/view.rs` featuring the room header card (avatar, name, alias/ID) and clickable `Button::ListItem` category rows with live status summaries and drill-down chevrons navigating to each subpage
- [x] 4.2 Implement `view_profile_page(&self)` in `src/settings/room/view.rs` combining room avatar upload/preview, room name, topic, room ID, canonical alias, and alternative aliases management with save button and error banner
- [x] 4.3 Implement `view_notifications_page(&self)` in `src/settings/room/view.rs` featuring the horizontal `segmented_control` wrapped inside `settings::flex_item` for room notification mode selection
- [x] 4.4 Implement `view_security_page(&self)` in `src/settings/room/view.rs` combining E2E encryption status/enablement, join rule selection, history visibility, and restricted space configuration with save button and error banner
- [x] 4.5 Refine `view_permissions_page(&self)` in `src/settings/room/view.rs` for standalone subpage presentation with power level threshold controls, save button, and error banner
- [x] 4.6 Implement `view_packs_page(&self)` in `src/settings/room/view.rs` for room sticker and custom emoji image pack creation, management, and image upload/deletion
- [x] 4.7 Update `view(&self)` in `src/settings/room/view.rs` to route to `view_overview(&self)` and verify view compilation with `cargo check`

## 5. Verification & Tests

- [x] 5.1 Add unit tests in `src/constellation/tests.rs` verifying navigation stack push, pop, back button, and truncation behavior for all room subpages
- [x] 5.2 Add unit and state tests in `src/settings/room/tests.rs` covering subpage open messages, segmented control notification mode selection, and overview summary formatting
- [x] 5.3 Run full test suite with `cargo test` and verify code hygiene with `cargo clippy --all-targets` and `cargo fmt -- --check`
