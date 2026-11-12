# Browser support and workflow checks

The target browsers are current stable Firefox and Chrome; run a functional
smoke check in Zen as well. Record tested versions, operating system, date, and
result before describing a browser as verified.

For each browser, check:

1. Route navigation and refresh on the account, transaction, and budget pages.
2. Create an account and verify it through the API.
3. Load the app at a 390px viewport and confirm the page does not overflow.

Run transaction entry/correction and budget assignment/close-preview checks in
Chrome and Chromium as well. These flows exercise the browser-facing financial
forms in addition to the cross-browser route smoke.

Formal accessibility review, including keyboard-only, ARIA, and screen-reader
review, is deferred until the final UI design and is not an MVP gate.

## Functional browser smoke record

On 2026-11-06, Firefox 158.0.1 and Zen 1.24b passed smoke checks through the
isolated verification proxy. They created and read back an account, navigated
and refreshed the transaction, budget, and account routes, and passed the 390px
viewport check. Chromium 155.0.8059.84 passed the detailed account, transaction,
and budget workflows documented below. Google Chrome 157.0.8103.38 passed the
cross-browser smoke plus transaction entry/correction and budget assignment/
close-preview checks on the development app.

## Automated Chromium workflow checks

Run these from the host shell in `tmux attach -t Interactive` with the Compose
stack running. They use Chromium's browser protocol and native Node APIs; no
additional npm dependency is required. Use only a development database: these
checks retain clearly named `Browser ...` account and transaction records. Each
run starts an isolated Chromium profile with its own debugging port.

```sh
node frontend/browser-checks/accounts.mjs
node frontend/browser-checks/account-liability.mjs
node frontend/browser-checks/retry.mjs
node frontend/browser-checks/transactions.mjs
node frontend/browser-checks/persistence.mjs snapshot
sudo docker compose restart api
node frontend/browser-checks/persistence.mjs verify
```

Run the functional browser smoke against Chrome with:

```sh
LUKAS_BROWSER_BIN=google-chrome-stable LUKAS_BROWSER_ORIGIN=http://localhost:4173 node frontend/browser-checks/functional-smoke.mjs
LUKAS_BROWSER_BIN=google-chrome-stable LUKAS_BROWSER_ORIGIN=http://localhost:4173 node frontend/browser-checks/transactions.mjs
```

It creates one clearly named `Browser functional smoke ...` account in the
development database, then checks account API read-back, budget assignment and
close preview, route navigation and refresh, and the 390px viewport. The
transaction script checks entry, correction, and pagination; it also adds
clearly named development records.

Run the Firefox or Zen cross-browser smoke through the isolated verification
proxy. Set `LUKAS_BROWSER_GENERATION` to the `installation_generation` returned
by `http://localhost:4174/api/bootstrap`:

```sh
LUKAS_COMPAT_BROWSER=firefox LUKAS_BROWSER_GENERATION=<generation> node frontend/browser-checks/compatibility.mjs
LUKAS_COMPAT_BROWSER=zen-browser LUKAS_BROWSER_GENERATION=<generation> node frontend/browser-checks/compatibility.mjs
```

The transaction check expects the `Browser verification bank ...` and
`Browser check cash ...` accounts recorded below; run the account checks first
and keep a cash account with a CLP wallet in the development database.

Recorded on 2026-11-05, Arch Linux, Chromium 155.0.8059.84:

- Account creation, name edit, opening correction, required-field association,
  authoritative reload read-back, and 390px viewport passed.
- Investment tracking-only default, immutable created account type, and USD
  opening correction paired with its signed CLP basis passed.
- A real committed account write with its response deliberately discarded
  remained pending across dismissal and reload. Retry preserved all bytes and
  headers, and returned one account with one opening, without duplicate effects.
- An invalid multi-wallet bank POST returned 422 and left the account list
  unchanged. All nine accounts read back identically after restarting the API.
- Income, categorized expense, linked refund category inheritance, same-currency
  transfer, explicit same-day opening treatment, and resulting wallet balances
  passed through browser → proxy → API → PostgreSQL.
- The isolated transactions flow passed same- and cross-currency transfers with
  conversion provenance, foreign expense confirmation and incomplete-value
  behavior, validation, correction, and deterministic pagination over 259 rows.
- The isolated budget flow passed category management, incomplete-value
  explanations, assignment and overassignment, stale draft retention, close,
  carry/release, closed-period correction, and PostgreSQL snapshot read-back.

Budget-close scenarios use the isolated `lukas_browser_verify_20261105`
database; do not repeat them against the main development database.

## Island appearance check

On 2026-11-06, Chromium 155.0.8059.84 on Arch Linux passed the read-only
`appearance.mjs` check against the live Compose development stack. Home,
accounts, transactions and budget fit 1440, 1024, 768, 650, 390 and 320px
viewports without document overflow. Theme switching updates the palette,
survives reload, and the appearance menu fits the mobile viewport. Desktop
and mobile screenshots were visually inspected. The budget table keeps its
own horizontal scroll container on narrow screens. No financial records are
created by this check. Firefox, Chrome and Zen have not been rerun for this
appearance change.

Nord and numeric-font follow-up: the read-only appearance script also checks
Nord selection/persistence, native dark color scheme, both bundled webfonts,
and numerical input styles on the mobile financial screens. These additions
have not been executed: local privileged session access was blocked by an
automatic approval-review usage limit. Build/type-check and static font and
contrast validation are recorded separately; do not treat them as browser QA.

## Grouped monthly budget workspace check

`frontend/browser-checks/budget-workspace.mjs` verifies the rewritten workspace
through Chromium, the real Rust API, and PostgreSQL. Run it only through the
isolated verification proxy on `http://localhost:4174`, backed by a freshly
created disposable database and the current backend build. It refuses to run
against the normal development origin or an account list that already contains
records. It retains its named fixtures and closes the previous calendar month.
A subsequent run requires a new disposable database; do not reset the owner's
development database to rerun it.

```sh
LUKAS_BROWSER_ORIGIN=http://localhost:4174 node frontend/browser-checks/budget-workspace.mjs
```

The check covers the single table named Presupuesto, the separate category
administration tab, group creation/rename/archive, keyboard category order, cross-group moves,
group drag persistence, active-group archive rejection, arrangement locking, drawer creation/edit/archive and
guarded deletion, decimal-string inline Enter saves, explicit stale-draft review,
committed-response loss and exact retry after reload, incomplete foreign activity,
actual close releases versus ordinary category carry, closed correction reasons,
immutable historical labels/totals, and a 390px drawer with focus restoration.
The older `budget.mjs` targets the previous category manager and archive rules;
use `budget-workspace.mjs` for the grouped budget workspace. Execution results
are recorded separately from syntax validation.

Recorded 2026-11-11, Chromium 155.0.8059.84 on Arch Linux: the complete
workspace workflow passed against the isolated `lukas_workspace_verify`
PostgreSQL database and Rust API. It verified the separate administration tab,
single Presupuesto table, visible release/carry explanation, exact retry after
reload, closed correction reason, archived history, and the 390px drawer's
focus and width behavior. The read-only 813×693 screenshot was visually
reviewed against the budget workspace model supplied for this change.


## Budget table interaction regression

With the local Vite development server running on port 4180:

```sh
LUKAS_BROWSER_ORIGIN=http://127.0.0.1:4180 node frontend/browser-checks/budget-table.mjs
```

This isolated Chromium check mounts the real table and assignment workflow with
in-memory API fixtures; it does not use PostgreSQL or write financial records.
It covers clear currency headers, display-to-editor focus, icon and Enter saves,
exact decimal request amounts, invalid input, rejected/stale draft retention,
category drops onto empty groups and category rows, pending organization locks,
closed correction reasons, and mobile overflow. The workspace integration check
opens assignment editors explicitly and keeps group edit-mode assertions separate
from always-available category dragging.

Recorded 2026-11-11, Chromium 155.0.8059.84 on Arch Linux: this isolated
regression passed, including native mouse dragging into an empty group,
synthetic drop events on category rows, both assignment save controls, retained
failed/stale drafts, required closed correction reasons, and 390px page width.
The PostgreSQL workspace integration check was updated but was not rerun for
this follow-up.
