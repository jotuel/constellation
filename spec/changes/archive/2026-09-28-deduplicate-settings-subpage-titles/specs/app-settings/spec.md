# Spec Delta

## MODIFIED Requirements

### Requirement: App Subpage Functional Isolation
The app settings system SHALL isolate distinct settings categories into dedicated subpages to minimize visual clutter and group related actions, presenting settings groups without redundant section titles that duplicate the context drawer title.

#### Scenario: Appearance and visual formatting management
- **WHEN** the user navigates to the Appearance & Display subpage
- **THEN** the system displays configuration toggles for compact mode, markdown rendering, sync indicator, video autoplay, and hiding threaded messages in a settings section without a redundant section header duplicating the drawer title.

#### Scenario: Notifications and diagnostics management
- **WHEN** the user navigates to the Notifications & Diagnostics subpage
- **THEN** the system displays the typing notifications toggle and a scrollable diagnostic log of recorded session errors with individual dismiss controls and a clear-all action in a settings section without a redundant section header duplicating the drawer title.

#### Scenario: Maintenance and shortcuts management
- **WHEN** the user navigates to the Maintenance subpage
- **THEN** the system displays cache clearing controls and an action to open the keyboard shortcuts configuration subpage in a settings section without a redundant section header duplicating the drawer title.
