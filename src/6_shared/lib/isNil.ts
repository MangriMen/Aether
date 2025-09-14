export const isNil = <T>(
  value: null | T | undefined,
): value is null | undefined => value === null || value === undefined;
