import { Page } from '@playwright/test';

export interface MockUser {
  login: string;
  node_id: string;
  id: number;
  email: string;
  name: string;
}

export async function setupGitHubOAuthMock(page: Page, user: MockUser) {
  // Intercept GitHub OAuth authorization
  await page.route('https://github.com/login/oauth/authorize**', async route => {
    const url = new URL(route.request().url());
    const redirectUri = url.searchParams.get('redirect_uri');
    const state = url.searchParams.get('state');
    
    // Immediately redirect back with mock code
    await route.fulfill({
      status: 302,
      headers: {
        'Location': `${redirectUri}?code=mock_code_${Date.now()}&state=${state}`
      }
    });
  });

  // Mock token exchange
  await page.route('https://github.com/login/oauth/access_token', async route => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        access_token: `mock_token_${Date.now()}`,
        expires_in: 28800,
        refresh_token: `mock_refresh_${Date.now()}`,
        refresh_token_expires_in: 15897600,
        scope: 'read:user user:email',
        token_type: 'bearer'
      })
    });
  });

  // Mock user API
  await page.route('https://api.github.com/user', async route => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(user)
    });
  });
}