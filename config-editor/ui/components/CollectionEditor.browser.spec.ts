import { describe, expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import CollectionHarness from '../testing/CollectionHarness.svelte';

describe('CollectionEditor', () => {
  test('adds a row and focuses its first field', async () => {
    const screen = await render(CollectionHarness, { schema: 'tuic-server', name: 'users' });
    await expect.element(screen.getByTestId('field-users.0.uuid')).toBeVisible();
    await screen.getByRole('button', { name: /添加用户/ }).click();
    await expect.element(screen.getByTestId('field-users.1.uuid')).toBeVisible();
    await expect.element(screen.getByTestId('field-users.1.uuid')).toHaveFocus();
  });

  test('removes a row and moves focus to the remaining row', async () => {
    const screen = await render(CollectionHarness, { schema: 'tuic-server', name: 'users' });
    await screen.getByRole('button', { name: /添加用户/ }).click();
    await expect.element(screen.getByTestId('field-users.1.uuid')).toBeVisible();
    await screen.getByRole('button', { name: '移除用户 2' }).click();
    await expect.poll(() => document.querySelectorAll('[data-row]').length).toBe(1);
    await expect.element(screen.getByTestId('field-users.0.uuid')).toHaveFocus();
  });
});
