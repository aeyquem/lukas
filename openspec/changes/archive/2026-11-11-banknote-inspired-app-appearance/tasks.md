# Tasks

## 1. Prepare the engraved vector artwork

- [x] 1.1 Convert the corrected green engraved portrait reference into optimized SVG artwork; verify the SVG contains vector paths rather than an embedded raster and remains recognizable at favicon and shell-icon sizes.
- [x] 1.2 Prepare the supplied Torres del Paine engraving as a high-fidelity wallpaper asset, preferring the supplied PNG where vector tracing loses detail; verify the tower summits stay prominent at desktop and mobile aspect ratios.
- [x] 1.3 Add the finalized portrait SVG and landscape PNG to frontend public assets, mount them into the Vite development container, and verify development and production delivery; record file sizes for review.

## 2. Integrate the portrait identity

- [x] 2.1 Replace the shell lettermark with the portrait SVG while preserving the brand link's accessible name and responsive layout; verify the portrait is visible and recognizable on desktop and mobile.
- [x] 2.2 Update the browser favicon reference to the portrait SVG and verify it loads from the built frontend.

## 3. Integrate the background and validate appearance

- [x] 3.1 Add the Torres del Paine artwork as a decorative page-wide background and tune responsive focal positioning so the towers remain the visual focus; verify representative narrow, desktop, and ultrawide layouts.
- [x] 3.2 Tune translucent shell surfaces and theme tokens while retaining solid financial controls/tables and opaque reduced-transparency behavior; verify foreground readability in Forest, Lagoon, Nord, and reduced-transparency modes.
- [x] 3.3 Extend or update the appearance browser checks to cover the portrait identity, tower focal placement, and content legibility, then run the appearance checks across supported routes and viewport sizes and resolve regressions.
