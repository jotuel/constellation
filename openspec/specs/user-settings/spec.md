# User Settings Specification

## Purpose
Provides a multi-page hierarchical settings interface for user preferences, account identity, notification defaults, privacy policies, active cryptographic sessions, and emoji/sticker packs with stack-based back navigation and responsive controls.

## Requirements

### Requirement: Multi-Page Navigation and Stack History
The settings interface SHALL maintain a navigation stack of active pages within the context drawer to provide hierarchical navigation history.

#### Scenario: Drilling down into a subpage
- **WHEN** the user selects any category row from the User Settings overview
- **THEN** the system pushes the selected subpage onto the navigation stack, updates the context drawer title to the subpage name, displays the subpage content, and reveals a back navigation action in the context drawer header

#### Scenario: Back navigation via header action
- **WHEN** the navigation stack has a depth greater than one and the user activates the back action in the context drawer header
- **THEN** the system pops the current subpage from the navigation stack, restores the previous page and title, and hides the back action if the stack depth returns to one

#### Scenario: Back navigation via Escape key
- **WHEN** the navigation stack has a depth greater than one and the user presses the Escape key
- **THEN** the system pops the current subpage from the navigation stack and returns to the parent page without closing the entire drawer

#### Scenario: Drawer dismissal clears stack
- **WHEN** the user closes the context drawer via the close action or top-level toggle shortcut
- **THEN** the system dismisses the context drawer and clears the navigation stack so subsequent openings start at the root overview

### Requirement: User Settings Overview Index and Live Summaries
The root User Settings page SHALL display an overview index dividing settings into dedicated functional categories with live status summaries.

#### Scenario: Rendering overview categories with live summaries
- **WHEN** the user opens the root User Settings page
- **THEN** the system displays a profile card followed by six category rows (Profile & Identity, Notifications, Privacy, Sessions & Encryption, Account & Security, Stickers & Emojis), each showing a title, a live status subtitle describing current settings (e.g. current notification modes, active session count, or connected identity), and a drill-down chevron indicator

#### Scenario: Selecting an overview category
- **WHEN** the user clicks or activates anywhere within an overview category row
- **THEN** the system navigates to the corresponding subpage

### Requirement: Subpage Isolation and Functional Grouping
The user settings system SHALL isolate distinct settings categories into dedicated subpages to minimize visual clutter and group related actions.

#### Scenario: Profile and identity management
- **WHEN** the user navigates to the Profile & Identity subpage
- **THEN** the system displays avatar preview/upload controls, display name editing, and connected third-party identifiers (emails and phone numbers)

#### Scenario: Notification defaults management
- **WHEN** the user navigates to the Notifications subpage
- **THEN** the system displays direct message notification defaults, group chat notification defaults, and the keyword notification list with add/remove controls

#### Scenario: Privacy and blocklist management
- **WHEN** the user navigates to the Privacy subpage
- **THEN** the system displays toggles for media preview policy and invite avatar policy, alongside the list of ignored users with unignore controls

#### Scenario: Sessions and cryptographic key management
- **WHEN** the user navigates to the Sessions & Encryption subpage
- **THEN** the system displays the current device details, other active sessions, device verification actions, interactive SAS/QR verification flows, and cross-signing status and bootstrapping

#### Scenario: Account credentials and lifecycle
- **WHEN** the user navigates to the Account & Security subpage
- **THEN** the system displays the password change form and the account deactivation action

#### Scenario: Stickers and emojis management
- **WHEN** the user navigates to the Stickers & Emojis subpage
- **THEN** the system displays the subscribed sticker and custom emoji packs with unsubscribe actions

### Requirement: Responsive Segmented Notification Controls
The notification preferences SHALL present three-way mode choices (`All Messages`, `Mentions and Keywords Only`, `Mute`) for Direct Messages and Group Chats using segmented controls hosted in a flexible layout.

#### Scenario: Mode selection updates global settings
- **WHEN** the user selects a notification mode segment for Direct Messages or Group Chats
- **THEN** the system updates the corresponding global notification policy and reflects the new mode in the control

#### Scenario: Responsive wrapping on narrow containers
- **WHEN** the context drawer width is constrained
- **THEN** the segmented control wraps beneath the label without clipping content or introducing horizontal scrollbars

### Requirement: Direct Deep Linking to Security and Verification
The system SHALL support deep linking directly to the Sessions & Encryption subpage from external verification notifications while preserving parent navigation history.

#### Scenario: Deep link from verification prompt
- **WHEN** the user activates a "Verify this device" prompt or incoming verification banner outside of settings
- **THEN** the system opens the context drawer directly to the Sessions & Encryption subpage with the navigation stack initialized to `[User, UserSessions]`, enabling the user to immediately perform verification and navigate back to User Settings overview via the back action
