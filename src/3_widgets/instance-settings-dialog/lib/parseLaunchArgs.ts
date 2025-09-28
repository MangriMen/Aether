export const stringToLaunchArgs = (value: string): string[] => value.split(' ');

export const launchArgsToString = (
  value: string[] | undefined,
): string | undefined => value?.join(' ');
