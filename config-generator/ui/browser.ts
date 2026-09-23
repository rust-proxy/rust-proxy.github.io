export function download(text: string, filename: string) {
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

function openAncestors(element: HTMLElement) {
  let parent = element.parentElement;
  while (parent) {
    if (parent instanceof HTMLDetailsElement) parent.open = true;
    parent = parent.parentElement;
  }
}

export function focusError(key: string) {
  const element = document.getElementById(`cg-${key}`);
  if (!element) return;
  openAncestors(element);
  element.focus();
  element.scrollIntoView({ block: 'center' });
}

export function focusSection(name: string) {
  const section = document.getElementById(`cg-section-${name}`);
  if (!section) return;
  if (section instanceof HTMLDetailsElement) section.open = true;
  const heading = section.querySelector<HTMLElement>('summary, h2');
  heading?.focus({ preventScroll: true });
  section.scrollIntoView({ block: 'start' });
}
