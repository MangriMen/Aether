export const lazyParseAndSanitizeMarkdown = async (
  text: string,
): Promise<string> => {
  const parsed = await lazyParseMarkdown(text);
  return lazySanitizeText(parsed);
};

export const lazyParseMarkdown = async (text: string): Promise<string> => {
  const { marked } = await import('marked');
  return await marked.parse(text);
};

export const lazySanitizeText = async (text: string): Promise<string> => {
  const DOMPurify = await import('dompurify');
  return DOMPurify.default.sanitize(text);
};
