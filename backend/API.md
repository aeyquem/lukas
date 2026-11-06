# Accounts and transaction API

The API is workspace-scoped. The current self-hosted installation has one owner workspace. Send dates as calendar dates (`YYYY-MM-DD`) and money as decimal strings; the server returns money as decimal strings, including signed wallet effects and balances. Do not send JSON floating point numbers for money.

Financial writes require an `Idempotency-Key` UUID and the latest `Installation-Generation` UUID from `GET /api/bootstrap`. The server validates the generation while holding the workspace write lock before looking up receipts or applying a mutation. Repeating the same method, route, key, generation, and exact request body returns the original receipt. Reusing the key with different content returns `409 idempotency_key_reused`. A generation mismatch returns `409 installation_generation_mismatch` and applies no write. Account, opening, transaction, and assignment edits also require the current revision. A stale revision returns `409 stale_revision`.

## Bootstrap and accounts

| Method and route | Purpose | Success |
| --- | --- | --- |
| `GET /api/bootstrap` | Workspace generation and expense categories | `200` |
| `GET /api/accounts` | Workspace accounts and wallet balances | `200` |
| `POST /api/accounts` | Create account, wallets, and dated openings atomically | `201` |
| `GET /api/accounts/{account_id}` | Read one account with wallet balances and opening provenance | `200` |
| `PUT /api/accounts/{account_id}` | Rename an account | `200` |
| `PUT /api/accounts/{account_id}/openings/{opening_id}` | Correct an opening amount and any paired foreign-liability CLP basis | `200` |

Account creation accepts an explicit eligibility value except for investments, which default to tracking-only:

```http
POST /api/accounts
Idempotency-Key: 237a2988-2a9e-4d49-830b-1244d5a1f01a
Installation-Generation: 5248fd46-2656-4f83-8db5-a2463eaac0a7
Content-Type: application/json

{
  "name": "Travel card",
  "account_type": "card",
  "base_currency": "CLP",
  "budget_eligible": true,
  "wallets": [
    { "currency": "CLP", "opening_amount": "-125000", "opening_date": "2026-11-01" },
    { "currency": "USD", "opening_amount": "-250.50", "opening_date": "2026-11-01", "liability_clp_basis": "-23800" }
  ]
}
```

Bank, savings, and investment accounts accept exactly one currency wallet.
Card and cash accounts may accept one wallet per currency. Cash held in
different currencies may also be represented by separate cash accounts. A
bank, savings, or investment account create request with multiple wallets
returns `422 validation_failed` with a `wallets` field error
(`multiple_currencies_unsupported_for_account_type`) and creates no records.

A mutation receipt contains the resource identity and resulting revision:

```json
{ "resource_id": "b8f8cf62-8c56-4b77-a345-2e6f76222ced", "revision": 1 }
```

Account edits currently change descriptive fields only. Opening entries count as activity, so account type, base currency, wallet currency, and budget eligibility remain fixed after creation. An opening correction requires `amount`, `expected_revision`, and a non-empty `reason`. For a foreign opening liability whose principal changes, include `liability_clp_basis`; include `expected_basis_revision` only when a basis record already exists. The signed original amount and CLP basis are saved atomically.

```http
PUT /api/accounts/b8f8cf62-8c56-4b77-a345-2e6f76222ced
Idempotency-Key: 2cc3d4f2-bf2d-4ac7-b374-5dcdf2c40e83
Installation-Generation: 5248fd46-2656-4f83-8db5-a2463eaac0a7
Content-Type: application/json

{ "name": "Daily spending", "expected_revision": 1 }
```

Account reads return each wallet's currency, signed balance, and dated opening entries, including liability-basis provenance and revisions when present. Balances are derived from stored financial facts; clients should refetch the account after a successful mutation.

## Transactions

| Method and route | Purpose | Success |
| --- | --- | --- |
| `POST /api/transactions` | Record income, expense, refund, or transfer | `201` |
| `GET /api/transactions` | List a deterministic page of workspace transactions | `200` |
| `GET /api/transactions/{transaction_id}` | Read a transaction and its wallet effects | `200` |
| `PUT /api/transactions/{transaction_id}` | Correct a transaction under its current revision | `200` |

Income, expenses, and refunds use one tracked wallet and a positive amount in that wallet's currency. Expense categories are optional. A refund may reference an expense; its category follows the source expense. A foreign expense can include a positive `confirmed_clp_amount` when its statement value is known. Without it, the original-currency effect is retained while dependent CLP budget calculations remain incomplete. `opening_treatment` is `apply_effect` or `preserve_balance` when the transaction is dated before or on the wallet's opening date; it is omitted when no choice is needed.

```json
{
  "kind": "expense",
  "business_date": "2026-11-05",
  "wallet_id": "0b479683-1a9a-48ef-9c89-94e431dd3e6a",
  "amount": "24.90",
  "category_id": "805b371e-c8aa-49f7-a893-7ef28984db77",
  "payee": "Market",
  "note": "Groceries",
  "confirmed_clp_amount": "23800"
}
```

A transfer is one logical transaction with source and destination wallet effects. Same-currency amounts must match. Cross-currency transfers preserve both actual amounts and require inspectable `conversion_evidence`.

```json
{
  "kind": "transfer",
  "business_date": "2026-11-05",
  "source_wallet_id": "0b479683-1a9a-48ef-9c89-94e431dd3e6a",
  "source_amount": "100",
  "destination_wallet_id": "d5c14d3c-0fac-4b4c-a660-76e9988a6817",
  "destination_amount": "92.15",
  "conversion_evidence": { "source": "bank statement", "rate": "0.9215" }
}
```

An edit submits a full transaction representation, including required `kind` and `business_date`, plus `expected_revision` and a non-empty `reason`. The actual date changes only when the request explicitly supplies a corrected date. Edits replace the transaction's effects and append correction history atomically. If an edited expense has linked refunds, category changes (including clearing the category) are propagated to those refunds; changing the source away from an expense is rejected while refunds reference it.

List pages default to 50 items and allow 1–100. Ordering is descending by `(business_date, id)`. A non-null `next` object supplies the cursor for the next request:

```http
GET /api/transactions?page_size=50
```

```json
{
  "items": [
    {
      "id": "e708cc8a-2716-4ef5-8a15-1533bc34f180",
      "kind": "income",
      "business_date": "2026-11-04",
      "category_id": null,
      "source_expense_id": null,
      "payee": "Employer",
      "note": null,
      "confirmed_clp_amount": null,
      "conversion_evidence": null,
      "revision": 1,
      "effects": [
        {
          "wallet_id": "0b479683-1a9a-48ef-9c89-94e431dd3e6a",
          "currency": "CLP",
          "role": "single",
          "amount": "125000",
          "opening_treatment": null
        }
      ]
    }
  ],
  "next": { "before_date": "2026-11-04", "before_id": "e708cc8a-2716-4ef5-8a15-1533bc34f180" }
}
```

To request the following page, pass the returned `next.before_date` and `next.before_id` as `before_date` and `before_id` query parameters.

## Categories and monthly budgets

| Method and route | Purpose | Success |
| --- | --- | --- |
| `GET /api/categories` | List workspace categories, including archived categories | `200` |
| `POST /api/categories` | Create a custom category | `201` |
| `PUT /api/categories/{category_id}` | Rename, reorder, or archive a category | `200` |
| `GET /api/budgets/{YYYY-MM}` | Read calculated or immutable closed-period values | `200` |
| `PUT /api/budgets/{YYYY-MM}/assignments/{category_id}` | Save the assignment for one category and month | `200` |
| `GET /api/budgets/{YYYY-MM}/close-preview` | Read values, eligibility, revision, and proposed dispositions | `200` |
| `POST /api/budgets/{YYYY-MM}/close` | Close an eligible period with explicit dispositions | `201` |

Category creation accepts `{ "name": "Pet care", "sort_order": 30 }`.
Category edits send the full editable representation, for example
`{ "name": "Pets", "sort_order": 30, "archived": false }`. Archiving is
rejected with `409 category_in_use` while any transaction or assignment refers
to the category. Starter categories are seeded idempotently during workspace
bootstrap. Expenses without a category remain visible in
`uncategorized_activity`.

Budget reads return server-calculated values and a revision fingerprint over
the inputs. A money value is a tagged object: `complete` includes `amount`,
`incomplete` includes a known subtotal and unresolved source details, and
`unavailable` includes reasons when a required dependency prevents a reliable
total. Signed activity and remaining values are preserved; the server does not
clamp negatives or reject an assignment merely because it exceeds available
funding. Assignments are saved owner intent and do not alter calculated values
such as activity or funding.

An assignment request contains a nonnegative decimal-string `amount`, the
latest `expected_period_revision`, and `expected_revision` (`null` when no
assignment exists yet). For a closed period, also provide a non-empty
`reason`; the immutable closed `assigned` amount remains unchanged while
`assignment_intent` exposes the latest corrected target. A successful write
returns a receipt with the new assignment revision and period revision. Stale
assignment or period revisions return `409 stale_revision`.

Close preview returns `close.eligible`, an optional stable `close.reason`, the
`close.expected_period_revision`, and one proposed `carry` disposition for each
category. Submit the same revision and exactly one `carry` or `release` choice
per category:

```json
{
  "expected_period_revision": "<period-fingerprint>",
  "categories": [
    { "category_id": "805b371e-c8aa-49f7-a893-7ef28984db77", "disposition": "carry" }
  ]
}
```

Only the earliest unclosed period after its calendar month ends can close, and
all required values must be complete. An incomplete close returns `422` with
the `period: incomplete_values` field error; current, out-of-order, and
already-closed attempts return stable `409` conflicts. Close writes the
calculated period, category choices, and valuation provenance atomically.
Exact command retries replay the original receipt. Closed snapshots remain
unchanged after later corrections; linked net corrections post to the current
open `America/Santiago` month or, if it is closed, the next unclosed month.

## Error responses

Errors returned by matched API routes use a stable envelope with a request ID. Path and query deserialization errors use `400 invalid_request_parameter`; validation field details are safe for display and do not include database diagnostics.

```json
{
  "error": {
    "code": "validation_failed",
    "message": "Review the highlighted fields.",
    "fields": [{ "field": "opening_treatment", "code": "required_for_opening_date" }]
  },
  "request_id": "bb6f1159-6c5e-477a-a8ca-2861660881f0"
}
```

| Status | Stable codes | Meaning |
| --- | --- | --- |
| `400` | Account routes: `malformed_json`, `idempotency_key_required`, `invalid_idempotency_key`; transaction routes: `invalid_json`; all routes: `invalid_request_parameter` | Malformed JSON, malformed path/query values, or missing/malformed account-route idempotency key |
| `422` | `validation_failed` | Syntactically valid request that violates a field or domain rule |
| `404` | `not_found` | Resource does not exist in this workspace |
| `409` | `stale_revision`, `idempotency_key_reused`, `installation_generation_mismatch` | Revision conflict, key bound to different request content, or write bound to another database generation |
| `503` | `storage_unavailable`, `storage_busy` | Storage unavailable, or a confirmed serialization/deadlock abort remained after bounded server retries; retry explicitly with the same key and exact body |
| `500` | `internal_error` | Unexpected server or database failure; database details are logged server-side only |

Account write routes return the `400` idempotency-key codes shown above. Transaction write routes report a missing or malformed key as `422 validation_failed` with an `Idempotency-Key` field error.

