-- Convert bio column to NOT NULL
-- Set empty string as default for existing NULL values
UPDATE users
SET bio = ''
WHERE bio IS NULL;

ALTER TABLE users
ALTER COLUMN bio SET NOT NULL;
