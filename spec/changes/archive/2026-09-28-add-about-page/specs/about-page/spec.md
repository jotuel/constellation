# Spec Delta

## Purpose

Displays application metadata, release version, licensing terms, author information, and external project links using native COSMIC widgets.

## ADDED Requirements

### Requirement: Application About Presentation
The system SHALL present an About interface displaying application identity, version, author, licensing, and external resource links.

#### Scenario: Viewing about details
- **WHEN** the user opens the About interface
- **THEN** the system displays the application icon, name, author name, current release version, license type, and external links for repository, issue tracker, and donation.

### Requirement: External Link Dispatch
The system SHALL support launching external project links from the About interface.

#### Scenario: Activating an external link
- **WHEN** the user activates any link row or license link within the About interface
- **THEN** the system dispatches the corresponding target URL to the host operating system's default browser or handler.

### Requirement: Header User Menu Access
The system SHALL provide access to the About interface from the primary user menu in the header bar.

#### Scenario: Opening about from header menu
- **WHEN** the user activates the About action from the header user menu
- **THEN** the system reveals the About page in the context drawer.
