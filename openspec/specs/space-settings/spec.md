# Space Settings Specification

## Purpose
Provides a multi-page hierarchical settings interface for space configuration, profile identity, discovery access rules, and child room/subspace hierarchy management with stack-based back navigation and live status summaries.

## Requirements

### Requirement: Space Settings Hierarchical Navigation and Stack History
The space settings interface SHALL maintain a navigation stack of active pages within the context drawer to provide hierarchical navigation history.

#### Scenario: Drilling down into a space subpage
- **WHEN** the user activates any category row or the header card from the Space Settings overview
- **THEN** the system pushes the selected subpage onto the navigation stack, updates the context drawer title to the subpage name, displays the subpage content, and reveals a back navigation action in the context drawer header.

#### Scenario: Back navigation via header action
- **WHEN** the navigation stack has a depth greater than one and the user activates the back action in the context drawer header
- **THEN** the system pops the current space subpage from the navigation stack, restores the previous page and title, and hides the back action if the stack depth returns to one.

#### Scenario: Back navigation via Escape key
- **WHEN** the navigation stack has a depth greater than one and the user presses the Escape key
- **THEN** the system pops the current space subpage from the navigation stack and returns to the root space overview.

#### Scenario: Drawer dismissal clears stack
- **WHEN** the user closes the context drawer via the close action or top-level toggle shortcut
- **THEN** the system dismisses the context drawer and clears the navigation stack so subsequent openings start at the root overview.

### Requirement: Space Settings Overview Index and Live Summaries
The root Space Settings page SHALL display an overview index dividing space settings into dedicated functional categories with live status summaries.

#### Scenario: Rendering space header card
- **WHEN** the user opens the root Space Settings page
- **THEN** the system displays a header card showing the space avatar, space display name, and canonical alias or space ID, formatted with list-item styling and a drill-down chevron navigating to the Space Profile subpage.

#### Scenario: Rendering category rows with live summaries
- **WHEN** the user views the Space Settings overview
- **THEN** the system displays category rows for Space Profile, Discovery & Access, and Rooms & Hierarchy, each showing a descriptive title, a live status summary (e.g. current canonical alias, access mode, or child room count), and a drill-down chevron indicator.

#### Scenario: Selecting an overview category
- **WHEN** the user activates anywhere within an overview category row
- **THEN** the system navigates to the corresponding subpage.

### Requirement: Space Subpage Functional Isolation
The space settings system SHALL isolate distinct settings categories into dedicated subpages to minimize visual clutter and group related actions.

#### Scenario: Space profile and identity management
- **WHEN** the user navigates to the Space Profile subpage
- **THEN** the system displays avatar upload/change controls, space name editing, space topic editing, and canonical alias management with dirty-checking save controls.

#### Scenario: Space discovery and access rules
- **WHEN** the user navigates to the Discovery & Access subpage
- **THEN** the system displays toggles for public discoverability and invite-only access with dirty-checking save controls.

#### Scenario: Space rooms and hierarchy management
- **WHEN** the user navigates to the Rooms & Hierarchy subpage
- **THEN** the system displays room filtering, child rooms and subspaces with order inputs, suggested toggles, join rule modification, and child addition by ID.

### Requirement: Space Settings Feedback and Error Handling
The space settings interface SHALL provide non-intrusive feedback banners and error dismissal controls across all space settings subpages.

#### Scenario: Displaying and dismissing space errors
- **WHEN** an operation fails and an error message is set in space settings state
- **THEN** the system displays a dismissable error banner in the active settings view that clears the error upon user dismissal.
