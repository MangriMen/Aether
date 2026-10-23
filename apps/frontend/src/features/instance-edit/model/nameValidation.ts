import * as v from 'valibot';

import { GeneralSettingsSchema } from './generalValidation';

type InstanceNameErrorKey =
  | 'generalSettingsError.name.maxLength'
  | 'generalSettingsError.name.minLength';

export type InstanceNameValidation =
  | { valid: true; name: string }
  | { valid: false; errorKey: InstanceNameErrorKey };

/** Trims the name and reports the first schema violation, if any. */
export const validateInstanceName = (input: string): InstanceNameValidation => {
  const result = v.safeParse(GeneralSettingsSchema.entries.name, input);

  if (result.success) {
    return { valid: true, name: result.output };
  }

  return {
    valid: false,
    errorKey:
      result.issues[0]?.message === 'generalSettingsError.name.maxLength'
        ? 'generalSettingsError.name.maxLength'
        : 'generalSettingsError.name.minLength',
  };
};
