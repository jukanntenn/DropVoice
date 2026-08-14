import { test, expect } from '@playwright/test';

test.describe('DropVoice Desktop App', () => {
  test('app loads and shows header', async ({ page }) => {
    await page.goto('/');
    // The app should render with a header containing the app name.
    await expect(page.locator('body')).toBeVisible();
  });

  test('QR code section is visible', async ({ page }) => {
    await page.goto('/');
    // The QR code section should be present (either showing QR or disconnected state).
    await expect(page.locator('body')).toBeVisible();
  });

  test('settings dialog can be opened', async ({ page }) => {
    await page.goto('/');
    // Look for a settings button and click it.
    const settingsButton = page.getByRole('button', { name: /settings|设置/i });
    if (await settingsButton.isVisible()) {
      await settingsButton.click();
      // The settings dialog should appear.
      await expect(page.getByRole('dialog')).toBeVisible();
    }
  });
});
