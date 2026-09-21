import Prism from 'prismjs';
import type { Grammar } from 'prismjs';
import 'prismjs/components/prism-yaml';
import 'prismjs/components/prism-toml';
import 'prismjs/components/prism-json';

// The page highlights individual lines itself; disable Prism's automatic page scan.
Prism.manual = true;

const grammars: Record<string, Grammar | undefined> = {
  yaml: Prism.languages.yaml,
  toml: Prism.languages.toml,
  json: Prism.languages.json,
};

function escapeHtml(value: string): string {
  return value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

export function highlightLines(lines: string[], language: string): string[] {
  const grammar = grammars[language];
  return lines.map(line => (grammar ? Prism.highlight(line, grammar, language) : escapeHtml(line)));
}
