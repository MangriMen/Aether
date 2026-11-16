import { readQuoted } from './quoting';

const WHITESPACE = /\s/;

type CommandLineError = 'EMPTY' | 'UNCLOSED_QUOTE';

export type CommandLineResult =
  | { program: string; args: string[] }
  | { error: CommandLineError };

/**
 * Reads a hook command line like the core does: words split on whitespace, `"..."` keeps spaces
 * and drops the quotes, `""` inside quotes is a literal quote, backslashes and single quotes are
 * plain characters. Unlike the launch arguments, a quote that is never closed is an error.
 */
export const parseCommandLine = (value: string): CommandLineResult => {
  const words: string[] = [];
  let current = '';
  let hasWord = false;
  let index = 0;

  while (index < value.length) {
    const char = value[index]!;

    if (char === '"') {
      const quoted = readQuoted(value, index);

      if (!quoted) {
        return { error: 'UNCLOSED_QUOTE' };
      }

      current += quoted.value;
      hasWord = true;
      index = quoted.end;
      continue;
    }

    if (WHITESPACE.test(char)) {
      if (hasWord) {
        words.push(current);
      }

      current = '';
      hasWord = false;
    } else {
      current += char;
      hasWord = true;
    }

    index++;
  }

  if (hasWord) {
    words.push(current);
  }

  const [program, ...args] = words;

  return program ? { program, args } : { error: 'EMPTY' };
};

/** The edges are never part of a command, and the core trims them too. */
export const normalizeHookCommand = (value: string): string => value.trim();

export const hasUnclosedQuote = (value: string): boolean => {
  const result = parseCommandLine(value);

  return 'error' in result && result.error === 'UNCLOSED_QUOTE';
};
