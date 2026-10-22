# Spec Delta

## MODIFIED Requirements

### Requirement: Subpage Isolation and Functional Grouping
The user settings system SHALL isolate distinct settings categories into dedicated subpages to minimize visual clutter and group related actions, presenting configuration controls without redundant section titles that duplicate the context drawer title.

#### Scenario: Profile and identity management
- **WHEN** the user navigates to the Profile & Identity subpage
- **THEN** the system displays avatar preview/upload controls and display name editing without a redundant profile section title, alongside connected third-party identifiers (emails and phone numbers) under a descriptive section title.

#### Scenario: Notification defaults management
- **WHEN** the user navigates to the Notifications subpage
- **THEN** the system displays direct message notification defaults, group chat notification defaults, and the keyword notification list with add/remove controls under descriptive section headings.

#### Scenario: Privacy and blocklist management
- **WHEN** the user navigates to the Privacy subpage
- **THEN** the system displays toggles for media preview policy and invite avatar policy, alongside the list of ignored users with unignore controls under descriptive section headings.

#### Scenario: Sessions and cryptographic key management
- **WHEN** the user navigates to the Sessions & Encryption subpage
- **THEN** the system displays the current device details, other active sessions, device verification actions, interactive SAS/QR verification flows, and cross-signing status and bootstrapping under descriptive section headings.

#### Scenario: Account credentials and lifecycle
- **WHEN** the user navigates to the Account & Security subpage
- **THEN** the system displays the password change form and the account deactivation action under descriptive section headings.

#### Scenario: Stickers and emojis management
- **WHEN** the user navigates to the Stickers & Emojis subpage
- **THEN** the system displays the subscribed sticker and custom emoji packs in a settings section without a redundant section header duplicating the drawer title.
