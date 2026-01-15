-- Create messages table to store chat messages
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id BIGINT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    user_sent_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for fetching messages in a conversation (ordered by time)
CREATE INDEX idx_messages_conversation_sent ON messages(conversation_id, sent_at DESC);

-- Index for finding all messages sent by a user
CREATE INDEX idx_messages_user_sent ON messages(user_sent_id);

-- Trigger function to check if user is a participant
CREATE OR REPLACE FUNCTION check_message_sender()
RETURNS TRIGGER AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM conversations 
        WHERE id = NEW.conversation_id 
        AND (user_id_1 = NEW.user_sent_id OR user_id_2 = NEW.user_sent_id)
    ) THEN
        RAISE EXCEPTION 'User % is not a participant in conversation %', NEW.user_sent_id, NEW.conversation_id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to enforce participant check
CREATE TRIGGER trigger_check_message_sender
    BEFORE INSERT OR UPDATE ON messages
    FOR EACH ROW
    EXECUTE FUNCTION check_message_sender();