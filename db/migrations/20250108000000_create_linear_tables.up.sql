-- Create Linear installations table for storing OAuth tokens
CREATE TABLE linear_installations (
    linear_installation_id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    external_workspace_id text NOT NULL,
    encrypted_access_token bytea NOT NULL,
    encrypted_refresh_token bytea,
    token_expires_at timestamp with time zone,
    scopes text[],
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    updated_at timestamp with time zone NOT NULL DEFAULT now()
);

-- Add unique constraint on workspace ID
CREATE UNIQUE INDEX linear_installations_external_workspace_id_idx ON linear_installations(external_workspace_id);

-- Create Linear webhook events table for tracking webhook events
CREATE TABLE linear_webhook_events (
    linear_webhook_event_id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    event_type text NOT NULL,
    payload jsonb NOT NULL,
    processed_at timestamp with time zone,
    created_at timestamp with time zone NOT NULL DEFAULT now()
);

-- Add index for unprocessed events
CREATE INDEX linear_webhook_events_unprocessed_idx ON linear_webhook_events(created_at) WHERE processed_at IS NULL;