# Design

## Context

See [proposal.md](proposal.md) for the motivation. The frontend already has a CSS theme token system, a lettermark in the shell and an `island` surface with progressive backdrop blur. Reduced-transparency settings restore opaque islands; financial controls and tables are intended to remain solid. The corrected green engraved icon reference is the user's `/tmp/codex-clipboard-RXWvGJ.png` attachment; its clean matching generated image is `exec-b68f43eb-ef49-4979-b7c5-f6ff56e5f572.png` in the generated image directory. The user later supplied `/tmp/codex-clipboard-3tdk4d.png` as the preferred Torres del Paine engraving. The earlier portrait image is not the icon source.

## Goals / Non-Goals

**Goals:**

- Turn the corrected portrait reference into genuine SVG artwork and retain the supplied landscape engraving in its most faithful practical format, including PNG when vector tracing loses its fine detail.
- Make the portrait the recognizable app identity and make the Torres del Paine towers the background's primary focal point.
- Let the landscape show subtly through the existing shell surfaces without reducing the clarity of financial information or controls.
- Preserve the current theme switcher and reduced-transparency support.

**Non-Goals:**

- Recreate a complete legal-tender banknote, add denomination markings, or change the app's financial workflows.
- Redesign navigation, page information hierarchy, or the theme-selection behavior.
- Add an image or styling dependency.

## Decisions

### Keep the portrait vector and preserve the supplied landscape engraving

Create a genuine SVG asset for the portrait from the corrected supplied artwork. Use the newly attached green engraved portrait as the portrait source and do not use the previously attached portrait. Crop closely around the face and cap, use a stronger ink green, and keep enough simplified engraved linework for the portrait to read at the 42-48px shell-icon size. Use the user's supplied Torres del Paine engraving as the wallpaper; prefer its PNG where tracing loses fine linework and landscape detail. Do not wrap that PNG in an SVG. Store both assets in the app's public directory.

**Alternative considered:** trace the landscape into SVG. That created a much larger, less faithful and visually busy result, so the supplied PNG is preferable for the page-wide wallpaper. The portrait remains SVG because it is a small, persistent app identity asset.

### Integrate assets at the app shell and page-canvas layers

Use the portrait SVG for the shell brand image and browser favicon, replacing the CSS lettermark. Keep the brand link's accessible name and treat the image itself as decorative within that link. Render the landscape PNG as a page-wide decorative background layer behind the existing app shell. Position the source artwork around its tower peaks so the peaks sit near the visual center of the available canvas; use responsive positioning that keeps the image covering the viewport without over-panning on ultrawide screens. A light theme-aware wash can soften the illustration while retaining its etched detail. Mount the frontend `public` directory into the Vite development container so these public assets are available without rebuilding the image.

**Alternative considered:** put the landscape into individual page components. That would duplicate behavior and make the banknote identity inconsistent across routes.

### Reuse translucency selectively

Use the existing island/glass tokens for modest translucency on the shell surfaces, adjusting theme values only as needed after viewing the actual artwork. Keep form controls and financial tables on solid surfaces, following the current appearance guidance. Preserve the opaque fallback and `prefers-reduced-transparency` behavior; do not make blur a requirement for readable content.

**Alternative considered:** make all foreground surfaces translucent. That would let the image interfere with dense financial data and inputs.

### Keep current themes in control of foreground contrast

The illustration can retain its banknote green and ivory colors, while the selected theme continues to control text, borders, feedback states, and opaque/glass surface colors. Check Forest, Lagoon, and Nord individually; the same background treatment must not depend on a light palette.

**Alternative considered:** create a single new banknote theme and make it mandatory. That would remove the user's current appearance choices and is unnecessary for the requested identity.

## Risks / Trade-offs

- [Dense crosshatching can create large SVG files or excessive path detail] → Keep the portrait as vector artwork and measure the landscape PNG; check fidelity at favicon, shell-icon, and viewport sizes.
- [Background cropping can hide or misplace the towers on mobile or ultrawide screens] → Use viewport-specific focal positioning and inspect representative aspect ratios.
- [A pale landscape may reduce contrast in a theme] → Keep the background decorative and subdued, retain solid controls and tables, and validate every existing theme including reduced-transparency mode.

## Migration Plan

Copy and prepare the supplied artwork as app assets, update the shell and canvas styling, and validate all routes and themes at desktop and mobile sizes. Rollback consists of restoring the current favicon, CSS lettermark, and theme canvas rules; no data migration or backend deployment change is required.

## Delivery Measurements

- `ignacio-carrera-pinto.svg`: 465,003 bytes in `public/` and in the production build.
- `torres-del-paine.png`: 3,536,382 bytes in `public/` and in the production build.
- Both development asset URLs returned HTTP 200 with their full file sizes after recreating the Vite container with the public-directory mount.
