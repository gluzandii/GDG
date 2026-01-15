-- Create conversations table to store metadata about chats between 2 users
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id_1 BIGSERIAL NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    user_id_2 BIGSERIAL NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_message_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for finding all conversations for a user
CREATE INDEX idx_conversations_user_1 ON conversations(user_id_1);
CREATE INDEX idx_conversations_user_2 ON conversations(user_id_2);

-- Unique constraint to prevent duplicate conversations between same users
CREATE UNIQUE INDEX idx_conversations_users ON conversations(user_id_1, user_id_2);