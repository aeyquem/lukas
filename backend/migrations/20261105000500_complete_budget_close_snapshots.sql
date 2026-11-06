ALTER TABLE budget_close_snapshots
    ADD COLUMN activity NUMERIC NOT NULL DEFAULT 0
        CHECK (lukas_decimal_in_envelope(activity)),
    ADD COLUMN uncategorized_activity NUMERIC NOT NULL DEFAULT 0
        CHECK (lukas_decimal_in_envelope(uncategorized_activity));

ALTER TABLE budget_close_category_snapshots
    ADD COLUMN category_name TEXT,
    ADD COLUMN category_sort_order SMALLINT;

ALTER TABLE financial_corrections
    ADD COLUMN assignment_target_amount NUMERIC,
    ADD COLUMN opening_target_amount NUMERIC,
    ADD COLUMN opening_target_liability_basis NUMERIC,
    ADD COLUMN opening_target_rate_observation_id UUID,
    ADD COLUMN transaction_target_business_date DATE,
    ADD COLUMN transaction_target_kind TEXT,
    ADD COLUMN transaction_target_category_id UUID,
    ADD COLUMN transaction_target_confirmed_clp_amount NUMERIC,
    ADD COLUMN transaction_target_conversion_evidence JSONB,
    ADD CONSTRAINT financial_corrections_assignment_target_check CHECK (
        assignment_target_amount IS NULL OR
        (record_kind = 'assignment' AND lukas_decimal_in_envelope(assignment_target_amount))
    ),
    ADD CONSTRAINT financial_corrections_opening_target_check CHECK (
        (opening_target_amount IS NULL OR
         (record_kind = 'opening' AND lukas_decimal_in_envelope(opening_target_amount)))
        AND (opening_target_liability_basis IS NULL OR
             (record_kind = 'opening' AND lukas_decimal_in_envelope(opening_target_liability_basis)))
    ),
    ADD CONSTRAINT financial_corrections_transaction_target_check CHECK (
        (transaction_target_business_date IS NULL OR record_kind = 'transaction')
        AND (transaction_target_kind IS NULL OR
             (record_kind = 'transaction' AND transaction_target_kind IN ('income','expense','refund','transfer')))
        AND (transaction_target_confirmed_clp_amount IS NULL OR
             (record_kind = 'transaction' AND lukas_decimal_in_envelope(transaction_target_confirmed_clp_amount)))
    );

ALTER TABLE budget_funding_effects
    ALTER COLUMN amount DROP NOT NULL,
    ADD COLUMN missing_reason TEXT,
    ADD CONSTRAINT budget_funding_effects_missing_value_check CHECK (
        (amount IS NOT NULL AND missing_reason IS NULL)
        OR (amount IS NULL AND missing_reason IS NOT NULL AND source_kind = 'correction')
    );

CREATE TABLE budget_activity_effects (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    period_start DATE NOT NULL,
    correction_id UUID NOT NULL,
    transaction_id UUID NOT NULL,
    category_id UUID,
    baseline_amount NUMERIC CHECK (baseline_amount IS NULL OR lukas_decimal_in_envelope(baseline_amount)),
    target_amount NUMERIC CHECK (target_amount IS NULL OR lukas_decimal_in_envelope(target_amount)),
    baseline_kind TEXT NOT NULL CHECK (baseline_kind IN ('expense','refund')),
    baseline_currency CHAR(3) NOT NULL CHECK (baseline_currency ~ '^[A-Z]{3}$'),
    baseline_original_amount NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(baseline_original_amount)),
    baseline_rate_observation_id UUID,
    baseline_business_date DATE NOT NULL,
    target_kind TEXT NOT NULL CHECK (target_kind IN ('expense','refund')),
    target_currency CHAR(3) NOT NULL CHECK (target_currency ~ '^[A-Z]{3}$'),
    target_original_amount NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(target_original_amount)),
    target_rate_observation_id UUID,
    target_business_date DATE NOT NULL,
    CONSTRAINT budget_activity_effects_period_month CHECK (extract(day FROM period_start) = 1),
    CONSTRAINT budget_activity_effects_correction_fk
        FOREIGN KEY (workspace_id, correction_id)
        REFERENCES financial_corrections (workspace_id, id),
    CONSTRAINT budget_activity_effects_transaction_fk
        FOREIGN KEY (workspace_id, transaction_id)
        REFERENCES financial_transactions (workspace_id, id),
    CONSTRAINT budget_activity_effects_category_fk
        FOREIGN KEY (workspace_id, category_id)
        REFERENCES expense_categories (workspace_id, id),
    CONSTRAINT budget_activity_effects_rate_fk
        FOREIGN KEY (workspace_id, target_rate_observation_id)
        REFERENCES daily_fx_observations (workspace_id, id),
    CONSTRAINT budget_activity_effects_baseline_rate_fk
        FOREIGN KEY (workspace_id, baseline_rate_observation_id)
        REFERENCES daily_fx_observations (workspace_id, id)
);

CREATE UNIQUE INDEX budget_activity_effects_source_category_key
    ON budget_activity_effects (workspace_id, period_start, transaction_id, category_id)
    WHERE category_id IS NOT NULL;

CREATE UNIQUE INDEX budget_activity_effects_uncategorized_key
    ON budget_activity_effects (workspace_id, period_start, transaction_id)
    WHERE category_id IS NULL;

CREATE TABLE budget_funding_correction_states (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    period_start DATE NOT NULL,
    correction_id UUID NOT NULL,
    source_kind TEXT NOT NULL CHECK (source_kind IN ('opening','transaction')),
    opening_id UUID,
    transaction_id UUID,
    baseline_amount NUMERIC CHECK (baseline_amount IS NULL OR lukas_decimal_in_envelope(baseline_amount)),
    target_amount NUMERIC CHECK (target_amount IS NULL OR lukas_decimal_in_envelope(target_amount)),
    missing_reason TEXT,
    baseline_rate_observation_id UUID,
    baseline_additional_rate_observation_id UUID,
    target_rate_observation_id UUID,
    target_additional_rate_observation_id UUID,
    baseline_business_date DATE NOT NULL,
    target_business_date DATE NOT NULL,
    currency CHAR(3) NOT NULL CHECK (currency ~ '^[A-Z]{3}$'),
    CONSTRAINT budget_funding_correction_states_period_month CHECK (extract(day FROM period_start) = 1),
    CONSTRAINT budget_funding_correction_states_source_check CHECK (
        (source_kind='opening' AND opening_id IS NOT NULL AND transaction_id IS NULL)
        OR (source_kind='transaction' AND opening_id IS NULL AND transaction_id IS NOT NULL)
    ),
    CONSTRAINT budget_funding_correction_states_value_check CHECK (
        (target_amount IS NOT NULL AND missing_reason IS NULL)
        OR (target_amount IS NULL AND missing_reason IS NOT NULL)
    ),
    CONSTRAINT budget_funding_correction_states_correction_fk
        FOREIGN KEY (workspace_id, correction_id)
        REFERENCES financial_corrections (workspace_id, id),
    CONSTRAINT budget_funding_correction_states_opening_fk
        FOREIGN KEY (workspace_id, opening_id)
        REFERENCES financial_openings (workspace_id, id),
    CONSTRAINT budget_funding_correction_states_transaction_fk
        FOREIGN KEY (workspace_id, transaction_id)
        REFERENCES financial_transactions (workspace_id, id),
    CONSTRAINT budget_funding_correction_states_baseline_rate_fk
        FOREIGN KEY (workspace_id, baseline_rate_observation_id)
        REFERENCES daily_fx_observations (workspace_id, id),
    CONSTRAINT budget_funding_correction_states_target_rate_fk
        FOREIGN KEY (workspace_id, target_rate_observation_id)
        REFERENCES daily_fx_observations (workspace_id, id),
    CONSTRAINT budget_funding_correction_states_baseline_additional_rate_fk
        FOREIGN KEY (workspace_id, baseline_additional_rate_observation_id)
        REFERENCES daily_fx_observations (workspace_id, id),
    CONSTRAINT budget_funding_correction_states_target_additional_rate_fk
        FOREIGN KEY (workspace_id, target_additional_rate_observation_id)
        REFERENCES daily_fx_observations (workspace_id, id)
);

CREATE UNIQUE INDEX budget_funding_correction_states_opening_key
    ON budget_funding_correction_states (workspace_id, period_start, opening_id)
    WHERE opening_id IS NOT NULL;
CREATE UNIQUE INDEX budget_funding_correction_states_transaction_key
    ON budget_funding_correction_states (workspace_id, period_start, transaction_id)
    WHERE transaction_id IS NOT NULL;

CREATE TABLE budget_funding_correction_rate_observations (
    workspace_id UUID NOT NULL,
    correction_id UUID NOT NULL,
    rate_observation_id UUID NOT NULL,
    PRIMARY KEY (workspace_id, correction_id, rate_observation_id),
    CONSTRAINT budget_funding_correction_rates_correction_fk
        FOREIGN KEY (workspace_id, correction_id)
        REFERENCES financial_corrections (workspace_id, id) ON DELETE CASCADE,
    CONSTRAINT budget_funding_correction_rates_observation_fk
        FOREIGN KEY (workspace_id, rate_observation_id)
        REFERENCES daily_fx_observations (workspace_id, id)
);

CREATE TABLE budget_close_opening_snapshots (
    workspace_id UUID NOT NULL,
    snapshot_id UUID NOT NULL,
    opening_id UUID NOT NULL,
    business_date DATE NOT NULL,
    amount NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(amount)),
    currency CHAR(3) NOT NULL CHECK (currency ~ '^[A-Z]{3}$'),
    funding_amount NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(funding_amount)),
    liability_clp_basis NUMERIC CHECK (liability_clp_basis IS NULL OR lukas_decimal_in_envelope(liability_clp_basis)),
    rate_observation_id UUID,
    PRIMARY KEY (workspace_id, snapshot_id, opening_id),
    CONSTRAINT budget_close_opening_snapshot_fk
        FOREIGN KEY (workspace_id, snapshot_id)
        REFERENCES budget_close_snapshots (workspace_id, id) ON DELETE CASCADE,
    CONSTRAINT budget_close_opening_source_fk
        FOREIGN KEY (workspace_id, opening_id)
        REFERENCES financial_openings (workspace_id, id),
    CONSTRAINT budget_close_opening_rate_fk
        FOREIGN KEY (workspace_id, rate_observation_id)
        REFERENCES daily_fx_observations (workspace_id, id)
);

CREATE TABLE budget_close_transaction_snapshots (
    workspace_id UUID NOT NULL,
    snapshot_id UUID NOT NULL,
    transaction_id UUID NOT NULL,
    business_date DATE NOT NULL,
    transaction_kind TEXT NOT NULL CHECK (transaction_kind IN ('income','expense','refund','transfer')),
    category_id UUID,
    currency CHAR(3) NOT NULL CHECK (currency ~ '^[A-Z]{3}$'),
    original_amount NUMERIC NOT NULL CHECK (lukas_decimal_in_envelope(original_amount)),
    confirmed_clp_amount NUMERIC CHECK (confirmed_clp_amount IS NULL OR lukas_decimal_in_envelope(confirmed_clp_amount)),
    conversion_evidence JSONB,
    activity_amount NUMERIC CHECK (activity_amount IS NULL OR lukas_decimal_in_envelope(activity_amount)),
    activity_kind TEXT CHECK (activity_kind IS NULL OR activity_kind IN ('expense','refund')),
    activity_rate_observation_id UUID,
    funding_amount NUMERIC CHECK (funding_amount IS NULL OR lukas_decimal_in_envelope(funding_amount)),
    funding_missing_reason TEXT,
    funding_rate_observation_id UUID,
    funding_additional_rate_observation_id UUID,
    PRIMARY KEY (workspace_id, snapshot_id, transaction_id),
    CONSTRAINT budget_close_transaction_snapshot_fk
        FOREIGN KEY (workspace_id, snapshot_id)
        REFERENCES budget_close_snapshots (workspace_id, id) ON DELETE CASCADE,
    CONSTRAINT budget_close_transaction_source_fk
        FOREIGN KEY (workspace_id, transaction_id)
        REFERENCES financial_transactions (workspace_id, id),
    CONSTRAINT budget_close_transaction_category_fk
        FOREIGN KEY (workspace_id, category_id)
        REFERENCES expense_categories (workspace_id, id),
    CONSTRAINT budget_close_transaction_activity_rate_fk
        FOREIGN KEY (workspace_id, activity_rate_observation_id)
        REFERENCES daily_fx_observations (workspace_id, id),
    CONSTRAINT budget_close_transaction_funding_rate_fk
        FOREIGN KEY (workspace_id, funding_rate_observation_id)
        REFERENCES daily_fx_observations (workspace_id, id),
    CONSTRAINT budget_close_transaction_funding_additional_rate_fk
        FOREIGN KEY (workspace_id, funding_additional_rate_observation_id)
        REFERENCES daily_fx_observations (workspace_id, id),
    CONSTRAINT budget_close_transaction_funding_value_check CHECK (
        (funding_amount IS NOT NULL AND funding_missing_reason IS NULL)
        OR (funding_amount IS NULL AND funding_missing_reason IS NOT NULL)
    )
);

CREATE TABLE budget_close_transaction_rate_observations (
    workspace_id UUID NOT NULL,
    snapshot_id UUID NOT NULL,
    transaction_id UUID NOT NULL,
    rate_observation_id UUID NOT NULL,
    PRIMARY KEY (workspace_id, snapshot_id, transaction_id, rate_observation_id),
    CONSTRAINT budget_close_transaction_rates_snapshot_fk
        FOREIGN KEY (workspace_id, snapshot_id, transaction_id)
        REFERENCES budget_close_transaction_snapshots (workspace_id, snapshot_id, transaction_id)
        ON DELETE CASCADE,
    CONSTRAINT budget_close_transaction_rates_observation_fk
        FOREIGN KEY (workspace_id, rate_observation_id)
        REFERENCES daily_fx_observations (workspace_id, id)
);

