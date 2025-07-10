export const stringToEnvVars = (value: string): [string, string][] =>
  value
    .split(' ')
    .map((variable) => variable.split('=', 2) as [string, string]);

export const envVarsToString = (value: [string, string][]): string =>
  value.map(([key, val]) => `${key}=${val}`).join(' ');
