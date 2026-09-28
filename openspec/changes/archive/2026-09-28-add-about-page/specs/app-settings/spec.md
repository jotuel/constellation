# Spec Delta

## MODIFIED Requirements

### Requirement: App Settings Overview Index and Live Summaries
The root App Settings page SHALL display an overview index dividing application configuration into dedicated functional categories with live status summaries and application metadata access.

#### Scenario: Rendering overview categories with live summaries
- **WHEN** the user views the App Settings overview
- **THEN** the system displays category rows for Appearance & Display, Notifications & Diagnostics, Maintenance & Shortcuts, and About, each showing a descriptive title, a status summary or version indicator, and a drill-down chevron indicator.

#### Scenario: Selecting an overview category
- **WHEN** the user activates anywhere within an overview category row
- **THEN** the system navigates to the corresponding subpage.
