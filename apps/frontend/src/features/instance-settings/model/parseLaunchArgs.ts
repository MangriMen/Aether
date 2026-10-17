const WHITESPACE = /\s/;
const NEEDS_QUOTES = /[\s"]/;

/**
 * Splits on whitespace like a shell. Text inside `"..."` keeps its spaces and the quotes are
 * dropped (`-Dx="a b"` is `-Dx=a b`); `""` inside quotes is a literal quote. Backslashes are
 * plain characters, so Windows paths need no escaping.
 */
export const stringToLaunchArgs = (value: string): string[] => {
  const args: string[] = [];
  let current = '';
  let hasArg = false;
  let inQuotes = false;

  for (let i = 0; i < value.length; i++) {
    const char = value[i]!;

    if (char === '"') {
      if (inQuotes && value[i + 1] === '"') {
        current += '"';
        i++;
      } else {
        inQuotes = !inQuotes;
      }

      hasArg = true;
      continue;
    }

    if (!inQuotes && WHITESPACE.test(char)) {
      if (hasArg) {
        args.push(current);
      }

      current = '';
      hasArg = false;
      continue;
    }

    current += char;
    hasArg = true;
  }

  if (hasArg) {
    args.push(current);
  }

  return args;
};

const launchArgToString = (arg: string) =>
  arg && !NEEDS_QUOTES.test(arg) ? arg : `"${arg.replaceAll('"', '""')}"`;

export const launchArgsToString = (
  value: string[] | undefined,
): string | undefined => value?.map(launchArgToString).join(' ');
