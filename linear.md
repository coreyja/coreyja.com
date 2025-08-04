# Linear Agent Integration Plan

## Overview

This document outlines the plan for integrating Linear's API as an AI agent using OAuth authentication. The goal of this phase is to establish authentication and set up the foundation for future Linear API interactions.

## Integration Goals

- Set up OAuth 2.0 authentication flow for Linear
- Configure the application as an AI agent (using `actor=app` parameter)
- Enable webhook subscriptions for agent session events
- Establish secure API access for future feature development

## Prerequisites

1. **Linear Workspace Admin Access**: Required to create OAuth applications and manage webhooks
2. **Public URL**: For OAuth redirect URI and webhook endpoints
3. **Server Infrastructure**: To handle OAuth flow and webhook events

## OAuth Application Setup

### 1. Create OAuth Application [Corey will do this in the UI, assume its done for now]

- Navigate to Linear Settings → Account → API
- Scroll to "OAuth Applications" section
- Click "Create new OAuth Application"
- Configure application with:
  - Application name: `coreyja.com AI Agent`
  - Description: AI agent for automating Linear workflows
  - Redirect URI: `https://coreyja.com/api/linear/callback`
  - Webhook URL: `https://coreyja.com/api/linear/webhooks`

### 2. Required OAuth Scopes

- `read`: Read access to Linear data
- `write`: Write access to create/update issues
- `admin`: Administrative operations
- `app:assignable`: Allow agent to be assigned to projects/issues
- `app:mentionable`: Allow agent to be mentioned in comments/documents
- `issue:create`: Create new issues
- `comment:create`: Create comments on issues

### 3. Environment Variables [Corey will do this in the UI, assume its done for now and the env vars exist locally and in prod]

```env
LINEAR_CLIENT_ID=<from OAuth app creation>
LINEAR_CLIENT_SECRET=<from OAuth app creation>
LINEAR_REDIRECT_URI=https://coreyja.com/api/linear/callback
LINEAR_WEBHOOK_SECRET=<generated webhook signing secret>
```

## Authentication Flow Implementation

### 1. Authorization Endpoint

- We want a button in the Admin interface that will trigger the OAuth flow to add the Agent to the workspace.

```
GET https://linear.app/oauth/authorize
Parameters:
- client_id: ${LINEAR_CLIENT_ID}
- redirect_uri: ${LINEAR_REDIRECT_URI}
- response_type: code
- scope: read,write,admin,app:assignable,app:mentionable,issue:create,comment:create
- state: ${RANDOM_UUID} (random UUID for state tracking)
- actor: app (IMPORTANT: This makes it an agent authentication)
```

### 2. Token Exchange

```
POST https://api.linear.app/oauth/token
Body:
- client_id: ${LINEAR_CLIENT_ID}
- client_secret: ${LINEAR_CLIENT_SECRET}
- redirect_uri: ${LINEAR_REDIRECT_URI}
- grant_type: authorization_code
- code: ${AUTHORIZATION_CODE}
- actor: app
```

### 3. Token Storage

- Store access tokens securely (encrypted in the database)
  - Use existing encryption methods
- Store refresh tokens for token renewal
- Track token expiration for automatic refresh
- Token refresh will be handled automatically when making API calls (future phase)

## Webhook Configuration

### 1. Enable Webhook Events

During OAuth app creation, enable webhooks for:

- **Agent session events** (required for agent functionality)
- Issue events (create, update, delete)
- Comment events
- Project events
- Document events

### 2. Webhook Handler Requirements

- Must respond within 5 seconds
- First response to "created" event must be within 10 seconds
- Should immediately acknowledge work with "thought" activity
- Complete work within 30 minutes before session becomes stale

### 3. Agent Activity Types

Implement handlers for emitting:

- `thought`: Initial acknowledgment and planning
- `action`: Actual operations being performed
- `response`: Final results/responses
- `error`: Error states and failures

## API Endpoints to Implement

### 1. OAuth Flow

- `GET /api/linear/auth` - Initiate OAuth flow
- `GET /api/linear/callback` - Handle OAuth callback

### 2. Webhook Handler

- `POST /api/linear/webhooks` - Handle all Linear webhook events
- Verify webhook signatures using `LINEAR_WEBHOOK_SECRET`
- Route events to appropriate handlers

## Database Schema

### 1. Linear Installations

```sql
CREATE TABLE linear_installations (
  linear_installation_id UUID PRIMARY KEY,
  external_workspace_id VARCHAR(255) NOT NULL,
  encrypted_access_token TEXT NOT NULL,
  encrypted_refresh_token TEXT,
  token_expires_at TIMESTAMP,
  scopes TEXT[],
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);
```

### 2. Webhook Events

```sql
CREATE TABLE linear_webhook_events (
  linear_webhook_event_id UUID PRIMARY KEY,
  event_type VARCHAR(255) NOT NULL,
  payload JSONB NOT NULL,
  processed_at TIMESTAMP,
  created_at TIMESTAMP DEFAULT NOW()
);
```

## Security Considerations

1. **Token Security**

   - Encrypt tokens at rest
   - Use environment variables for secrets
   - Never expose tokens in logs or error messages

2. **Webhook Verification**

   - Verify all webhook signatures (confirm HMAC-SHA256 algorithm from Linear docs)

3. **Error Handling**
   - Graceful error handling for API failures
   - Proper error logging without exposing sensitive data
   - Use `?` operator to propagate errors to standard error reporting

## Implementation Phases

### Phase 1: Basic OAuth Setup (Current)

- [ ] Create OAuth application in Linear
- [ ] Implement OAuth flow endpoints with `actor=app`
- [ ] Set up token storage
- [ ] Basic webhook handler

### Phase 2: Agent Configuration

- [ ] Implement agent activity handlers
- [ ] Set up webhook event processing
- [ ] Test agent interactions

## Resources

- [Linear API Documentation](https://developers.linear.app)
- [Linear Agent Documentation](https://linear.app/developers/agents)
- [Linear OAuth Documentation](https://linear.app/developers/oauth-2-0-authentication)

## Notes

- Agent installations do not count as billable users in Linear
- Agents behave like regular users but are clearly marked as automated
- Use existing GraphQL client (add Linear schema)
- Local webhook testing will use ngrok
- No CSRF token needed for OAuth state parameter
