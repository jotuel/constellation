# Spec Delta

## Purpose

Provides a multi-page hierarchical settings interface for room configuration, profile identity, notification preferences, access security, role permissions, member moderation, and custom image packs with stack-based back navigation and responsive controls.

## ADDED Requirements

### Requirement: Multi-Page Navigation and Stack History
The room settings interface SHALL maintain a navigation stack of active pages within the context drawer to provide hierarchical navigation history.

#### Scenario: Drilling down into a room subpage
- **WHEN** the user selects any category row or the profile card from the Room Settings overview
- **THEN** the system pushes the selected subpage onto the navigation stack, updates the context drawer title to the subpage name, displays the subpage content, and reveals a back navigation action in the context drawer header

#### Scenario: Back navigation via header action
- **WHEN** the navigation stack has a depth greater than one and the user activates the back action in the context drawer header
- **THEN** the system pops the current room subpage from the navigation stack, restores the previous page and title, and hides the back action if the stack depth returns to one

#### Scenario: Back navigation via Escape key
- **WHEN** the navigation stack has a depth greater than one and the user presses the Escape key
- **THEN** the system pops the current room subpage from the navigation stack and returns to the parent page without closing the entire drawer

#### Scenario: Drawer dismissal clears stack
- **WHEN** the user closes the context drawer via the close action or top-level toggle shortcut
- **THEN** the system dismisses the context drawer and clears the navigation stack so subsequent openings start at the root overview

### Requirement: Room Settings Overview Index and Live Summaries
The root Room Settings page SHALL display an overview index dividing room settings into dedicated functional categories with live status summaries.

#### Scenario: Rendering room header card with live room information
- **WHEN** the user opens the root Room Settings page
- **THEN** the system displays a prominent room card showing the room avatar, room name, and canonical alias or room ID, formatted with list-item styling and a drill-down chevron navigating to the Room Profile subpage

#### Scenario: Rendering overview categories with live summaries
- **WHEN** the user views the Room Settings overview
- **THEN** the system displays category rows for Profile & Identity, Notifications, Access & Security, Roles & Permissions, Members & Moderation, and Stickers & Emojis, each showing a descriptive title, a live status subtitle (e.g. current notification mode, encryption state, join rule, member count, or pack count), and a drill-down chevron indicator

#### Scenario: Selecting an overview category
- **WHEN** the user clicks or activates anywhere within an overview category row
- **THEN** the system navigates to the corresponding subpage

### Requirement: Subpage Isolation and Functional Grouping
The room settings system SHALL isolate distinct settings categories into dedicated subpages to minimize visual clutter and group related actions.

#### Scenario: Profile and identity management
- **WHEN** the user navigates to the Profile & Identity subpage
- **THEN** the system displays avatar preview/upload controls, room name editing, room topic editing, room ID, canonical alias input, and alternative aliases management with add and remove controls

#### Scenario: Room notification preferences
- **WHEN** the user navigates to the Notifications subpage
- **THEN** the system displays the room-specific notification mode choices (`All Messages`, `Mentions and Keywords Only`, `Mute`)

#### Scenario: Access rules and encryption security
- **WHEN** the user navigates to the Access & Security subpage
- **THEN** the system displays end-to-end encryption status and enablement, join rule selection (Public, Invite, Knock, Restricted by Space), history visibility selection, and restricted parent space configuration

#### Scenario: Roles and permissions management
- **WHEN** the user navigates to the Roles & Permissions subpage
- **THEN** the system displays power level thresholds for events and actions including message sending, invite, kick, ban, redaction, and room metadata modification

#### Scenario: Member list and moderation management
- **WHEN** the user navigates to the Members & Moderation subpage
- **THEN** the system displays a filterable member list with role assignment controls, user invitation input, and moderation actions for kicking and banning users with reason inputs

#### Scenario: Stickers and custom emoji packs
- **WHEN** the user navigates to the Stickers & Emojis subpage
- **THEN** the system displays room image packs with item previews, pack creation controls, image upload triggers, and pack/image deletion controls

### Requirement: Responsive Segmented Room Notification Controls
The room notification preferences SHALL present three-way mode choices (`All Messages`, `Mentions and Keywords Only`, `Mute`) using segmented controls hosted in a flexible layout.

#### Scenario: Mode selection updates room notification setting
- **WHEN** the user selects a notification mode segment on the room notifications subpage
- **THEN** the system updates the room notification mode on the Matrix homeserver and reflects the new mode in the control and overview summary

#### Scenario: Responsive wrapping on narrow containers
- **WHEN** the context drawer width is constrained
- **THEN** the segmented control wraps beneath the label without clipping content or introducing horizontal scrollbars

### Requirement: Room Lifecycle and Destructive Actions
The room settings interface SHALL provide isolated actions for leaving and forgetting rooms with destructive visual styling.

#### Scenario: Leaving room from settings
- **WHEN** the user activates the leave room action from the room settings overview
- **THEN** the system submits the room leave request and updates the room membership state

#### Scenario: Forgetting room from settings
- **WHEN** the room membership state is Left and the user activates the forget room action
- **THEN** the system removes the room from the client store and dismisses the context drawer
