ALTER TABLE users
    ADD COLUMN first_name TEXT NOT NULL DEFAULT '',
    ADD COLUMN last_name  TEXT NOT NULL DEFAULT '',
    ADD COLUMN role       TEXT NOT NULL DEFAULT 'user';

ALTER TABLE users
    ADD CONSTRAINT users_role_check CHECK (role IN ('user', 'admin'));
