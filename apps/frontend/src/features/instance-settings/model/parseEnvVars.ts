export const stringToEnvVars = (value: string): [string, string][] =>
  value
    .split(';')
    .filter((variable) => variable.includes('=') && variable.trim() !== '=')
    .map((variable) => {
      const [key, ...valueParts] = variable.trim().split('=');
      return [key, valueParts.join('=')] as [string, string];
    });

export const envVarsToString = (
  value: [string, string][] | undefined,
): string | undefined => value?.map(([key, val]) => `${key}=${val}`).join('; ');
