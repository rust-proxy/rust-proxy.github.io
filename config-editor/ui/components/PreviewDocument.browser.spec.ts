import { describe, expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import PreviewDocument from './PreviewDocument.svelte';
import type { PreviewLine } from '../types';

const lines: PreviewLine[] = [
  { text: 'title: "example"', description: '清单所属项目的名称。' },
  { text: 'items:', description: '' },
];

describe('PreviewDocument', () => {
  test('renders highlighted lines and shows per-line descriptions on hover', async () => {
    const screen = await render(PreviewDocument, { lines, format: 'yaml' });
    const first = screen.getByRole('button', { name: /清单所属项目的名称/ });
    await expect.element(first).toBeVisible();
    await expect.element(screen.getByText('title: "example"')).toBeVisible();
    await first.hover();
    await expect.element(screen.getByRole('tooltip')).toBeVisible();
    await expect.element(screen.getByRole('tooltip')).toHaveTextContent('清单所属项目的名称。');
  });
});
