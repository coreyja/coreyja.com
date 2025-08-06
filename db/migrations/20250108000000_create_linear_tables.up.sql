-- Create Linear installations table for storing OAuth tokens
CREATE TABLE linear_installations (
    linear_installation_id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    external_workspace_id text NOT NULL,
    external_actor_id text,
    encrypted_access_token bytea NOT NULL,
    token_expires_at timestamp with time zone,
    scopes text[],
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    updated_at timestamp with time zone NOT NULL DEFAULT now()
);

-- Add unique constraint on workspace ID
CREATE UNIQUE INDEX linear_installations_external_workspace_id_idx ON linear_installations(external_workspace_id);