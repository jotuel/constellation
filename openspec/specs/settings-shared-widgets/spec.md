# Settings Shared Widgets Specification

## Purpose
Provides reusable, accessible, and standardized UI components and state selectors for COSMIC settings panels, including overview navigation items, feedback actions, avatar containers, and notification mode segmented controls.

## Requirements

### Requirement: Overview Category Navigation Row
The settings widgets system SHALL provide an overview category navigation row widget that renders a descriptive title, a live status summary, and a drill-down chevron indicator using standard list-item styling and theme corner radii.

#### Scenario: Category row rendering and interaction
- **WHEN** the category row is rendered with a title, summary, and action message
- **THEN** the component displays the title, summary, and trailing chevron in a full-width clickable list item that dispatches the configured message on activation.

### Requirement: Overview Header Card
The settings widgets system SHALL provide an interactive overview header card widget displaying an avatar, primary title, subtitle or identifier, and a navigation chevron.

#### Scenario: Header card navigation
- **WHEN** the user activates the overview header card
- **THEN** the component triggers the configured navigation action to open the corresponding profile configuration page.

### Requirement: Uniform Avatar Container
The settings widgets system SHALL provide a uniform avatar presentation widget that renders an image handle when available or a centered fallback label inside a standard 64x64 container.

#### Scenario: Avatar handle present
- **WHEN** an avatar image handle is provided
- **THEN** the component renders the image sized to 64x64 pixels.

#### Scenario: Avatar handle absent
- **WHEN** no avatar image handle is available
- **THEN** the component renders a 64x64 centered fallback text container.

### Requirement: Feedback and Error Display
The settings widgets system SHALL provide a dismissable error banner section for displaying failure alerts within settings layouts.

#### Scenario: Error banner rendering and dismissal
- **WHEN** an error message string is present
- **THEN** the component renders a settings section containing the error text and an actionable dismiss button that triggers the dismiss action on click.

#### Scenario: Error banner absent when clean
- **WHEN** no error message string is present
- **THEN** the component returns no view element.

### Requirement: Standardized Save Action
The settings widgets system SHALL provide a standardized save button that reflects in-progress saving state, triggers save actions when modifications exist, and displays a guidance tooltip when inactive.

#### Scenario: Save button active when dirty
- **WHEN** changes exist and saving is not in progress
- **THEN** the save button is actionable and emits the save message on activation.

#### Scenario: Save button disabled when clean
- **WHEN** no modifications exist and saving is not in progress
- **THEN** the save button does not emit save actions and presents a tooltip indicating that changes are required to save.

#### Scenario: Save button during active save
- **WHEN** saving is in progress
- **THEN** the save button displays in-progress status text and does not accept further save actions.

### Requirement: Encapsulated Notification Mode Segmented Selector
The settings widgets system SHALL provide an encapsulated 3-way notification mode selector (`All Messages`, `Mentions and Keywords Only`, `Mute`) that manages segmented button model state and supports value cloning.

#### Scenario: Mode selection update
- **WHEN** a notification mode option is selected in the segmented control
- **THEN** the selector updates the active model state and emits the corresponding notification mode event.

#### Scenario: Selector cloning
- **WHEN** the selector instance is cloned
- **THEN** the cloned instance preserves the active mode selection without referencing stale or invalidated widget entities.
