# Design: User Settings Subpages and Navigation Stack

## Context

Constellation manages its contextual side panels through `cosmic::app::context_drawer::ContextDrawer` in `src/constellation/app.rs`. Currently, `Constellation` tracks active settings using a scalar `current_settings_panel: Option<SettingsPanel>`. When a panel transitions (such as Room Settings opening Permissions), the previous panel state is overwritten without history, preventing back-navigation.

In User Settings (`src/settings/user/`), all 12 sections are concatenated in a single `settings::view_column` in `view.rs`. Notification modes are presented using an ad-hoc row of buttons that wraps awkwardly across narrow drawer widths. See `proposal.md` for motivation and background.

## Goals / Non-Goals

**Goals:**
- Provide a persistent navigation history stack within `Constellation` to support hierarchical drill-down and back navigation across settings.
- Utilize the `ContextDrawer::actions` header slot for `< Back` navigation when drilled into subpages.
- Modularize `src/settings/user/view.rs` into an Overview index and 6 focused subpages.
- Implement clickable overview rows with status summaries and drill-down chevrons.
- Use `cosmic::widget::segmented_control::horizontal` hosted in `settings::flex_item` for responsive DM/Group notification selection.
- Support deep linking from external verification triggers into `SettingsPanel::UserSessions` while preserving parent history.

**Non-Goals:**
- Refactoring Room Settings or Space Settings widgets/subpages in this change (scoped for follow-up changes).
- Changing Matrix SDK API calls or notification sync logic.
- Altering the desktop layout outside the context drawer.

## Decisions

### 1. Centralized Navigation Stack (`Vec<SettingsPanel>`)

**Decision:** Replace `current_settings_panel: Option<SettingsPanel>` in `Constellation` with `settings_stack: Vec<SettingsPanel>`.

**Rationale:**
- Keeps navigation history pure and centralized in the Elm architecture.
- Any subpage or external trigger can push onto the stack without losing the parent context.
- Allows `context_drawer()` to query `self.settings_stack.last()` for the active view, and `self.settings_stack.len() > 1` to determine whether to render the `< Back` action.

**Alternatives considered:**
- *Domain-internal page state in `settings::user::State`*: Would require each domain to independently implement back-button logic and communicate with `ContextDrawer` to update drawer titles and header actions, creating cross-boundary boilerplate.

### 2. Flat `SettingsPanel` Variants

**Decision:** Add flat enum variants to `SettingsPanel`:
```rust
pub enum SettingsPanel {
    App,
    User,             // Root overview index
    UserProfile,      // Avatar, display name, 3PIDs
    UserNotifications,// DM/Group modes, keywords
    UserPrivacy,      // Media previews, invite avatars, ignored users
    UserSessions,     // Devices, verification UI, cross-signing
    UserAccount,      // Password change, account deactivation
    UserPacks,        // Sticker and emoji packs
    Room,
    Permissions,
    ManageRoomMembers,
    Space,
    ManageSpaceRooms,
    Members,
    Pinned,
    ActiveThreads,
    Shortcuts,
}
```

**Rationale:**
- Follows the existing pattern in Constellation where `Permissions` and `ManageRoomMembers` are already flat variants alongside `Room`.
- Simplifies title mapping in `context_drawer()`, shortcut dispatch, and test assertions.

### 3. Header Action Back-Button

**Decision:** Inject a back button via `ContextDrawer::actions` when `settings_stack.len() > 1`:
```rust
let actions = if self.settings_stack.len() > 1 {
    Some(
        cosmic::widget::button::icon(cosmic::widget::icon::from_name("go-previous-symbolic"))
            .on_press(Message::SettingsBack)
            .into()
    )
} else {
    None
};
```
And handle `Message::SettingsBack` in `update.rs`:
```rust
Message::SettingsBack => {
    if self.settings_stack.len() > 1 {
        self.settings_stack.pop();
    }
    Task::none()
}
```
In keyboard shortcut / Escape handling: if `settings_stack.len() > 1`, Escape pops the top panel instead of closing the entire drawer.

### 4. Overview Rows with Status Summaries

**Decision:** Render overview items as full-width clickable list items using `cosmic::theme::Button::ListItem`:
```rust
button::custom(
    Row::new()
        .align_y(Alignment::Center)
        .push(
            Column::new()
                .spacing(2)
                .push(text::body(title))
                .push(text::caption(summary))
                .width(Length::Fill)
        )
        .push(icon::from_name("go-next-symbolic").symbolic(true))
)
.class(theme::Button::ListItem(radii))
.on_press(Message::OpenPanel(panel))
```
**Summaries:**
- Profile: `display_name` + primary email if available
- Notifications: current DM & Group mode names + keyword count
- Privacy: media preview status + ignored user count
- Sessions: device count + verification status
- Account: password status
- Stickers: subscribed pack count

### 5. Responsive Segmented Notification Controls

**Decision:** Wrap `cosmic::widget::segmented_control::horizontal` in `settings::flex_item` for Direct Messages and Group Chats:
- Each mode maps to `RoomNotificationMode`: `AllMessages`, `MentionsAndKeywordsOnly`, `Mute`.
- On standard drawer widths, title sits left, segmented control sits right.
- On narrow drawer constraints, `flex_item` allows the segmented control to wrap beneath the label smoothly without clipping.

## Risks / Trade-offs

- **Verification Overlay Conflict**: `src/view/app.rs` currently suppresses the verification overlay with `self.current_settings_panel == Some(SettingsPanel::User)`.
  - *Mitigation*: Add helper `self.is_user_settings_open()` that checks if `self.settings_stack.last()` is `Some(SettingsPanel::User | SettingsPanel::UserProfile | ... | SettingsPanel::UserSessions)`.
- **Active Navigation During Sync**: If a Matrix event updates device list or profile while on a subpage, the state updates in place without resetting the navigation stack.
  - *Mitigation*: `settings::user::State` retains its fields across views; subpages render directly from the existing state.
- **Escape Key Interception**: Pressing Escape might be desired to close the drawer immediately by some users rather than navigating back.
  - *Mitigation*: Standard COSMIC HIG practice in hierarchical drawers and dialogs is Escape pops to parent first, then closes on root. The close `[X]` button remains available at all times to dismiss the entire drawer in one click.
