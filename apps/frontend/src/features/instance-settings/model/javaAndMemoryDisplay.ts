export const getDisplayValue = <T>(
  overridable: boolean | undefined,
  value: T | undefined,
  isOverridden: boolean | undefined,
  defaultValue: T | undefined,
): T | undefined => {
  if (overridable && !isOverridden) {
    return defaultValue;
  }

  return value ?? defaultValue;
};
