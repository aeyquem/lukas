# Proposal

## Why

Lukas currently has a generic lettermark and a plain theme-colored canvas. A Chilean 1,000-peso banknote-inspired visual identity, using Ignacio Carrera Pinto and Torres del Paine, will make the app feel distinctive and complete while grounding it in Chile.

## What Changes

- Use the newly supplied green engraved Ignacio Carrera Pinto portrait as the app icon reference and replace the current lettermark in the application shell.
- Use the supplied Torres del Paine landscape illustration as the page background, keeping the tower summits as its visual focal point.
- Provide genuine SVG artwork for the icon. Use the supplied Torres del Paine engraving as the wallpaper in its most faithful practical format; a PNG is acceptable for the wide background if converting it to SVG reduces detail or quality.
- Let the existing translucent surfaces reveal some of the background while preserving clear separation, readable text, and legible financial information and controls.
- Preserve the existing theme selection and ensure the banknote imagery works with each available theme and on desktop and mobile layouts.

## Capabilities

### New Capabilities
- `app-appearance`: Defines the app's banknote-inspired visual identity, icon, background, and readability behavior.

### Modified Capabilities
- None.

## Impact

- Frontend shell and global styles in `frontend/app/src/app.vue` and `frontend/app/src/styles.css`.
- App icon and static image assets under `frontend/app/public/`.
- Existing theme tokens and appearance behavior in `frontend/app/src/themes/` and `frontend/app/src/app/theme-store.ts`.
- No backend, API, financial behavior, or dependency changes are expected.
