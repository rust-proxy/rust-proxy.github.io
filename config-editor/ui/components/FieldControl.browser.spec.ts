import { describe, expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import FieldHarness from '../testing/FieldHarness.svelte';

describe('FieldControl', () => {
  test('renders a text input and propagates edits', async () => {
    const screen = await render(FieldHarness, { schema: 'tuic-client', path: 'host' });
    const input = screen.getByTestId('field-host');
    await expect.element(input).toBeVisible();
    await input.fill('tuic.example.com');
    await expect.element(input).toHaveValue('tuic.example.com');
  });

  test('renders a select with the declared options', async () => {
    const screen = await render(FieldHarness, { schema: 'tuic-server', path: 'tlsMode' });
    const select = screen.getByTestId('field-tlsMode');
    await expect.element(select).toBeVisible();
    const element = (await select.element()) as HTMLSelectElement;
    element.value = 'self';
    element.dispatchEvent(new Event('change', { bubbles: true }));
    await expect.element(select).toHaveValue('self');
  });

  test('renders a toggle as a real checkbox', async () => {
    const screen = await render(FieldHarness, { schema: 'tuic-server', path: 'dnsEnabled' });
    const toggle = screen.getByTestId('field-dnsEnabled');
    await expect.element(toggle).toBeVisible();
    await toggle.click();
    await expect.element(toggle).toBeChecked();
  });
});
