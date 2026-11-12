# Spec Delta

## Purpose

Defines the app's visual identity and appearance so owners experience a cohesive Chilean banknote-inspired interface while financial content remains easy to read and use.

## ADDED Requirements

### Requirement: Show the selected portrait as the app identity
The application SHALL provide the newly supplied green engraved Ignacio Carrera Pinto portrait as genuine SVG artwork and use it as the app icon and brand image in the application shell.

#### Scenario: Owner opens the application
- **WHEN** the owner views the application shell or its browser tab
- **THEN** the selected portrait is shown as the app's brand image and icon
- **AND** the portrait artwork scales cleanly without relying on an embedded raster image

### Requirement: Show Torres del Paine as the app background
The application SHALL use the supplied engraved Torres del Paine landscape as a decorative page background, with the tower summits as its visual focal point. The wallpaper MAY use PNG when that preserves the supplied engraving more faithfully; the portrait icon SHALL remain genuine SVG artwork.

#### Scenario: Owner views a finance page
- **WHEN** the owner views a page at desktop or mobile width
- **THEN** the Torres del Paine towers remain the background's focal point without covering or competing with page content
- **AND** the supplied landscape artwork scales cleanly while retaining its engraved detail and focal position

### Requirement: Preserve foreground readability over the background
The application SHALL keep text, financial values, controls, and status indicators visually distinct and readable over the decorative background, including when content surfaces are translucent.

#### Scenario: Owner uses content over the background
- **WHEN** foreground content is displayed over the illustrated background
- **THEN** surface transparency may reveal the background subtly while foreground content remains clearly separated and legible

#### Scenario: Reduced transparency is preferred
- **WHEN** the owner has enabled the operating system's reduced-transparency preference
- **THEN** content surfaces use an opaque treatment that preserves readability

### Requirement: Retain appearance theme support
The banknote-inspired identity SHALL remain usable with every available appearance theme.

#### Scenario: Owner changes the appearance theme
- **WHEN** the owner selects any available theme
- **THEN** the app icon and background remain visible and foreground content retains clear contrast
