export function download(text: string, filename: string): void {
  const url = URL.createObjectURL(new Blob([text], { type: 'text/plain;charset=utf-8' }));
  const anchor = document.createElement('a');
  try {
    anchor.href = url;
    anchor.download = filename;
    document.body.append(anchor);
    anchor.click();
  } finally {
    anchor.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
}

function openAncestors(element: HTMLElement): void {
  let parent = element.parentElement;
  while (parent) {
    if (parent instanceof HTMLDetailsElement) parent.open = true;
    parent = parent.parentElement;
  }
}

export function focusField(path: string): void {
  const element = document.getElementById(`ce-${path}`);
  if (!element) return;
  openAncestors(element);
  element.focus();
  element.scrollIntoView({ block: 'center' });
}

export function focusSection(name: string): void {
  const section = document.getElementById(`ce-section-${name}`);
  if (!section) return;
  if (section instanceof HTMLDetailsElement) section.open = true;
  const heading = section.querySelector<HTMLElement>('summary, h2');
  heading?.focus({ preventScroll: true });
  section.scrollIntoView({ block: 'start' });
}
