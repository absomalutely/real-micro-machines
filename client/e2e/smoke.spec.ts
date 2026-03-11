import { test, expect } from '@playwright/test';

test('homepage loads and shows content', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('#app')).not.toBeEmpty();
});
