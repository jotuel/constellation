# Spec Delta

## Purpose

Provides a multi-page hierarchical settings interface for global application preferences, appearance toggles, notification options, diagnostic session errors, and cache maintenance with stack-based back navigation and live status summaries.

## ADDED Requirements

### Requirement: App Settings Navigation Stack History
The app settings interface SHALL maintain a navigation stack of active pages within the context drawer to provide hierarchical navigation history.

#### Scenario: Drilling down into an app subpage
- **WHEN** the user activates any category row from the App Settings overview
- **THEN** the system pushes the selected subpage onto the navigation stack, updates the context drawer title to the subpage name, displays the subpage content, and reveals a back navigation action in the context drawer header.

#### Scenario: Back navigation via header action
- **WHEN** the navigation stack has a depth greater than one and the user activates the back action in the context drawer header
- **THEN** the system pops the current app subpage from the navigation stack, restores the previous page and title, and hides the back action if the stack depth returns to one.

#### Scenario: Back navigation via Escape key
- **WHEN** the navigation stack has a depth greater than one and the user presses the Escape key
- **THEN** the system pops the current app subpage from the navigation stack and returns to the root app overview.

#### Scenario: Drawer dismissal clears stack
- **WHEN** the user closes the context drawer via the close action or top-level toggle shortcut
- **THEN** the system dismisses the context drawer and clears the navigation stack so subsequent openings start at the root overview.

### Requirement: App Settings Overview Index and Live Summaries
The root App Settings page SHALL display an overview index dividing application configuration into dedicated functional categories with live status summaries.

#### Scenario: Rendering overview categories with live summaries
- **WHEN** the user views the App Settings overview
- **THEN** the system displays category rows for Appearance & Display, Notifications & Diagnostics, and Maintenance & Shortcuts, each showing a descriptive title, a live status summary (e.g. active visual modes, notification state, or session error count), and a drill-down chevron indicator.

#### Scenario: Selecting an overview category
- **WHEN** the user activates anywhere within an overview category row
- **THEN** the system navigates to the corresponding subpage.

### Requirement: App Subpage Functional Isolation
The app settings system SHALL isolate distinct settings categories into dedicated subpages to minimize visual clutter and group related actions.

#### Scenario: Appearance and visual formatting management
- **WHEN** the user navigates to the Appearance & Display subpage
- **THEN** the system displays configuration toggles for compact mode, markdown rendering, sync indicator, video autoplay, and hiding threaded messages.

#### Scenario: Notifications and diagnostics management
- **WHEN** the user navigates to the Notifications & Diagnostics subpage
- **THEN** the system displays the typing notifications toggle and a scrollable diagnostic log of recorded session errors with individual dismiss controls and a clear-all action.

#### Scenario: Maintenance and shortcuts management
- **WHEN** the user navigates to the Maintenance subpage
- **THEN** the system displays cache clearing controls and an action to open the keyboard shortcuts configuration subpage.
