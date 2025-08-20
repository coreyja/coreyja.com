-- Create LinearOauthStates table for Linear OAuth flow
CREATE TABLE LinearOauthStates (
    linear_oauth_state_id UUID PRIMARY KEY,
    state TEXT NOT NULL,
    return_to TEXT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Add index for looking up states
CREATE INDEX idx_linear_oauth_states_created_at ON LinearOauthStates(created_at);