CREATE TABLE expense_categories (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    sort_order SMALLINT NOT NULL,
    archived BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT expense_categories_workspace_slug_key UNIQUE (workspace_id, slug),
    CONSTRAINT expense_categories_sort_order_nonnegative CHECK (sort_order >= 0)
);

CREATE INDEX expense_categories_workspace_order_idx
    ON expense_categories (workspace_id, sort_order, id);
