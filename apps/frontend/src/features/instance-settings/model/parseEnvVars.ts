import { EnvVarsError } from './javaAndMemoryConstants';
import { quote, readQuoted } from './quoting';

export type EnvVarsParseResult =
  | { vars: [string, string][] }
  | { error: EnvVarsError; part: string };

const SEPARATOR = /[\s;]/;
const ONLY_SEPARATORS = /^[\s;]*$/;
const ASSIGNMENT = /[A-Za-z_]\w*=/y;

const trimSeparatorsEnd = (value: string) => {
  let end = value.length;

  while (end > 0 && SEPARATOR.test(value[end - 1]!)) {
    end--;
  }

  return value.slice(0, end);
};

const isAssignmentAt = (value: string, index: number) => {
  ASSIGNMENT.lastIndex = index;

  return ASSIGNMENT.test(value);
};

/** Index of the next `NAME=` that follows whitespace or `;`, or the string length. */
const findNextAssignment = (value: string, from: number) => {
  for (let i = from; i < value.length; i++) {
    if (SEPARATOR.test(value[i]!) && isAssignmentAt(value, i + 1)) {
      return i + 1;
    }
  }

  return value.length;
};

const describeInvalidPart = (part: string): EnvVarsError => {
  if (!part.includes('=')) {
    return EnvVarsError.InvalidFormat;
  }

  return part.startsWith('=') ? EnvVarsError.EmptyKey : EnvVarsError.InvalidKey;
};

/**
 * Reads `KEY=value` variables separated by whitespace or `;`. A variable starts at a POSIX name
 * followed by `=`; any other text belongs to the previous value, so `PATH=C:/a;C:/b` and
 * `JAVA_HOME=C:\Program Files\Java` need no quotes. A value wrapped in `"..."` as a whole is
 * taken as is (`""` is a quote); quotes inside a longer value stay. Text before the first variable
 * is an error, reported with its part.
 */
export const stringToEnvVars = (value: string): EnvVarsParseResult => {
  const vars: [string, string][] = [];
  let index = 0;

  while (index < value.length) {
    while (index < value.length && SEPARATOR.test(value[index]!)) {
      index++;
    }

    if (index >= value.length) {
      break;
    }

    if (!isAssignmentAt(value, index)) {
      const part = value.slice(index).split(SEPARATOR)[0]!;

      return { error: describeInvalidPart(part), part };
    }

    const equals = value.indexOf('=', index);
    const key = value.slice(index, equals);
    const valueStart = equals + 1;

    const quoted =
      value[valueStart] === '"' ? readQuoted(value, valueStart) : undefined;

    const valueEnd = findNextAssignment(value, quoted?.end ?? valueStart);

    vars.push([
      key,
      quoted && ONLY_SEPARATORS.test(value.slice(quoted.end, valueEnd))
        ? quoted.value
        : trimSeparatorsEnd(value.slice(valueStart, valueEnd).trimStart()),
    ]);
    index = valueEnd;
  }

  return { vars };
};

const envVarToString = ([key, value]: [string, string]) => {
  const plain = `${key}=${value}`;
  const parsed = stringToEnvVars(plain);
  const readsBack =
    'vars' in parsed &&
    parsed.vars.length === 1 &&
    parsed.vars[0]![1] === value;

  return readsBack ? plain : `${key}=${quote(value)}`;
};

export const envVarsToString = (
  value: [string, string][] | undefined,
): string | undefined => value?.map(envVarToString).join('; ');
