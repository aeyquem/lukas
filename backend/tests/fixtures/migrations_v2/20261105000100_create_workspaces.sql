CREATE TABLE workspaces (
    id UUID PRIMARY KEY,
    singleton_key TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT workspaces_single_owner CHECK (singleton_key = 'owner')
);
