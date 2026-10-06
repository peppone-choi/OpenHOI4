import { expect, test } from '@playwright/test';

test('REQ-PLAT-01 built client loads without browser errors', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto('/');
  await expect(page.locator('main[data-openhoi-shell]')).toBeVisible();
  expect(errors).toEqual([]);
});
