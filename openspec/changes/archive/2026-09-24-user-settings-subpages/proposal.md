# Proposal: User Settings Subpages and Responsive Controls

## Why

The current User Settings panel renders 12 dense sections (750+ lines of view logic) in a single continuous scroll inside the narrow right-hand context drawer. This creates visual clutter, buries critical security/session verification tasks beneath routine preferences, lacks back-navigation when drilling into subpages, and uses ad-hoc button rows for notification mode selection that wrap unpredictably. Decomposing User Settings into a multi-page hierarchy with status summaries and responsive controls significantly improves clarity, discoverability, and navigation flow.

## What Changes

- **Centralized Settings Navigation Stack**: Replace `current_settings_panel: Option<SettingsPanel>` with `settings_stack: Vec<SettingsPanel>` in `Constellation` to maintain navigation history.
- **Top-Left Drawer Actions**: Dynamically inject a `< Back` button (`go-previous-symbolic`) into `ContextDrawer::actions` whenever `settings_stack.len() > 1`, popping the stack on press (and on Escape).
- **Subpage Decomposition**: Decompose the single User Settings view into an Overview index and 6 focused subpages:
  - `UserProfile`: Avatar upload/removal, display name, and 3PIDs (emails/phone numbers).
  - `UserNotifications`: Global DM and Group chat notification defaults, plus keyword alerts.
  - `UserPrivacy`: Media preview policies, invite avatar policies, and ignored users list.
  - `UserSessions`: Active devices, interactive SAS/QR verification UI, and cross-signing status/bootstrap.
  - `UserAccount`: Password change and account deactivation.
  - `UserPacks`: Subscribed sticker and emoji packs.
- **Overview Status Summaries**: Display informative subtitles on the root overview rows (e.g., current display name, DM/Group notification modes, active session count, cross-signing status) with clickable row styling (`cosmic::theme::Button::ListItem`) and `go-next-symbolic` chevrons.
- **Responsive Flex Segmented Controls**: Replace the ad-hoc wrapped button rows for Direct Message and Group Chat notification modes with `cosmic::widget::segmented_control::horizontal` wrapped inside `settings::flex_item` for clean alignment and responsive wrapping.
- **Direct Deep Linking**: Allow banners (such as the app header's "Verify this device" prompt) to push `[SettingsPanel::User, SettingsPanel::UserSessions]` directly onto the stack so users land directly on verification while retaining back navigation.

## Capabilities

### New Capabilities
- `user-settings`: Multi-page navigation, overview status summaries, responsive flex segmented notification controls, and session verification integration for user settings.

### Modified Capabilities
None (brownfield project; no existing specs in `openspec/specs/`).

## Impact

- **UI / UX**: Context drawer displays a clean, compact overview index instead of a monolithic 12-section scroll; back-navigation is consistent via `ContextDrawer::actions`.
- **Code Architecture**: Modularizes `src/settings/user/view.rs` into distinct subpage functions; updates `src/constellation/app.rs` and handlers to operate on `settings_stack`.
- **Dependencies**: Uses existing libcosmic widgets (`segmented_control`, `flex_item`, `ContextDrawer::actions`). No new external dependencies required.
- **Localization**: Adds Fluent string IDs for new subpage titles in `res/i18n/en/constellation.ftl`.
