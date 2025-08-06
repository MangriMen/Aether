export const stringToExtraLaunchArgs = (value: string): string[] =>
  value.split(' ');

export const extraLaunchArgsToString = (
  value: string[] | undefined,
): string | undefined => value?.join(' ');
