import type { Config as DOMPurifyConfig } from 'dompurify';
import type { Renderer, Tokens } from 'marked';

export const lazyParseMarkdown = async (text: string): Promise<string> => {
  const { marked } = await import('marked');

  const renderer = await createDefaultRenderer();

  return await marked.parse(text, { renderer });
};

export const lazySanitizeText = async (text: string): Promise<string> => {
  const DOMPurify = await import('dompurify');
  return DOMPurify.default.sanitize(text, DEFAULT_SANITIZE_CONFIG);
};

const linkStrictOpensInBrowser = ({ href, title, text }: Tokens.Link): string =>
  `<a href="${href}" title="${title || ''}" target="_blank" rel="noreferrer">${text}</a>`;

const createDefaultRenderer = async (): Promise<Renderer<string, string>> => {
  const { marked } = await import('marked');

  const renderer = new marked.Renderer<string, string>();

  renderer.link = linkStrictOpensInBrowser;

  return renderer;
};

const DEFAULT_SANITIZE_CONFIG: DOMPurifyConfig = {
  ADD_ATTR: ['target', 'rel'],
};
