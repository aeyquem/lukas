-- Keep every distinct provider value so historical funding effects retain
-- the exact observation that was applied when they were created.
ALTER TABLE daily_fx_observations
    DROP CONSTRAINT daily_fx_observations_pair_date_key;

ALTER TABLE daily_fx_observations
    ADD CONSTRAINT daily_fx_observations_pair_date_rate_key
        UNIQUE (workspace_id, source, base_currency, quote_currency, effective_date, rate);

COMMENT ON COLUMN daily_fx_observations.observed_at IS
    'Last time this exact provider rate was observed; rate values and IDs remain immutable.';
