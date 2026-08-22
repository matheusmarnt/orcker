import { withMermaid } from 'vitepress-plugin-mermaid'
import llmstxt from 'vitepress-plugin-llms'

// Shared SEO strings (reused for the meta description, social cards, and the
// per-page tags injected by `transformPageData`).
const SITE_TITLE = 'Orcker - Local PHP development environment for macOS & Linux'
const SITE_DESCRIPTION =
  'A fast, rootless, open-source local PHP development environment for macOS and Linux. Serve .test sites over HTTP and HTTPS, run a different PHP version per site, and manage databases, mail, and tooling from one tiny daemon - a Laravel Herd alternative.'
const OG_IMAGE = 'https://orcker.app/images/social-card.png'

// Site config for the Orcker documentation (https://orcker.app).
// Run with `npm run dev` from the `docs/` directory.
// Wrapped with withMermaid so ```mermaid code blocks render as responsive SVG.
export default withMermaid({
  title: 'Orcker',
  description: SITE_DESCRIPTION,
  lang: 'en-US',

  // Canonical host for the generated sitemap and absolute URLs.
  sitemap: {
    hostname: 'https://orcker.app',
  },

  // Clean URLs (`/guide/getting-started` instead of `.html`).
  cleanUrls: true,
  lastUpdated: true,

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],
    ['meta', { name: 'theme-color', content: '#6366f1' }],
    ['meta', { name: 'author', content: 'Forjed' }],
    [
      'meta',
      {
        name: 'keywords',
        content:
          'PHP, local development, Laravel, Herd alternative, .test domains, HTTPS, macOS, Linux, rootless, open source, PHP versions',
      },
    ],
    // Open Graph + Twitter — the per-page title/description/url are injected by
    // `transformPageData` below; these are the static, page-independent bits.
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:site_name', content: 'Orcker' }],
    ['meta', { property: 'og:locale', content: 'en_US' }],
    ['meta', { property: 'og:image', content: OG_IMAGE }],
    ['meta', { property: 'og:image:width', content: '1200' }],
    ['meta', { property: 'og:image:height', content: '630' }],
    [
      'meta',
      {
        property: 'og:image:alt',
        content: 'Orcker - local PHP dev for macOS and Linux',
      },
    ],
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }],
    ['meta', { name: 'twitter:image', content: OG_IMAGE }],
    // Structured data: marks Orcker as a free developer application for rich results.
    [
      'script',
      { type: 'application/ld+json' },
      JSON.stringify({
        '@context': 'https://schema.org',
        '@type': 'SoftwareApplication',
        name: 'Orcker',
        applicationCategory: 'DeveloperApplication',
        operatingSystem: 'macOS, Linux',
        description: SITE_DESCRIPTION,
        url: 'https://orcker.app',
        image: OG_IMAGE,
        license: 'https://opensource.org/licenses/MIT',
        isAccessibleForFree: true,
        offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
      }),
    ],
  ],

  // Per-page SEO: a canonical URL plus page-specific og:/twitter: title and
  // description, so inner pages aren't all tagged with the homepage's metadata.
  transformPageData(pageData) {
    const isHome = pageData.relativePath === 'index.md'
    const path = pageData.relativePath
      .replace(/(^|\/)index\.md$/, '$1')
      .replace(/\.md$/, '')
    const canonical = `https://orcker.app/${path}`
    const title = isHome ? SITE_TITLE : `${pageData.title} | Orcker`
    const description = pageData.description || SITE_DESCRIPTION
    pageData.frontmatter.head ??= []
    pageData.frontmatter.head.push(
      ['link', { rel: 'canonical', href: canonical }],
      ['meta', { property: 'og:title', content: title }],
      ['meta', { property: 'og:description', content: description }],
      ['meta', { property: 'og:url', content: canonical }],
      ['meta', { name: 'twitter:title', content: title }],
      ['meta', { name: 'twitter:description', content: description }],
    )
  },

  // Render node labels as HTML so long text wraps instead of being clipped,
  // and scale diagrams to the container width (good on small screens).
  mermaid: {
    securityLevel: 'loose',
    flowchart: { htmlLabels: true, useMaxWidth: true },
  },

  // Emits llms.txt / llms-full.txt and a clean .md copy of every page
  // (https://llmstxt.org) so agents can consume the docs without HTML noise.
  vite: {
    plugins: [
      llmstxt({
        domain: 'https://orcker.app',
        title: SITE_TITLE,
        description: SITE_DESCRIPTION,
      }),
    ],
  },

  themeConfig: {
    logo: '/logo.svg',

    nav: [
      { text: 'Guide', link: '/guide/introduction', activeMatch: '/guide/' },
      { text: 'CLI Reference', link: '/reference/cli/', activeMatch: '/reference/' },
      {
        text: 'Developer',
        link: '/developer/architecture',
        activeMatch: '/developer/',
      },
      {
        text: 'v2.x',
        items: [
          {
            text: 'Changelog',
            link: 'https://github.com/forjedio/orcker/releases',
          },
          { text: 'Contributing', link: '/developer/contributing' },
        ],
      },
    ],

    sidebar: {
      '/guide/': [
        {
          text: 'Introduction',
          items: [
            { text: 'What is Orcker?', link: '/guide/introduction' },
            { text: 'Getting Started', link: '/guide/getting-started' },
            { text: 'Guides', link: '/guide/guides' },
            { text: 'Switching to Orcker', link: '/guide/switching-to-orcker' },
            { text: 'Upgrade Guide', link: '/guide/upgrading-from-v1' },
            { text: 'Features', link: '/guide/desktop-app' },
          ],
        },
        {
          text: 'Using Orcker',
          items: [
            { text: 'Sites', link: '/guide/sites' },
            { text: 'Reverse Proxies', link: '/guide/proxies' },
            { text: 'PHP Versions', link: '/guide/php-versions' },
            { text: 'Code Coverage', link: '/guide/code-coverage' },
            { text: 'Tooling', link: '/guide/tooling' },
            { text: 'Services & Databases', link: '/guide/services' },
            { text: 'Mail Capture', link: '/guide/mail' },
            { text: 'Laravel Dumps', link: '/guide/laravel-dumps' },
            { text: 'AI Agents', link: '/guide/ai-agents' },
            { text: 'Sharing Sites', link: '/guide/sharing' },
            { text: 'HTTPS & Certificates', link: '/guide/https' },
            { text: 'DNS & .test Domains', link: '/guide/dns' },
            { text: 'Elevation & Privileges', link: '/guide/elevation' },
            { text: 'Localhost Access', link: '/guide/localhost-access' },
            { text: 'The Daemon', link: '/guide/daemon' },
            { text: 'Diagnostics', link: '/guide/diagnostics' },
          ],
        },
      ],

      '/reference/': [
        {
          text: 'CLI Reference',
          items: [
            { text: 'Overview', link: '/reference/cli/' },
            { text: 'Sites', link: '/reference/cli/sites' },
            { text: 'Domains', link: '/reference/cli/domains' },
            { text: 'Proxies', link: '/reference/cli/proxies' },
            { text: 'Routing rules', link: '/reference/cli/routes' },
            { text: 'HTTPS', link: '/reference/cli/https' },
            { text: 'PHP', link: '/reference/cli/php' },
            { text: 'Exec and Which', link: '/reference/cli/exec' },
            { text: 'Coverage', link: '/reference/cli/coverage' },
            { text: 'Tooling', link: '/reference/cli/tooling' },
            { text: 'Services', link: '/reference/cli/services' },
            { text: 'Databases', link: '/reference/cli/db' },
            { text: 'Mail', link: '/reference/cli/mail' },
            { text: 'MCP', link: '/reference/cli/mcp' },
            { text: 'LAN sharing', link: '/reference/cli/lan' },
            { text: 'Tunnel', link: '/reference/cli/tunnel' },
            { text: 'Diagnostics', link: '/reference/cli/diagnostics' },
            { text: 'Elevation', link: '/reference/cli/elevation' },
            { text: 'Daemon control', link: '/reference/cli/daemon' },
            { text: 'Self-Update', link: '/reference/cli/update' },
            { text: 'Uninstall', link: '/reference/cli/uninstall' },
          ],
        },
        {
          text: 'Configuration',
          items: [
            { text: 'Config file', link: '/reference/configuration' },
          ],
        },
      ],

      '/developer/': [
        {
          text: 'Overview',
          items: [
            { text: 'Architecture', link: '/developer/architecture' },
            { text: 'Crates Overview', link: '/developer/crates' },
            { text: 'Building from Source', link: '/developer/building' },
            { text: 'Contributing', link: '/developer/contributing' },
          ],
        },
        {
          text: 'Internals',
          items: [
            { text: 'IPC Protocol', link: '/developer/ipc-protocol' },
            { text: 'Dev-Tool Installers', link: '/developer/dev-tools' },
            { text: 'Cross-Platform Model', link: '/developer/cross-platform' },
            { text: 'Config Schema History', link: '/developer/config-schema-history' },
          ],
        },
        {
          text: 'Crates',
          collapsed: false,
          items: [
            { text: 'orcker-core', link: '/developer/crates/orcker-core' },
            { text: 'orcker-ipc', link: '/developer/crates/orcker-ipc' },
            { text: 'orcker-config', link: '/developer/crates/orcker-config' },
            { text: 'orcker-tls', link: '/developer/crates/orcker-tls' },
            { text: 'orcker-dns', link: '/developer/crates/orcker-dns' },
            { text: 'orcker-proxy', link: '/developer/crates/orcker-proxy' },
            { text: 'orcker-supervise', link: '/developer/crates/orcker-supervise' },
            { text: 'orcker-php', link: '/developer/crates/orcker-php' },
            { text: 'orcker-mail', link: '/developer/crates/orcker-mail' },
            { text: 'orcker-mcp', link: '/developer/crates/orcker-mcp' },
            { text: 'orcker-tunnel', link: '/developer/crates/orcker-tunnel' },
            { text: 'orcker-services', link: '/developer/crates/orcker-services' },
            { text: 'orcker-doctor', link: '/developer/crates/orcker-doctor' },
            { text: 'orcker-platform', link: '/developer/crates/orcker-platform' },
            { text: 'orcker-service-ctl', link: '/developer/crates/orcker-service-ctl' },
            { text: 'orcker-update', link: '/developer/crates/orcker-update' },
            { text: 'orcker-depcheck (test-only)', link: '/developer/crates/orcker-depcheck' },
          ],
        },
        {
          text: 'Binaries',
          collapsed: false,
          items: [
            { text: 'orckerd (daemon)', link: '/developer/binaries/orckerd' },
            { text: 'orcker (CLI)', link: '/developer/binaries/orcker' },
            {
              text: 'orcker-helper (privileged)',
              link: '/developer/binaries/orcker-helper',
            },
          ],
        },
        {
          text: 'App & Tooling',
          items: [
            { text: 'Desktop App Internals', link: '/developer/gui' },
            { text: 'Build Automation (xtask)', link: '/developer/xtask' },
          ],
        },
      ],
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/forjedio/orcker' },
    ],

    editLink: {
      pattern: 'https://github.com/forjedio/orcker/edit/main/docs/:path',
      text: 'Edit this page on GitHub',
    },

    search: {
      provider: 'local',
    },

    footer: {
      message:
        'A <a href="https://forjed.io" target="_blank" rel="noopener">Forjed</a> project. Released under the MIT License.',
      copyright: 'Copyright © 2026 Orcker',
    },
  },
})
