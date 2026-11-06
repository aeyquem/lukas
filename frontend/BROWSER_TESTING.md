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
