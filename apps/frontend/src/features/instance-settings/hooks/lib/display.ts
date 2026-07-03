import type { HookFieldKey, HooksSettingsSchemaInput } from '../model';

export const getHookDisplayValue = (
  defaultValues:
    (() => Partial<HooksSettingsSchemaInput> | undefined) | undefined,
  overridable: boolean | undefined,
  key: HookFieldKey,
  value: string | undefined,
  isOverridden: boolean | undefined,
): string => {
  const defaultValue = defaultValues?.()?.[key]?.toString() ?? '';

  if (overridable && !isOverridden) {
    return defaultValue;
  }

  return value ?? defaultValue;
};
