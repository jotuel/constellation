# Tasks

## 1. Localization Strings

- [x] 1.1 Add Fluent string identifiers for subpage titles and category summaries in `res/i18n/en/constellation.ftl`

## 2. Navigation Stack Architecture

- [x] 2.1 Update `SettingsPanel` enum in `src/constellation/mod.rs` with flat user settings subpage variants (`UserProfile`, `UserNotifications`, `UserPrivacy`, `UserSessions`, `UserAccount`, `UserPacks`)
- [x] 2.2 Add `settings_stack: Vec<SettingsPanel>` to `Constellation` state in `src/constellation/mod.rs` and provide compatibility helper methods
- [x] 2.3 Implement stack push, pop, clear operations and add `Message::SettingsBack` in `src/constellation/mod.rs` and `update.rs`
- [x] 2.4 Update `handle_open_settings` and `handle_toggle_settings_panel` in handlers to manage `settings_stack`
- [x] 2.5 Update `context_drawer()` in `src/constellation/app.rs` to render the `< Back` action in `ContextDrawer::actions` when `settings_stack.len() > 1` and map active subpage titles/views
- [x] 2.6 Update keyboard shortcut and event handling in `src/constellation/handlers/shortcuts.rs` so the Escape key pops the navigation stack when `len() > 1` before closing

## 3. User Settings View Modularization

- [x] 3.1 Implement `view_overview(&self)` in `src/settings/user/view.rs` with clickable `ListItem` category rows, status summaries, and `go-next-symbolic` chevrons
- [x] 3.2 Implement `view_profile_page(&self)` in `src/settings/user/view.rs` grouping avatar upload/removal, display name editor, and 3PID management
- [x] 3.3 Implement `view_notifications_page(&self)` in `src/settings/user/view.rs` with `segmented_control` within `settings::flex_item` for DM and Group notification defaults, alongside keyword management
- [x] 3.4 Implement `view_privacy_page(&self)` in `src/settings/user/view.rs` for media previews, invite avatars, and ignored users
- [x] 3.5 Implement `view_sessions_page(&self)` in `src/settings/user/view.rs` for device listings, interactive verification UI (SAS/QR), and cross-signing status/actions
- [x] 3.6 Implement `view_account_page(&self)` in `src/settings/user/view.rs` for password change and account deactivation
- [x] 3.7 Implement `view_packs_page(&self)` in `src/settings/user/view.rs` for subscribed sticker and emoji packs

## 4. Deep Linking & Integration

- [x] 4.1 Update verification prompt banners in `src/view/app.rs` and `src/view/chat.rs` to push `[SettingsPanel::User, SettingsPanel::UserSessions]` directly onto the stack
- [x] 4.2 Add `is_user_settings_open(&self)` helper to ensure verification overlays are suppressed across any user settings subpage

## 5. Verification & Tests

- [x] 5.1 Add unit tests for settings stack push, pop, clear, and Escape handling in `src/constellation/tests.rs`
- [x] 5.2 Add unit tests for user settings subpage message routing and status summary generation in `src/settings/user/tests.rs`
- [x] 5.3 Verify that all tests pass cleanly via `cargo test`
