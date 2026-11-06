# Appearance themes

`forest.css` defines the complete default theme contract on `:root`. The
`lagoon` selector demonstrates a palette override; it inherits shared geometry
and materials. `nord.css` adds the dark Nord palette, native dark controls,
and matching feedback colors. Feature components use semantic tokens rather than literal
colors, including financial forms and feedback states.

To add a theme:

1. Add a selector `:root[data-theme='name']` in an imported theme stylesheet.
   Override palette, font stacks, island/control radii, island gap/padding,
   shadows and glass settings as needed. Unspecified tokens inherit Bosque.
2. Register its ID and visible label in `src/app/theme-store.ts`.
3. Check the home, accounts, transactions and budget routes at desktop and
   mobile widths. Check normal, warning, error, focus and disabled states.

The appearance menu stores only its theme ID in `lukas.appearance.theme`.
Storage restrictions leave switching usable for the session. Theme choice is
applied before Vue mounts. Prose uses local/system stacks. Numerical values use bundled Fira Code
WOFF2 subsets, with no font service or new UI dependency required. See
`src/assets/fonts/r-e-a-d-m-e.md` for provenance and licensing.

Glass is progressive enhancement: islands have an opaque base surface,
with a translucent surface and backdrop blur in supported browsers. The
reduced-transparency preference restores opaque surfaces. Form controls and
financial tables stay on solid surfaces. Avoid motion and decorative imagery
behind financial values. Keep text contrast readable in each palette.

Read-only browser validation with the development stack running:

```sh
node frontend/browser-checks/appearance.mjs
```

It checks all four routes at six widths, theme switching and persistence,
and mobile menu placement. It writes desktop/mobile screenshots to `/tmp`.
It creates no financial records.

Nord is selectable as **Nord · Noche** in the same appearance menu and uses
`color-scheme: dark` for native controls. It follows the official palette:
https://www.nordtheme.com/docs/colors-and-palettes/ . Selection stays explicit;
there is no automatic time-based theme change.

Use `.numeric` on displayed monetary values and accounting identifiers.
Decimal/numeric inputs, date inputs, and month inputs use `--font-numeric`.
Keep descriptive labels in the body font. Existing scoped `font: inherit`
shorthands require the numerical input overrides in their owning components.
