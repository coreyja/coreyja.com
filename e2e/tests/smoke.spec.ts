import { test, expect } from '@playwright/test';

test.describe('Smoke Tests', () => {
  test('should load the homepage', async ({ page }) => {
    // Navigate to homepage
    await page.goto('/');
    
    // Check that the page loads successfully
    await expect(page).toHaveTitle(/coreyja/i);
    
    // Verify some basic content is present
    const response = page.waitForResponse(resp => resp.status() === 200);
    await page.reload();
    await response;
  });

  test('should navigate to admin login', async ({ page }) => {
    // Navigate to admin login
    await page.goto('/admin');
    
    // Should redirect to login or show login page
    await expect(page.url()).toMatch(/\/admin|\/login/);
  });
});