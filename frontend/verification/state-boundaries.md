# Frontend state boundaries verification

## Baseline (2026-11-06)

The initial working tree was clean. Existing finance-page behavior is the baseline;
no reset or discard was performed. `npm test` in `frontend/app` passed all 28 tests
across seven files before implementation.

`frontend/browser-checks/accounts.mjs` exercises create validation, creation,
rename, opening correction, reload readback and narrow layout. Added an explicit
close/reopen selection assertion so the selected account identity is checked as
part of the same real-API workflow. Initial browser invocation could not reach
the local application from the sandbox; the integration run records real-API
results separately below.

The isolated real-API baseline run at `http://localhost:4174` completed creation,
rename and opening correction, then failed a newly added redundant selected-card
ARIA assertion. That assertion was removed; the close/reopen name assertion
remains the selection regression. This was a harness failure after successful
writes, not evidence of a failed financial command. Final integration reruns
below use the corrected harness.

## Integration (2026-11-06, Arch Linux)

Validation uses a separate `lukas-state-check` Compose project with a new PostgreSQL
volume and API at loopback 4175, through the existing isolated 4174 browser proxy.
The owner application and database were not reset. The previous isolated API was
stopped temporarily to release 4175.

- `npm test` and `npm run build` in `frontend/app`: 44 tests across 11 files and
  Vue type checking / production build passed.
- `LUKAS_BROWSER_ORIGIN=http://localhost:4174 node frontend/browser-checks/budget.mjs`:
  passed against the fresh database, including category operations, incomplete
  valuation, stale assignment draft, overassignment, explicit close, carry/release
  and closed snapshot correction.
- Same origin with `accounts.mjs`, `account-liability.mjs`, `retry.mjs`:
  passed create/rename/correction, selected identity, dirty refetch, paired signed
  USD liability correction, persist-before-send and exact retry across dismissal
  and reload without duplicate effects.
- `account-state-boundaries.mjs`: passed dirty name and opening correction refetch,
  stale captured revisions returning 409, draft retention and explicit reload.
- `transaction-state-boundaries.mjs`: passed mounted shared-account cache
  invalidation while a transaction has dirty amount/note/wallet/opening treatment.
- `shell.mjs`: passed independent URL periods, direct links, active navigation,
  period control routing and header layouts at 1440/1024/390/320 pixels.
- Firefox 158.0.1 and Zen 1.24b `compatibility.mjs`: passed account creation/API
  readback, route reload/navigation and 390px layout. Generation was obtained from
  the disposable database bootstrap before running.
- Google Chrome 157.0.8103.38 `functional-smoke.mjs`: passed account readback,
  route/reload, budget assignment/close preview and 390px layout. Chromium is
  155.0.8059.84.
- `appearance.mjs`: the initial 24 route/viewport/theme checks passed; its final
  font assertion exposed a scoped WalletEditor font override. The override was
  corrected, then `font-focused.mjs` passed in headless Chrome on the Accounts
  route: the opening amount computed to `Lukas Fira Code`, and both 400 and 600
  font faces loaded. The full appearance script could not be rerun after this
  final CSS fix because the disposable Compose stack was no longer running.
- After the final CSS change, `npm test` passed 44 tests across 11 files and
  `npm run build` passed, including Vue type checking and the production build.

Harness repairs preserve existing product behavior: current transaction table
rows replace obsolete card selectors; full API pagination and loading older UI
pages find newly created fixtures regardless of same-date ID ordering; page
reload markers prevent old-document conditions satisfying new-document waits.
The scoped transaction grid sizing fix keeps table scrolling inside its panel;
all 24 appearance route/viewport combinations passed after that fix.
