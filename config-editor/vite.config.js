import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [
    svelte(),
    paraglideVitePlugin({
      project: './project.inlang',
      outdir: './ui/paraglide',
      emitTsDeclarations: true,
    }),
  ],
  base: './',
  build: { assetsInlineLimit: 0 },
});
