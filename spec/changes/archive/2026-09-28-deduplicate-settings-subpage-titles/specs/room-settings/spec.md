# Spec Delta

## MODIFIED Requirements

### Requirement: Subpage Isolation and Functional Grouping
The room settings system SHALL isolate distinct settings categories into dedicated subpages to minimize visual clutter and group related actions, presenting configuration controls without redundant section titles that duplicate the context drawer title.

#### Scenario: Profile and identity management
- **WHEN** the user navigates to the Profile & Identity subpage
- **THEN** the system displays avatar preview/upload controls, room name editing, room topic editing, room ID, and canonical alias input without a redundant profile section title, alongside a dedicated alternative aliases section with add and remove controls.

#### Scenario: Room notification preferences
- **WHEN** the user navigates to the Notifications subpage
- **THEN** the system displays the room-specific notification mode choices (`All Messages`, `Mentions and Keywords Only`, `Mute`) in a settings section without a redundant section header duplicating the drawer title.

#### Scenario: Access rules and encryption security
- **WHEN** the user navigates to the Access & Security subpage
- **THEN** the system displays end-to-end encryption status and enablement, join rule selection (Public, Invite, Knock, Restricted by Space), history visibility selection, and restricted parent space configuration with descriptive section subtitles.

#### Scenario: Roles and permissions management
- **WHEN** the user navigates to the Roles & Permissions subpage
- **THEN** the system displays power level thresholds for events and actions including message sending, invite, kick, ban, redaction, and room metadata modification in a settings section without a redundant section header duplicating the drawer title.

#### Scenario: Member list and moderation management
- **WHEN** the user navigates to the Members & Moderation subpage
- **THEN** the system displays a filterable member list with role assignment controls, user invitation input, and moderation actions for kicking and banning users with reason inputs.

#### Scenario: Stickers and custom emoji packs
- **WHEN** the user navigates to the Stickers & Emojis subpage
- **THEN** the system displays room image packs with item previews, pack creation controls, image upload triggers, and pack/image deletion controls in a settings section without a redundant section header duplicating the drawer title.
