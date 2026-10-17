import type { Config as DOMPurifyConfig } from 'dompurify';
import type { Renderer, Tokens } from 'marked';

/** Hosts whose images are embedded directly; every other host goes through the wsrv.nl proxy. */
const TRUSTED_IMAGE_HOSTS: readonly string[] = [
  'cdn.modrinth.com',
  'media.forgecdn.net',
  'cdn.curseforge.com',
  'cf.way2muchnoise.eu',
  'bstats.org',
  'cdn-raw.modrinth.com',
  'raw.githubusercontent.com',
  'user-images.githubusercontent.com',
  'github.com',
  'img.shields.io',
  'i.imgur.com',
  'imgur.com',
  'i.postimg.cc',
  'wsrv.nl',
];

/** Hostname suffixes that are trusted as a whole (e.g. project pages on GitHub Pages). */
const TRUSTED_IMAGE_HOST_SUFFIXES: readonly string[] = ['.github.io'];

export const lazyParseMarkdown = async (text: string): Promise<string> => {
  const { marked } = await import('marked');
  const renderer = await createDefaultRenderer();

  return await marked.parse(text, {
    renderer,
    gfm: true,
    breaks: false,
    async: true,
  });
};

export const lazySanitizeText = async (text: string): Promise<string> => {
  const DOMPurify = await import('dompurify');
  return DOMPurify.default.sanitize(text, DEFAULT_SANITIZE_CONFIG);
};

const createDefaultRenderer = async (): Promise<Renderer<string, string>> => {
  const { marked } = await import('marked');
  const renderer = new marked.Renderer<string, string>();

  const originalLink = renderer.link.bind(renderer);

  // Safely modify links to open in the system browser (critical for Tauri)
  renderer.link = (token: Tokens.Link): string => {
    const defaultHtml = originalLink(token);
    return defaultHtml.replace('<a ', '<a target="_blank" rel="noreferrer" ');
  };

  // Custom image renderer mirrorring Modrinth proxy logic
  renderer.image = ({ href, title, text }: Tokens.Image): string => {
    let finalSrc = href;

    try {
      const url = new URL(href);

      // Proxy images via wsrv.nl if the hostname is not trusted (avoids CORS/broken assets)
      if (
        !TRUSTED_IMAGE_HOSTS.includes(url.hostname) &&
        !TRUSTED_IMAGE_HOST_SUFFIXES.some((s) => url.hostname.endsWith(s))
      ) {
        finalSrc = `https://wsrv.nl/?url=${encodeURIComponent(href)}&n=-1`;
      }
    } catch {
      // Ignore relative paths
    }

    return `<img src="${finalSrc}" alt="${text || ''}" title="${title || ''}" loading="lazy" />`;
  };

  return renderer;
};

const DEFAULT_SANITIZE_CONFIG: DOMPurifyConfig = {
  // Allow attributes for images and embedded video iframes
  ADD_ATTR: [
    'target',
    'rel',
    'src',
    'alt',
    'title',
    'width',
    'height',
    'frameborder',
    'allow',
    'allowfullscreen',
  ],
  ADD_TAGS: ['iframe', 'img', 'picture', 'source'],
};
