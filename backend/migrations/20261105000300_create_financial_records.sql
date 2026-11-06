-- Financial facts are stored separately from derived balances and budget views.
-- Keep NUMERIC unconstrained by typmod so PostgreSQL cannot round on insertion.
CREATE FUNCTION lukas_decimal_in_envelope(value NUMERIC)
RETURNS BOOLEAN
LANGUAGE SQL
IMMUTABLE
STRICT
AS $$
    SELECT CASE
        WHEN value::TEXT IN ('NaN', 'Infinity', '-Infinity') THEN FALSE
        ELSE scale(value) <= 28
            AND abs(value) * power(10::NUMERIC, scale(value))
                <= 79228162514264337593543950335::NUMERIC
    END
$$;

ALTER TABLE expense_categories
    ADD CONSTRAINT expense_categories_workspace_id_key UNIQUE (workspace_id, id);

ALTER TABLE workspaces
    ADD COLUMN installation_generation UUID NOT NULL DEFAULT gen_random_uuid();

CREATE TABLE financial_accounts (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    account_type TEXT NOT NULL CHECK (account_type IN ('bank', 'cash', 'savings', 'card', 'investment')),
    base_currency CHAR(3) NOT NULL CHECK (base_currency ~ '^[A-Z]{3}$'),
    budget_eligible BOOLEAN NOT NULL,
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT financial_accounts_name_nonempty CHECK (length(btrim(name)) > 0),
    CONSTRAINT financial_accounts_workspace_id_key UNIQUE (workspace_id, id)
);

CREATE INDEX financial_accounts_workspace_name_idx
    ON financial_accounts (workspace_id, name, id);

CREATE TABLE financial_wallets (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    account_id UUID NOT NULL,
    currency CHAR(3) NOT NULL CHECK (currency ~ '^[A-Z]{3}$'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT financial_wallets_account_fk
        FOREIGN KEY (workspace_id, account_id)
        REFERENCES financial_accounts (workspace_id, id) ON DELETE CASCADE,
    CONSTRAINT financial_wallets_account_currency_key UNIQUE (workspace_id, account_id, currency),
    CONSTRAINT financial_wallets_workspace_id_key UNIQUE (workspace_id, id)
);

CREATE TABLE financial_openings (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    wallet_id UUID NOT NULL,
    amount NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(amount)),
    business_date DATE NOT NULL,
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT financial_openings_wallet_fk
        FOREIGN KEY (workspace_id, wallet_id)
        REFERENCES financial_wallets (workspace_id, id) ON DELETE CASCADE,
    CONSTRAINT financial_openings_workspace_id_key UNIQUE (workspace_id, id)
);

CREATE INDEX financial_openings_wallet_date_idx
    ON financial_openings (workspace_id, wallet_id, business_date, id);

CREATE TABLE foreign_liability_bases (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    opening_id UUID NOT NULL,
    clp_amount NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(clp_amount)),
    provenance TEXT NOT NULL CHECK (length(btrim(provenance)) > 0),
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT foreign_liability_bases_opening_fk
        FOREIGN KEY (workspace_id, opening_id)
        REFERENCES financial_openings (workspace_id, id) ON DELETE CASCADE,
    CONSTRAINT foreign_liability_bases_workspace_id_key UNIQUE (workspace_id, id),
    CONSTRAINT foreign_liability_bases_opening_key UNIQUE (workspace_id, opening_id)
);

CREATE TABLE financial_transactions (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    transaction_kind TEXT NOT NULL CHECK (transaction_kind IN ('income', 'expense', 'refund', 'transfer')),
    business_date DATE NOT NULL,
    category_id UUID,
    source_expense_id UUID,
    payee TEXT,
    note TEXT,
    confirmed_clp_amount NUMERIC CHECK (confirmed_clp_amount IS NULL OR lukas_decimal_in_envelope(confirmed_clp_amount)),
    conversion_evidence JSONB,
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT financial_transactions_workspace_id_key UNIQUE (workspace_id, id),
    CONSTRAINT financial_transactions_category_fk
        FOREIGN KEY (workspace_id, category_id)
        REFERENCES expense_categories (workspace_id, id),
    CONSTRAINT financial_transactions_source_fk
        FOREIGN KEY (workspace_id, source_expense_id)
        REFERENCES financial_transactions (workspace_id, id),
    CONSTRAINT financial_transactions_refund_link_kind
        CHECK (source_expense_id IS NULL OR transaction_kind = 'refund')
);

CREATE INDEX financial_transactions_workspace_date_idx
    ON financial_transactions (workspace_id, business_date DESC, id DESC);

CREATE TABLE financial_wallet_effects (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    transaction_id UUID NOT NULL,
    wallet_id UUID NOT NULL,
    effect_role TEXT NOT NULL CHECK (effect_role IN ('single', 'source', 'destination')),
    amount NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(amount)),
    opening_treatment TEXT CHECK (opening_treatment IS NULL OR opening_treatment IN ('apply_effect', 'preserve_balance')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT financial_wallet_effects_transaction_fk
        FOREIGN KEY (workspace_id, transaction_id)
        REFERENCES financial_transactions (workspace_id, id) ON DELETE CASCADE,
    CONSTRAINT financial_wallet_effects_wallet_fk
        FOREIGN KEY (workspace_id, wallet_id)
        REFERENCES financial_wallets (workspace_id, id),
    CONSTRAINT financial_wallet_effects_workspace_id_key UNIQUE (workspace_id, id)
);

CREATE INDEX financial_wallet_effects_transaction_idx
    ON financial_wallet_effects (workspace_id, transaction_id, id);
CREATE INDEX financial_wallet_effects_wallet_idx
    ON financial_wallet_effects (workspace_id, wallet_id, transaction_id);

CREATE TABLE financial_idempotency_receipts (
    workspace_id UUID NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    idempotency_key UUID NOT NULL,
    http_method TEXT NOT NULL,
    route TEXT NOT NULL,
    request_hash BYTEA NOT NULL CHECK (octet_length(request_hash) = 32),
    response_status SMALLINT NOT NULL CHECK (response_status BETWEEN 200 AND 299),
    resource_id UUID,
    resource_revision BIGINT CHECK (resource_revision IS NULL OR resource_revision > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (workspace_id, idempotency_key)
);

CREATE TABLE budget_assignments (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    period_start DATE NOT NULL,
    category_id UUID NOT NULL,
    amount NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(amount) AND amount >= 0),
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision > 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT budget_assignments_period_month CHECK (extract(day FROM period_start) = 1),
    CONSTRAINT budget_assignments_category_fk
        FOREIGN KEY (workspace_id, category_id)
        REFERENCES expense_categories (workspace_id, id),
    CONSTRAINT budget_assignments_workspace_period_category_key
        UNIQUE (workspace_id, period_start, category_id),
    CONSTRAINT budget_assignments_workspace_id_key UNIQUE (workspace_id, id)
);

CREATE TABLE daily_fx_observations (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    source TEXT NOT NULL,
    base_currency CHAR(3) NOT NULL CHECK (base_currency ~ '^[A-Z]{3}$'),
    quote_currency CHAR(3) NOT NULL CHECK (quote_currency ~ '^[A-Z]{3}$'),
    effective_date DATE NOT NULL,
    rate NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(rate) AND rate > 0),
    observed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT daily_fx_observations_pair_date_key
        UNIQUE (workspace_id, source, base_currency, quote_currency, effective_date),
    CONSTRAINT daily_fx_observations_workspace_id_key UNIQUE (workspace_id, id)
);

CREATE TABLE financial_corrections (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    record_kind TEXT NOT NULL CHECK (record_kind IN ('account', 'opening', 'liability_basis', 'transaction', 'assignment')),
    account_id UUID,
    opening_id UUID,
    liability_basis_id UUID,
    transaction_id UUID,
    assignment_id UUID,
    prior_revision BIGINT NOT NULL CHECK (prior_revision > 0),
    new_revision BIGINT NOT NULL CHECK (new_revision = prior_revision + 1),
    reason TEXT NOT NULL CHECK (length(btrim(reason)) > 0),
    correction_period DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT financial_corrections_workspace_id_key UNIQUE (workspace_id, id),
    CONSTRAINT financial_corrections_account_fk
        FOREIGN KEY (workspace_id, account_id)
        REFERENCES financial_accounts (workspace_id, id),
    CONSTRAINT financial_corrections_opening_fk
        FOREIGN KEY (workspace_id, opening_id)
        REFERENCES financial_openings (workspace_id, id),
    CONSTRAINT financial_corrections_liability_basis_fk
        FOREIGN KEY (workspace_id, liability_basis_id)
        REFERENCES foreign_liability_bases (workspace_id, id),
    CONSTRAINT financial_corrections_transaction_fk
        FOREIGN KEY (workspace_id, transaction_id)
        REFERENCES financial_transactions (workspace_id, id),
    CONSTRAINT financial_corrections_assignment_fk
        FOREIGN KEY (workspace_id, assignment_id)
        REFERENCES budget_assignments (workspace_id, id),
    CONSTRAINT financial_corrections_target_matches_kind CHECK (
        (record_kind = 'account' AND account_id IS NOT NULL AND opening_id IS NULL AND liability_basis_id IS NULL AND transaction_id IS NULL AND assignment_id IS NULL)
        OR (record_kind = 'opening' AND account_id IS NULL AND opening_id IS NOT NULL AND liability_basis_id IS NULL AND transaction_id IS NULL AND assignment_id IS NULL)
        OR (record_kind = 'liability_basis' AND account_id IS NULL AND opening_id IS NULL AND liability_basis_id IS NOT NULL AND transaction_id IS NULL AND assignment_id IS NULL)
        OR (record_kind = 'transaction' AND account_id IS NULL AND opening_id IS NULL AND liability_basis_id IS NULL AND transaction_id IS NOT NULL AND assignment_id IS NULL)
        OR (record_kind = 'assignment' AND account_id IS NULL AND opening_id IS NULL AND liability_basis_id IS NULL AND transaction_id IS NULL AND assignment_id IS NOT NULL)
    ),
    CONSTRAINT financial_corrections_period_month
        CHECK (correction_period IS NULL OR extract(day FROM correction_period) = 1)
);

CREATE TABLE budget_funding_effects (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    period_start DATE NOT NULL,
    source_kind TEXT NOT NULL CHECK (source_kind IN ('opening', 'income', 'transfer', 'valuation', 'correction')),
    opening_id UUID,
    transaction_id UUID,
    correction_id UUID,
    wallet_id UUID,
    amount NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(amount)),
    currency CHAR(3) NOT NULL CHECK (currency ~ '^[A-Z]{3}$'),
    rate_observation_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT budget_funding_effects_period_month CHECK (extract(day FROM period_start) = 1),
    CONSTRAINT budget_funding_effects_opening_fk
        FOREIGN KEY (workspace_id, opening_id)
        REFERENCES financial_openings (workspace_id, id),
    CONSTRAINT budget_funding_effects_transaction_fk
        FOREIGN KEY (workspace_id, transaction_id)
        REFERENCES financial_transactions (workspace_id, id),
    CONSTRAINT budget_funding_effects_correction_fk
        FOREIGN KEY (workspace_id, correction_id)
        REFERENCES financial_corrections (workspace_id, id),
    CONSTRAINT budget_funding_effects_wallet_fk
        FOREIGN KEY (workspace_id, wallet_id)
        REFERENCES financial_wallets (workspace_id, id),
    CONSTRAINT budget_funding_effects_rate_fk
        FOREIGN KEY (workspace_id, rate_observation_id)
        REFERENCES daily_fx_observations (workspace_id, id),
    CONSTRAINT budget_funding_effects_source_matches_kind CHECK (
        (source_kind = 'opening' AND opening_id IS NOT NULL AND transaction_id IS NULL AND correction_id IS NULL AND wallet_id IS NULL AND rate_observation_id IS NULL)
        OR (source_kind IN ('income', 'transfer') AND opening_id IS NULL AND transaction_id IS NOT NULL AND correction_id IS NULL AND wallet_id IS NULL AND rate_observation_id IS NULL)
        OR (source_kind = 'valuation' AND opening_id IS NULL AND transaction_id IS NULL AND correction_id IS NULL AND wallet_id IS NOT NULL AND rate_observation_id IS NOT NULL)
        OR (source_kind = 'correction' AND opening_id IS NULL AND transaction_id IS NULL AND correction_id IS NOT NULL AND wallet_id IS NULL AND rate_observation_id IS NULL)
    )
);

CREATE UNIQUE INDEX budget_funding_effects_opening_source_key
    ON budget_funding_effects (workspace_id, period_start, opening_id)
    WHERE opening_id IS NOT NULL;
CREATE UNIQUE INDEX budget_funding_effects_transaction_source_key
    ON budget_funding_effects (workspace_id, period_start, source_kind, transaction_id)
    WHERE transaction_id IS NOT NULL;
CREATE UNIQUE INDEX budget_funding_effects_correction_source_key
    ON budget_funding_effects (workspace_id, period_start, correction_id)
    WHERE correction_id IS NOT NULL;
CREATE UNIQUE INDEX budget_funding_effects_valuation_source_key
    ON budget_funding_effects (workspace_id, period_start, wallet_id, rate_observation_id)
    WHERE wallet_id IS NOT NULL;

CREATE TABLE budget_close_snapshots (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    period_start DATE NOT NULL,
    revision BIGINT NOT NULL CHECK (revision > 0),
    funding NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(funding)),
    ready_to_assign NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(ready_to_assign)),
    closed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT budget_close_snapshots_period_month CHECK (extract(day FROM period_start) = 1),
    CONSTRAINT budget_close_snapshots_workspace_period_key UNIQUE (workspace_id, period_start),
    CONSTRAINT budget_close_snapshots_workspace_id_key UNIQUE (workspace_id, id)
);

CREATE TABLE budget_close_category_snapshots (
    workspace_id UUID NOT NULL,
    snapshot_id UUID NOT NULL,
    category_id UUID NOT NULL,
    assigned NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(assigned)),
    activity NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(activity)),
    remaining NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(remaining)),
    disposition TEXT NOT NULL CHECK (disposition IN ('carry', 'release')),
    PRIMARY KEY (workspace_id, snapshot_id, category_id),
    CONSTRAINT budget_close_category_snapshot_fk
        FOREIGN KEY (workspace_id, snapshot_id)
        REFERENCES budget_close_snapshots (workspace_id, id) ON DELETE CASCADE,
    CONSTRAINT budget_close_category_category_fk
        FOREIGN KEY (workspace_id, category_id)
        REFERENCES expense_categories (workspace_id, id)
);
