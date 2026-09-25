# Design: Room Settings Subpages and Responsive Controls

## Context

Constellation manages its contextual side drawer using `cosmic::app::context_drawer::ContextDrawer` in `src/constellation/app.rs`. Following the recent User Settings modularization, `Constellation` tracks active settings using a centralized `settings_stack: Vec<SettingsPanel>` and automatically injects a `< Back` button (`go-previous-symbolic`) into `ContextDrawer::actions` whenever `settings_stack.len() > 1`.

Room Settings (`src/settings/room/`), however, currently renders profile, encryption security, aliases, notification modes, image packs, and navigation buttons in a single monolithic 818-line scroll (`view.rs`). While `SettingsPanel::Permissions` and `SettingsPanel::ManageRoomMembers` exist as standalone panels, they are accessed via simple "Open" buttons at the bottom of the long scroll, and other major functional areas (notifications, security, aliases, image packs) have no subpage isolation. Furthermore, notification modes are rendered as ad-hoc radio buttons rather than the responsive horizontal segmented controls used in User Settings.

See `proposal.md` for motivation and background.

## Goals / Non-Goals

**Goals:**
- Modularize `src/settings/room/view.rs` into an Overview index and focused subpage views (`RoomProfile`, `RoomNotifications`, `RoomSecurity`, `Permissions`, `ManageRoomMembers`, `RoomPacks`).
- Implement an Overview index page with a room header card and clickable category rows showing live status summaries and drill-down chevrons.
- Add `RoomProfile`, `RoomNotifications`, `RoomSecurity`, and `RoomPacks` variants to `SettingsPanel`.
- Integrate room subpages into `settings_stack` within `handle_open_settings` in `src/constellation/handlers/rooms.rs`, ensuring `< Back` and Escape key navigation work consistently.
- Implement responsive segmented controls for room notification mode selection using `cosmic::widget::segmented_control::horizontal` wrapped inside `settings::flex_item`.
- Standardize feedback (error banners and conditional save buttons) across subpages with editable state.

**Non-Goals:**
- Modifying underlying Matrix SDK API calls, matrix state caches, or network request flows.
- Refactoring Space Settings in this change.
- Changing the desktop UI layout outside the context drawer.

## Decisions

### 1. Centralized Navigation Stack Integration

**Decision:** Add flat enum variants to `SettingsPanel` for room subpages:
```rust
pub enum SettingsPanel {
    App,
    User,
    UserProfile,
    UserNotifications,
    UserPrivacy,
    UserSessions,
    UserAccount,
    UserPacks,
    Room,               // Root Room Overview index
    RoomProfile,        // Name, topic, avatar, ID, canonical/alt aliases
    RoomNotifications,  // Room notification mode segmented control
    RoomSecurity,       // E2E encryption, join rules, history visibility, restricted space
    RoomPacks,          // Room sticker and custom emoji image packs
    Permissions,        // Power level thresholds for events and actions
    ManageRoomMembers,  // Filterable member list and moderation actions
    Space,
    ManageSpaceRooms,
    Members,
    Pinned,
    ActiveThreads,
    Shortcuts,
}
```

**Rationale:**
- Keeps the navigation model pure and centralized in `Constellation`.
- Allows `ContextDrawer` in `src/constellation/app.rs` to derive drawer titles directly via `match panel` and route rendering to the appropriate `room_settings` subpage function.
- In `src/constellation/handlers/rooms.rs:handle_open_settings`, all room subpages (`RoomProfile`, `RoomNotifications`, `RoomSecurity`, `RoomPacks`, `Permissions`, `ManageRoomMembers`) follow the exact stack-pushing semantics:
  - If `settings_stack.first() == Some(&SettingsPanel::Room)`, pop any existing subpage if `settings_stack.len() > 1`, then push the target subpage.
  - If entering from outside, initialize `settings_stack` to `vec![SettingsPanel::Room, panel]`.
  - If entering `SettingsPanel::Room`, truncate the stack to `vec![SettingsPanel::Room]`.

**Alternatives considered:**
- *Nesting a sub-page enum inside `SettingsPanel::Room(RoomSubpage)`*: Would require changing all match statements across `mod.rs`, `app.rs`, `handlers/`, and test suites, whereas flat variants preserve existing patterns established by `Permissions` and `ManageRoomMembers`.

### 2. Room Overview Index Design

**Decision:** The root Room Settings view (`view_overview`) renders:
1. **Room Header Card**: A clickable `Button::ListItem` card at the top displaying:
   - The room avatar image (or 64x64 container placeholder if none).
   - Room title (`name` or "Unnamed Room").
   - Room subtitle (canonical alias if set, otherwise room ID).
   - Trailing `go-next-symbolic` chevron navigating to `SettingsPanel::RoomProfile`.
2. **Category Rows**: Full-width clickable list items (`Button::ListItem`) with titles, live status summaries, and `go-next-symbolic` chevrons:
   - `RoomProfile`: "Room Profile & Identity" -> Summary: Canonical alias or alternative alias count.
   - `RoomNotifications`: "Notifications" -> Summary: Current notification mode (`All Messages`, `Mentions and Keywords Only`, or `Muted`).
   - `RoomSecurity`: "Access & Security" -> Summary: E2E status ("Encrypted" / "Unencrypted") and Join Rule ("Public", "Invite only", "Restricted", "Knock").
   - `Permissions`: "Roles & Permissions" -> Summary: Default power level and moderation threshold.
   - `ManageRoomMembers`: "Members & Moderation" -> Summary: Member count.
   - `RoomPacks`: "Stickers & Emojis" -> Summary: Pack count.
3. **Danger Zone**: Destructive room lifecycle actions at the bottom of the overview:
   - "Leave Room" (`Message::LeaveRoom`) with confirmation/destructive styling.
   - "Forget Room" (`Message::ForgetRoom`) when membership is `RoomState::Left`.

### 3. Responsive Segmented Notification Controls

**Decision:** Replace the radio button group for room notifications with `cosmic::widget::segmented_control::horizontal` wrapped inside `settings::flex_item`:
- `src/settings/room/state.rs` manages a `cosmic::widget::segmented_button::SingleSelectModel` with three entities corresponding to `RoomNotificationMode::AllMessages`, `MentionsAndKeywordsOnly`, and `Mute`.
- When `Message::NotificationModeChanged` is emitted, the state model selection is updated and a task is spawned to submit the change to the homeserver.
- If the context drawer is narrow, `settings::flex_item` wraps the control underneath the label smoothly without clipping or horizontal scrollbars.

### 4. Subpage View Modularization & Feedback

**Decision:** Decompose `src/settings/room/view.rs` into specialized page renderers:
- `view_overview(&self) -> Element<'_, Message>`
- `view_profile_page(&self) -> Element<'_, Message>` (combines profile avatar/name/topic and alias sections)
- `view_notifications_page(&self) -> Element<'_, Message>` (notifications segmented control)
- `view_security_page(&self) -> Element<'_, Message>` (encryption status, join rule radio/inputs, history visibility, restricted space)
- `view_permissions_page(&self) -> Element<'_, Message>` (power level threshold rows)
- `view_manage(&self) -> Element<'_, Message>` (member list, filter, invite input, kick/ban actions)
- `view_packs_page(&self) -> Element<'_, Message>` (image pack manager, creation, upload)

Each subpage that supports dirty edits (`RoomProfile`, `Permissions`) renders its own contextual `view_save_button()` and `view_error()` banner at the bottom of its column.

## Risks / Trade-offs

- **Unsaved State Across Subpage Navigation**: Users might edit fields on `RoomProfile` or `Permissions` and navigate back to the overview before clicking "Save Changes".
  - *Mitigation*: All input state is held in `settings::room::State` and is not lost when navigating between subpages. An unsaved changes indicator or keeping the save button visible when navigating back preserves user modifications.
- **Room Switching While Drilled In**: Switching the active timeline room while a room subpage is open could display stale room settings for the previous room.
  - *Mitigation*: In `handle_select_room`, if room settings are open, reload the new room data and reset `settings_stack` to `vec![SettingsPanel::Room]`, ensuring the user starts fresh on the overview for the new room.
- **Segmented Control SingleSelectModel State**: `SingleSelectModel` requires entity IDs to be tracked.
  - *Mitigation*: Follow the proven pattern in `src/settings/user/state.rs`: initialize entities in `State::default()` and select the matching entity whenever `notification_mode` is loaded or updated.
