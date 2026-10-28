import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

import { lazyParseMarkdown } from './parseMarkdown';

const imgSrc = (html: string) => /<img[^>]*\ssrc="([^"]*)"/.exec(html)?.[1];

describe('lazyParseMarkdown image hosts', () => {
  it.each([
    'https://cdn.modrinth.com/data/a.png',
    'https://media.forgecdn.net/a.png',
    'https://i.imgur.com/a.png',
    'https://img.shields.io/badge/a-b-green',
    'https://user.github.io/a.png',
    'https://cf.way2muchnoise.eu/a.png',
    'https://bstats.org/a.png',
    'https://cdn-raw.modrinth.com/a.png',
  ])('keeps trusted host %s direct', async (src) => {
    expect(imgSrc(await lazyParseMarkdown(`![a](${src})`))).toBe(src);
  });

  it.each([
    'https://example.com/a.png',
    'https://evilgithub.io/a.png',
    'https://github.io.evil.com/a.png',
    'https://notgithub.com/a.png',
  ])('proxies untrusted host %s through wsrv.nl', async (src) => {
    expect(imgSrc(await lazyParseMarkdown(`![a](${src})`))).toBe(
      `https://wsrv.nl/?url=${encodeURIComponent(src)}&n=-1`,
    );
  });
});

describe('lazyParseMarkdown image hosts against the app CSP', () => {
  const imgSources = (): string[] => {
    const config = JSON.parse(
      readFileSync(
        resolve(__dirname, '../../../../desktop/tauri.conf.json'),
        'utf8',
      ),
    ) as { app: { security: { csp: string } } };
    const directive = config.app.security.csp
      .split(';')
      .map((part) => part.trim())
      .find((part) => part.startsWith('img-src '));
    return directive?.split(/\s+/).slice(1) ?? [];
  };

  const allows = (sources: string[], src: string): boolean => {
    const { origin } = new URL(src);
    return sources.some((source) =>
      source.includes('*')
        ? new RegExp(
            `^${source.replaceAll('.', '\\.').replace('*', '[^.]+')}$`,
          ).test(origin)
        : source === origin,
    );
  };

  it('should not allow every https host', () => {
    expect(imgSources()).not.toContain('https://*');
  });

  it.each([
    'https://cdn.modrinth.com/data/a.png',
    'https://media.forgecdn.net/a.png',
    'https://i.imgur.com/a.png',
    'https://img.shields.io/badge/a-b-green',
    'https://user.github.io/a.png',
    'https://cf.way2muchnoise.eu/a.png',
    'https://bstats.org/a.png',
    'https://cdn-raw.modrinth.com/a.png',
    'https://example.com/a.png',
  ])('should allow the image that %s renders to', async (src) => {
    const rendered = imgSrc(await lazyParseMarkdown(`![a](${src})`));
    expect(allows(imgSources(), rendered ?? '')).toBe(true);
  });

  it('should not allow an arbitrary host', () => {
    expect(allows(imgSources(), 'https://tracker.example.com/p.png')).toBe(
      false,
    );
  });
});
