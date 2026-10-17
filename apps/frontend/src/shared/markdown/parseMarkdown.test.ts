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
