-- Restore webhook_url column to discord_thread_metadata table
ALTER TABLE discord_thread_metadata
ADD COLUMN webhook_url TEXT;