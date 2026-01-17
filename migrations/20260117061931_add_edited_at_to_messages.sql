-- Add edited_at column to messages table
ALTER TABLE messages
ADD COLUMN edited_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP;

-- Update existing rows to set edited_at to sent_at
UPDATE messages
SET edited_at = sent_at;

-- Make edited_at match the time zone handling of sent_at
ALTER TABLE messages
ALTER COLUMN edited_at SET NOT NULL;
