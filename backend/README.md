# Backend developer notes

## PostgreSQL schema

The database stores financial facts and explicit budget checkpoints separately
from calculated balances. All financial rows carry a `workspace_id`; child
tables use composite `(workspace_id, id)` foreign keys so a record cannot link
to another workspace.

- `financial_accounts` owns one currency wallet for bank, savings, and
  investment accounts; card and cash accounts can own one wallet per currency.
  Cash in another currency may also be represented as a separate cash account.
  `financial_openings` retain signed amounts and business dates per wallet.
  `foreign_liability_bases` hold a separately confirmed CLP amount and
  provenance for a foreign-currency liability opening.
- `financial_transactions` describe income, expense, refund, or transfer
  activity. `financial_wallet_effects` retain signed changes for each affected
  wallet. A refund can link to its source expense; a transfer remains one
  transaction with linked source and destination effects.
- `expense_categories` and `budget_assignments` store owner-managed categories
  and nonnegative assignment intent by calendar month. `budget_funding_effects`
  retain distinct opening, income, transfer, valuation, and correction sources.
- `daily_fx_observations` retain each distinct rate returned by the provider,
  including its source and effective date. A corrected rate is a new immutable
  observation; `observed_at` tracks when that exact value was last seen.
  `financial_corrections` link revisions and reasons to their workspace-scoped
  target records.
- `financial_idempotency_receipts` retain the request identity and minimal
  original response for the lifetime of the workspace. `budget_close_snapshots`
  and `budget_close_category_snapshots` preserve the values and signed
  carry/release choices reviewed at an explicit period close.
  `budget_close_opening_snapshots`, `budget_close_transaction_snapshots`, and
  their rate-observation links retain the per-record contributions and FX facts
  used for that close. `budget_funding_correction_states` and
  `budget_activity_effects` preserve each source record's baseline and latest
  target in an open correction period; the linked `budget_funding_effects`
  row applies the net funding delta during checkpoint replay.

Money columns use unconstrained PostgreSQL `NUMERIC`; do not add a `NUMERIC(p,s)`
typmod, which can round input before application validation. The immutable
`lukas_decimal_in_envelope` check rejects non-finite values, scale above 28, and
scaled mantissas beyond the accepted 96-bit Decimal limit. Business event dates
use `DATE`; audit timestamps use `TIMESTAMPTZ`.

## Migration behavior

Migrations are ordered, forward-only files in `backend/migrations/`. SQLx
applies pending migrations at API startup with its PostgreSQL advisory lock, so
two API instances cannot apply the same migration concurrently. Startup fails
and readiness stays unavailable if connecting or migrating fails. Hosted
deployments can run the same migrator separately with `--migrate-only` before
starting application instances. Do not edit a migration after it has been
applied; add a new migration instead.

The current chain creates workspaces, seeds expense categories, adds financial
records, then makes foreign-exchange observations append-only by rate. The
fourth migration preserves existing observations while allowing a changed
provider rate for the same effective date to be stored as a distinct fact. The
fifth migration adds immutable close facts and linked funding/activity state
for budget corrections.

After restoring a database backup, rotate the installation generation before
starting API instances. This makes pending browser commands from the previous
database state fail with `409 installation_generation_mismatch` instead of
being accepted without their original idempotency receipt:

```sh
docker compose run --rm --entrypoint /usr/local/bin/lukas-api api --rotate-database-generation
```

For local development, start PostgreSQL and the API with the existing Compose
configuration. The API container runs the same startup migrator:

```sh
docker compose up --build -d postgres api
docker compose logs api
docker compose exec postgres psql -U "${POSTGRES_USER:-lukas}" -d "${POSTGRES_DB:-lukas}" -c '\dt'
```

The same command upgrades an existing local database in place. To validate a
fresh migration without changing the main development database, create a
separate database in the Compose PostgreSQL service and run the migration-only
command against it:

```sh
docker compose exec postgres sh -c 'createdb -U "$POSTGRES_USER" lukas_validation'
docker compose run --rm --entrypoint /bin/sh api -c \
  'export DATABASE_URL="${DATABASE_URL%/*}/lukas_validation"; exec /usr/local/bin/lukas-api --migrate-only'
```

`docker compose down --volumes` removes the development database and is only
appropriate when its local data can be discarded. There are no down migrations;
restore a database backup or recreate disposable development data if an
unreleased migration must be abandoned.

The migration and persistence integration tests use SQLx test databases and
need a `DATABASE_URL` whose PostgreSQL role can create databases. Run them with
`cargo test` from this directory when that test database is reachable.

## Monthly budget calculation

Budget periods are calendar months. An opening contributes its signed eligible
CLP value once in its opening month. A foreign asset opening uses the selected
dated FX observation; a foreign liability uses its separately confirmed signed
CLP basis, which daily debt FX does not change. As an implementation
convention, later positive flows retire negative foreign principal and its CLP
basis proportionally; any net positive units are then valued at the selected
dated rate. Eligible income contributes funding in its transaction month.
Income in a tracking-only account does not fund the budget. Expenses and
refunds contribute signed category activity
(spending is positive, refunds are negative); unassigned activity is shown as
uncategorized. Transfers are one boundary event: an eligible source loses the
wallet amount's CLP carrying value immediately before transfer, and an eligible
destination gains the received CLP amount or the selected transaction-date
valuation when no received amount is supplied.

Budget values use explicit states. `complete` includes an amount;
`incomplete` includes a known subtotal and unresolved records; `unavailable`
means a required funding dependency prevents a reliable total. The API does not
return partial totals as complete. Assignments remain editable when activity
or funding is incomplete, but close is blocked until every required value is
complete. A missing or stale FX observation includes its selected source and
effective date in the budget's provenance; a provider outage can use a stored
observation as a marked stale fallback. An unvalued foreign expense remains
visible as an unresolved original-currency amount in its category's activity
and remaining values and in the period activity total, even when it is the
category's only activity.

Assignments are saved owner intent for one month and category, with a revision
for conflict checks. They do not change when an earlier period is recalculated.
For an open period, the server derives ready-to-assign and each category's
remaining amount from the most recent close plus dated openings, transactions,
valuation events, correction effects, and saved assignments.

Close is an explicit chronological checkpoint. Only an ended, complete earliest
unclosed month can close. Preview returns its current period revision and one
proposed `carry` disposition per category; the close command submits that
revision and exactly one `carry` or `release` choice per category. Carry keeps
the signed remainder in that category; release moves it to ready-to-assign.
Unassigned ready-to-assign carries automatically. The close stores immutable
period values, category details, dispositions, and source valuation facts in
one transaction. Later-period reads replay from that checkpoint without
reseeding account balances or changing saved assignments.

Corrections to closed periods leave their snapshots unchanged. The system
posts a linked net budget effect in the current `America/Santiago` month while
it is open, otherwise in the next unclosed month. Repeated corrections in that
month update the linked net effect; after that month closes, a later correction
starts from the prior target and applies only its incremental difference.
