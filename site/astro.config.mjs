// @ts-check
import { defineConfig } from 'astro/config'
import starlight from '@astrojs/starlight'
import starlightLinksValidator from 'starlight-links-validator'

// The organisation page repository serves at the apex, so there is no base path and no `base`
// option — which is the whole reason the site lives in `vitui-rs/vitui-rs.github.io` rather than
// being a project page of the workspace repository. See `site/README.md`.
export default defineConfig({
  site: 'https://vitui-rs.github.io',
  integrations: [
    starlight({
      title: 'vitui',
      customCss: ['./src/styles/site.css'],
      description:
        'A Rust TUI library: layered terminal rendering with damage tracking, built so that frame cost is proportional to visible cells, never to data volume.',
      social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/vitui-rs/vitui' }],
      editLink: { baseUrl: 'https://github.com/vitui-rs/vitui/edit/master/' },
      lastUpdated: true,
      // A staged page's edit link would point into a build artefact, and the plugin below cannot
      // know that. The three staged trees say where their source lives on the page itself.
      sidebar: [
        {
          label: 'Start here',
          items: [
            { label: 'Getting started', slug: 'start' },
            { label: 'What it looks like', slug: 'showcase' },
            { label: 'Components', slug: 'components' },
          ],
        },
        {
          label: 'Building components',
          items: [{ autogenerate: { directory: 'guide' } }],
        },
        {
          label: 'Evidence',
          items: [
            { label: 'Benchmarks', slug: 'benchmarks' },
            { label: 'Terminals', slug: 'terminals' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'Glossary', slug: 'reference/glossary' },
            { label: 'Architecture specs', collapsed: true, items: [{ autogenerate: { directory: 'spec' } }] },
            { label: 'Decision records', collapsed: true, items: [{ label: 'All 53, by number', slug: 'adr' }] },
          ],
        },
      ],
      plugins: [
        // A broken internal link fails the build. Every page on this site is derived from something
        // — a chapter, a record, a spec, or a value in the crate — so a link that stops resolving is
        // a rename that has already happened, and the only cheap moment to hear about it is here.
        starlightLinksValidator({ errorOnRelativeLinks: false }),
      ],
    }),
  ],
})
