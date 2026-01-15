-- Add migration script here
-- Create a function that sends a notification when a message is inserted
CREATE OR REPLACE FUNCTION notify_message_insert()
RETURNS TRIGGER AS $$
DECLARE
    notification json;
BEGIN
    -- Build the notification payload
    notification = json_build_object(
        'user_id', NEW.user_sent_id,
        'content', NEW.content,
        'sent_at', NEW.sent_at
    );
    
    -- Send notification to channel named after the conversation_id
    PERFORM pg_notify(
        'conversation_' || NEW.conversation_id::text,
        notification::text
    );
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create a trigger that fires after each message insert
CREATE TRIGGER message_insert_trigger
    AFTER INSERT ON messages
    FOR EACH ROW
    EXECUTE FUNCTION notify_message_insert();
