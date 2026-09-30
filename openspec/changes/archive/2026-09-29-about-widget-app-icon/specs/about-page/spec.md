# Spec Delta

## MODIFIED Requirements

### Requirement: Application About Presentation
The system SHALL present an About interface displaying application identity, version, author, licensing, and external resource links, ensuring the application icon is reliably displayed from embedded application assets regardless of host system icon theme installation.

#### Scenario: Viewing about details
- **WHEN** the user opens the About interface
- **THEN** the system displays the application icon loaded from embedded assets, name, author name, current release version, license type, and external links for repository, issue tracker, and donation.
