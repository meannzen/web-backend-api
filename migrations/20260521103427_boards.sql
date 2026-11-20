CREATE TABLE boards (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL,
    code       TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Board codes are compared case-insensitively
CREATE UNIQUE INDEX boards_code_key ON boards (lower(code));

CREATE TRIGGER boards_set_updated_at
BEFORE UPDATE ON boards
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Supports default ORDER BY created_at DESC, id DESC
CREATE INDEX idx_boards_created_at_id ON boards (created_at DESC, id DESC);

-- Supports ORDER BY name, id
CREATE INDEX idx_boards_name_id ON boards (name, id);

-- Supports ILIKE '%query%' search on name and code via trigram matching
CREATE INDEX idx_boards_name_trgm ON boards USING GIN (name gin_trgm_ops);
CREATE INDEX idx_boards_code_trgm ON boards USING GIN (code gin_trgm_ops);
