// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import mermaid from 'astro-mermaid';

export default defineConfig({
  integrations: [
    mermaid({
      enableLog: false,
    }),
    starlight({
      title: 'RustMine Docs',
      favicon: 'favicon.png',
      social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/bennetrr/rustmine' }],
    }),
  ],
});
