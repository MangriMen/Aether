import { EnvVarsError } from './javaAndMemoryConstants';

export type EnvVarsParseResult =
  { vars: [string, string][] } | { error: EnvVarsError; part: string };

const SEPARATOR = /[\s;]/;
const LEADING_SEPARATORS = /^[\s;]+/;
const ONLY_SEPARATORS = /^[\s;]*$/;
const ASSIGNMENT = /^[A-Za-z_]\w*=/;

const trimSeparatorsEnd = (value: string) => {
  let end = value.length;

  while (end > 0 && SEPARATOR.test(value[end - 1]!)) {
    end--;
  }

  return value.slice(0, end);
};

const isAssignmentAt = (value: string, index: number) =>
  ASSIGNMENT.test(value.slice(index));

/** Index of the next `NAME=` that follows whitespace or `;`, or the string length. */
const findNextAssignment = (value: string, from: number) => {
  for (let i = from; i < value.length; i++) {
    if (SEPARATOR.test(value[i]!) && isAssignmentAt(value, i + 1)) {
      return i + 1;
    }
  }

  return value.length;
};

/** Reads `"..."` at `start`, where `""` is a literal quote. Undefined unless a separator or the end follows the closing quote. */
const readQuoted = (value: string, start: number) => {
  let result = '';

  for (let i = start + 1; i < value.length; i++) {
    if (value[i] !== '"') {
      result += value[i];
      continue;
    }

    if (value[i + 1] === '"') {
      result += '"';
      i++;
      continue;
    }

    const next = value[i + 1];

    return next === undefined || SEPARATOR.test(next)
      ? { value: result, end: i + 1 }
      : undefined;
  }

  return undefined;
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
    index += LEADING_SEPARATORS.exec(value.slice(index))?.[0].length ?? 0;

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

    if (quoted) {
      const quotedEnd = findNextAssignment(value, quoted.end);

      if (ONLY_SEPARATORS.test(value.slice(quoted.end, quotedEnd))) {
        vars.push([key, quoted.value]);
        index = quotedEnd;
        continue;
      }
    }

    const valueEnd = findNextAssignment(value, quoted?.end ?? valueStart);

    vars.push([
      key,
      trimSeparatorsEnd(value.slice(valueStart, valueEnd).trimStart()),
    ]);
    index = valueEnd;
  }

  return { vars };
};

const quote = (value: string) => `"${value.replaceAll('"', '""')}"`;

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
