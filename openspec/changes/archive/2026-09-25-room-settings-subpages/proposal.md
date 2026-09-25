# Proposal: Room Settings Subpages and Responsive Controls

## Why

The current Room Settings panel renders all room configuration sections (profile, encryption security, aliases, notification modes, image packs, and navigation to permissions and member moderation) in a single continuous scroll inside the narrow right-hand context drawer (800+ lines in `src/settings/room/view.rs`). This creates visual clutter, buries critical settings like join rules and power levels, uses ad-hoc radio buttons for notification modes instead of responsive segmented controls, and lacks the clean hierarchical overview index that was recently introduced for User Settings. Decomposing Room Settings into an Overview index and focused subpages with stack-based back navigation, live status summaries, and responsive controls significantly improves clarity, discoverability, and navigation flow.

## What Changes

- **Room Settings Overview Index**: Replace the monolithic single-scroll room settings view with a clean overview index:
  - **Room Header Card**: A prominent card at the top displaying the room avatar, room display name, and canonical alias or room ID with clickable row styling (`cosmic::theme::Button::ListItem`) and a `go-next-symbolic` chevron navigating directly to `RoomProfile`.
  - **Category Rows with Live Summaries**: Distinct category rows with descriptive titles, live status summaries, and drill-down chevrons navigating to:
    - `RoomProfile`: Avatar upload/change, room name, topic, room ID, canonical alias, and alternative aliases list/management.
    - `RoomNotifications`: Room-specific notification mode (`All Messages`, `Mentions and Keywords Only`, `Mute`).
    - `RoomSecurity`: End-to-end encryption status and enablement, join rule selection (Public, Invite, Knock, Restricted by Space), history visibility, and restricted space configuration.
    - `Permissions`: Role event levels and power level thresholds for room moderation.
    - `ManageRoomMembers`: Member list with role promotion/demotion, user invitation, and kick/ban actions.
    - `RoomPacks`: Room sticker and custom emoji packs, pack creation, and image upload/deletion.
  - **Room Lifecycle Actions**: Destructive actions (Leave Room, Forget Room) cleanly separated at the bottom of the overview.
- **Centralized Settings Navigation Stack Integration**:
  - Add `RoomProfile`, `RoomNotifications`, `RoomSecurity`, and `RoomPacks` variants to `SettingsPanel`.
  - Update `handle_open_settings` in `src/constellation/handlers/rooms.rs` so that drilling into any room subpage pushes `[SettingsPanel::Room, subpage]` onto `settings_stack`, enabling automatic `< Back` navigation in `ContextDrawer::actions` and via the Escape key.
  - Wire context drawer titles and subpage view dispatch in `src/constellation/app.rs`.
- **Responsive Flex Segmented Notification Controls**:
  - Replace the radio button group for room notification modes with `cosmic::widget::segmented_control::horizontal` wrapped inside `settings::flex_item` using `SingleSelectModel`, matching the responsive layout used in User Notifications.
- **Subpage View Modularization and Feedback**:
  - Decompose `src/settings/room/view.rs` into dedicated subpage functions (`view_overview`, `view_profile_page`, `view_notifications_page`, `view_security_page`, `view_packs_page`).
  - Standardize error banners (`DismissError`) and conditional save buttons (`Message::SaveRoom`) across subpages with unsaved modifications.

## Capabilities

### New Capabilities
- `room-settings`: Multi-page hierarchical navigation, overview status summaries, responsive segmented notification controls, and modular subpages for room profile, notifications, security, permissions, members, and image packs.

### Modified Capabilities
None (brownfield project; no existing specs under `openspec/specs/room-settings/`).

## Impact

- **UI / UX**: Context drawer displays a clean, compact overview index instead of a monolithic 800+ line scroll; back-navigation is consistent via `ContextDrawer::actions`.
- **Code Architecture**: Modularizes `src/settings/room/view.rs` into distinct subpage functions; updates `src/constellation/app.rs`, `src/constellation/mod.rs`, and `src/constellation/handlers/rooms.rs` to operate on `settings_stack`.
- **Dependencies**: Uses existing libcosmic widgets (`segmented_control`, `flex_item`, `ContextDrawer::actions`). No new external dependencies required.
- **Localization**: Adds Fluent string IDs for new room subpage titles and overview status summaries in `res/i18n/en/constellation.ftl`.
