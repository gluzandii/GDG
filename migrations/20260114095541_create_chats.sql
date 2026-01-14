-- Create chat_codes table to store chat codes and their associated users
CREATE TABLE chat_codes (
    id BIGSERIAL PRIMARY KEY,
    code INTEGER NOT NULL UNIQUE CHECK (code >= 10000 AND code < 65536),
    user_id BIGSERIAL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index on code for quick lookups
CREATE INDEX idx_chat_codes_code ON chat_codes(code);

-- Index on user_id to find all chat codes created by a user
CREATE INDEX idx_chat_codes_user_id ON chat_codes(user_id);
