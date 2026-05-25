CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Supports default ORDER BY created_at DESC, id DESC
CREATE INDEX idx_users_created_at_id ON users (created_at DESC, id DESC);

-- Supports ILIKE '%query%' search on email via trigram matching
CREATE INDEX idx_users_email_trgm ON users USING GIN (email gin_trgm_ops);
