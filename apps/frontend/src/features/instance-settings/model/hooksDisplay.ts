import type { HookFieldKey } from './hooksFields';
import type { HooksSettingsSchemaInput } from './hooksSettingsSchema';

export const getHookDisplayValue = (
  defaultValues:
    | (() => Partial<HooksSettingsSchemaInput> | undefined)
    | undefined,
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
