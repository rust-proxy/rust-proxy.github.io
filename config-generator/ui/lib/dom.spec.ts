import { beforeEach, describe, expect, it, vi } from 'vitest';
import { download, focusField, focusSection } from './dom';

beforeEach(() => {
  document.body.innerHTML = '';
  Element.prototype.scrollIntoView = vi.fn();
  Object.defineProperty(URL, 'createObjectURL', { value: vi.fn(() => 'blob:mock'), configurable: true });
  Object.defineProperty(URL, 'revokeObjectURL', { value: vi.fn(), configurable: true });
});

describe('download', () => {
  it('clicks a temporary anchor and removes it', () => {
    const click = vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {});
    download('hello', 'a.json');
    expect(URL.createObjectURL).toHaveBeenCalled();
    expect(click).toHaveBeenCalled();
    expect(document.querySelector('a')).toBeNull();
    click.mockRestore();
  });
});

describe('focusField', () => {
  it('opens collapsed ancestors and focuses the field', () => {
    document.body.innerHTML = '<details id="cg-section-s"><summary>sec</summary><input id="cg-host"></details>';
    const input = document.getElementById('cg-host') as HTMLInputElement;
    focusField('host');
    expect((document.getElementById('cg-section-s') as HTMLDetailsElement).open).toBe(true);
    expect(document.activeElement).toBe(input);
    expect(input.scrollIntoView).toHaveBeenCalled();
  });

  it('is a no-op for unknown fields', () => {
    expect(() => focusField('missing')).not.toThrow();
  });
});

describe('focusSection', () => {
  it('opens the section and focuses its heading', () => {
    document.body.innerHTML = '<details id="cg-section-s"><summary>sec</summary></details>';
    const summary = document.querySelector('summary') as HTMLElement;
    focusSection('s');
    expect((document.getElementById('cg-section-s') as HTMLDetailsElement).open).toBe(true);
    expect(document.activeElement).toBe(summary);
  });
});
