-- Add organization without changing category identities or financial facts.
CREATE TABLE budget_category_groups (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    slug TEXT NOT NULL,
    name TEXT NOT NULL CHECK (length(btrim(name)) > 0),
    sort_order SMALLINT NOT NULL CHECK (sort_order >= 0),
    archived BOOLEAN NOT NULL DEFAULT FALSE,
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (workspace_id, id),
    UNIQUE (workspace_id, slug)
);
INSERT INTO budget_category_groups (id, workspace_id, slug, name, sort_order)
SELECT gen_random_uuid(), w.id, g.slug, g.name, g.sort_order
FROM workspaces w CROSS JOIN (VALUES
    ('general', 'General', 1::SMALLINT),
    ('transport', 'Transporte', 2::SMALLINT),
    ('savings', 'Ahorro', 3::SMALLINT)
) AS g(slug, name, sort_order);
ALTER TABLE expense_categories ADD COLUMN group_id UUID;
ALTER TABLE expense_categories ADD COLUMN revision BIGINT NOT NULL DEFAULT 1 CHECK (revision > 0);
UPDATE expense_categories c SET group_id = g.id
FROM budget_category_groups g WHERE c.workspace_id = g.workspace_id AND g.slug = CASE
    WHEN c.slug = 'transporte' THEN 'transport'
    WHEN c.slug = 'ahorro-general' THEN 'savings'
    ELSE 'general' END;
ALTER TABLE expense_categories ALTER COLUMN group_id SET NOT NULL;
ALTER TABLE expense_categories ADD CONSTRAINT expense_categories_group_workspace_fk
    FOREIGN KEY (workspace_id, group_id) REFERENCES budget_category_groups(workspace_id, id);
CREATE INDEX expense_categories_group_order_idx ON expense_categories(workspace_id, group_id, sort_order, id);
-- A permanently deleted starter category must not reappear on restart.
ALTER TABLE workspaces ADD COLUMN budget_categories_seeded BOOLEAN NOT NULL DEFAULT FALSE;
UPDATE workspaces SET budget_categories_seeded = TRUE WHERE EXISTS (
    SELECT 1 FROM expense_categories c WHERE c.workspace_id = workspaces.id
);
-- New closes can preserve their detailed explanation. Older closes explicitly
-- report unavailable source components rather than replaying mutable facts.
ALTER TABLE budget_close_snapshots ADD COLUMN available_to_budget_breakdown JSONB;

-- Older application versions omit group_id. Retain their insert/bootstrap
-- compatibility while the current API always validates explicit membership.
CREATE FUNCTION lukas_default_category_group() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.group_id IS NULL THEN
        INSERT INTO budget_category_groups (id, workspace_id, slug, name, sort_order)
        SELECT gen_random_uuid(), NEW.workspace_id, g.slug, g.name, g.sort_order
        FROM (VALUES ('general', 'General', 1::SMALLINT),
                     ('transport', 'Transporte', 2::SMALLINT),
                     ('savings', 'Ahorro', 3::SMALLINT)) AS g(slug,name,sort_order)
        ON CONFLICT (workspace_id,slug) DO NOTHING;
        SELECT id INTO NEW.group_id FROM budget_category_groups
        WHERE workspace_id=NEW.workspace_id AND slug=CASE
            WHEN NEW.slug='transporte' THEN 'transport'
            WHEN NEW.slug='ahorro-general' THEN 'savings'
            ELSE 'general' END;
    END IF;
    RETURN NEW;
END;
$$;
CREATE TRIGGER expense_category_default_group BEFORE INSERT ON expense_categories
FOR EACH ROW EXECUTE FUNCTION lukas_default_category_group();
