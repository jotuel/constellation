# Spec Delta

## MODIFIED Requirements

### Requirement: Overview Category Navigation Row
The settings widgets system SHALL provide an overview category navigation row widget that renders a descriptive title, a live status summary, and a drill-down chevron indicator using high-contrast list item styling that ensures readable text contrast and active interactive feedback in both light and dark system themes.

#### Scenario: Category row rendering and interaction
- **WHEN** the category row is rendered with a title, summary, and action message
- **THEN** the component displays the title, summary, and trailing chevron in a full-width clickable list item that inherits contextual text contrast and dispatches the configured message on activation.

#### Scenario: Visual contrast across theme modes
- **WHEN** the category row is rendered in either light or dark theme mode
- **THEN** title, summary, and chevron elements render with legible text contrast against the drawer surface without appearing greyed out or disabled.

### Requirement: Overview Header Card
The settings widgets system SHALL provide an interactive overview header card widget displaying an avatar, primary title, subtitle or identifier, and a navigation chevron with high-contrast text and interactive feedback across light and dark system themes.

#### Scenario: Header card navigation
- **WHEN** the user activates the overview header card
- **THEN** the component triggers the configured navigation action to open the corresponding profile configuration page.

#### Scenario: Header card contrast across theme modes
- **WHEN** the overview header card is rendered in either light or dark theme mode
- **THEN** primary title, subtitle, and navigation chevron render with legible text contrast against the drawer surface without appearing greyed out or disabled.
