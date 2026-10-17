import { EnvVarsError } from './javaAndMemoryConstants';

export type EnvVarsParseResult =
  { vars: [string, string][] } | { error: EnvVarsError; part: string };

/**
 * Reads `KEY=value; KEY2=value2`, splitting each variable on its first `=`. A part without `=`
 * or without a key is an error, reported with the part.
 */
export const stringToEnvVars = (value: string): EnvVarsParseResult => {
  const parts = value
    .split(';')
    .map((part) => part.trim())
    .filter(Boolean);

  const vars: [string, string][] = [];

  for (const part of parts) {
    if (!part.includes('=')) {
      return { error: EnvVarsError.InvalidFormat, part };
    }

    const [key, ...valueParts] = part.split('=');

    if (!key?.trim()) {
      return { error: EnvVarsError.EmptyKey, part };
    }

    vars.push([key.trim(), valueParts.join('=').trim()]);
  }

  return { vars };
};

export const envVarsToString = (
  value: [string, string][] | undefined,
): string | undefined => value?.map(([key, val]) => `${key}=${val}`).join('; ');
