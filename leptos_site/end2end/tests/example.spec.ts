import { test, expect } from "@playwright/test";

test("homepage has title", async ({ page }) => {
  await page.goto("http://localhost:3000/");

  await expect(page).toHaveTitle("Welcome to Leptos");

  await expect(page.locator("h1")).toHaveText("Welcome to Leptos!");
});

test("homepage mentions my username", async ({ page }) => {
  await page.goto("http://localhost:3000/");

  await expect(page).toHaveTitle("Welcome to Leptos");

  await expect(page.locator("html")).toHaveText(/coreyja/);
});
