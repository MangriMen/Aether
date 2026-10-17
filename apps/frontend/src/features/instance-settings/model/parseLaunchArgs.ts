import { quote, readQuoted } from './quoting';

const WHITESPACE = /\s/;
const NEEDS_QUOTES = /[\s"]/;

/**
 * Splits on whitespace like a shell. Text inside `"..."` keeps its spaces and the quotes are
 * dropped (`-Dx="a b"` is `-Dx=a b`); `""` inside quotes is a literal quote. A quote that is
 * never closed is a plain character, so it does not swallow the following arguments.
 * Backslashes are plain characters, so Windows paths need no escaping.
 */
export const stringToLaunchArgs = (value: string): string[] => {
  const args: string[] = [];
  let current = '';
  let hasArg = false;

  let index = 0;

  while (index < value.length) {
    const char = value[index]!;
    const quoted = char === '"' ? readQuoted(value, index) : undefined;

    if (quoted) {
      current += quoted.value;
      hasArg = true;
      index = quoted.end;
      continue;
    }

    if (WHITESPACE.test(char)) {
      if (hasArg) {
        args.push(current);
      }

      current = '';
      hasArg = false;
    } else {
      current += char;
      hasArg = true;
    }

    index++;
  }

  if (hasArg) {
    args.push(current);
  }

  return args;
};

const launchArgToString = (arg: string) =>
  arg && !NEEDS_QUOTES.test(arg) ? arg : quote(arg);

export const launchArgsToString = (
  value: string[] | undefined,
): string | undefined => value?.map(launchArgToString).join(' ');
