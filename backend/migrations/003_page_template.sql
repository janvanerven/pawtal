ALTER TABLE pages ADD COLUMN template TEXT NOT NULL DEFAULT 'default' CHECK (template IN ('default', 'project'));
