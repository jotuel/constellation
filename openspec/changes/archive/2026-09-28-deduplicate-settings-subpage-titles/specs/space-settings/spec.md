# Spec Delta

## MODIFIED Requirements

### Requirement: Space Subpage Functional Isolation
The space settings system SHALL isolate distinct settings categories into dedicated subpages to minimize visual clutter and group related actions, presenting configuration controls without redundant section titles that duplicate the context drawer title.

#### Scenario: Space profile and identity management
- **WHEN** the user navigates to the Space Profile subpage
- **THEN** the system displays avatar upload/change controls, space name editing, space topic editing, and canonical alias management in a settings section without a redundant section header duplicating the drawer title.

#### Scenario: Space discovery and access rules
- **WHEN** the user navigates to the Discovery & Access subpage
- **THEN** the system displays toggles for public discoverability and invite-only access in a settings section without a redundant section header duplicating the drawer title.

#### Scenario: Space rooms and hierarchy management
- **WHEN** the user navigates to the Rooms & Hierarchy subpage
- **THEN** the system displays room filtering, child rooms and subspaces with order inputs, suggested toggles, join rule modification, and child addition by ID under descriptive section headings.
