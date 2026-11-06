# Numeric typeface

The bundled typeface is **Fira Code Nerd Font Mono**, sourced from this
machine's `ttf-firacode-nerd` package (Regular and SemiBold). It is used under
its SIL Open Font License; see `o-f-l.txt`. Upstream projects:

- https://github.com/tonsky/FiraCode
- https://github.com/ryanoasis/nerd-fonts

These WOFF2 files are subsets made with HarfBuzz's `hb_subset_or_fail` and
`woff2_compress`. Included Unicode ranges: U+0020–024F (Latin and Spanish),
U+2000–206F (punctuation and spacing), U+20A0–20CF (currency symbols), and
U+2212 (minus). Unused Nerd Font icons are excluded. Each weight is roughly
33 KB. Font metadata and license are retained.

CSS names the web face `Lukas Fira Code` to distinguish the bundled subset
from any locally installed font. Regular serves weight 400; SemiBold serves
500–700. `--font-numeric` allows a theme to substitute another numeric face.
Numerals use equal-width lining digits with programming ligatures disabled.
Fallbacks handle glyphs outside the subset. Prose retains `--font-body`.
