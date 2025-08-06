import {
  type FormStore,
  getError,
  setValue,
  type FieldPath,
  type FieldPathValue,
} from '@modular-forms/solid';

import type { JavaAndMemorySettingsSchemaValues } from '../model/javaAndMemoryValidation';

type JavaFieldPath = FieldPath<JavaAndMemorySettingsSchemaValues>;
type JavaFieldValue<
  TFieldPath extends FieldPath<JavaAndMemorySettingsSchemaValues>,
> = FieldPathValue<JavaAndMemorySettingsSchemaValues, TFieldPath>;

export const useFieldOnChangeWithMapping = <
  TFieldPath extends JavaFieldPath,
  TFieldValue extends JavaFieldValue<TFieldPath>,
  TSyncValue = unknown,
>(
  form: FormStore<JavaAndMemorySettingsSchemaValues>,
  path: TFieldPath,
  syncMapper: (value: TFieldValue) => TSyncValue,
  onSync: (value: TSyncValue) => void,
) => {
  return (value: TFieldValue) => {
    setValue(form, path, value);

    if (!getError(form, path)) {
      onSync(syncMapper(value));
    }
  };
};
