import {
  type FieldValues,
  type FormStore,
  getError,
  setValue,
  type FieldPath,
  type FieldPathValue,
} from '@modular-forms/solid';

export const useFieldOnChangeWithMapping = <
  TFieldValues extends FieldValues,
  TFieldPath extends FieldPath<TFieldValues>,
  TFieldValue extends FieldPathValue<TFieldValues, TFieldPath>,
  TSyncValue = unknown,
>(
  form: FormStore<TFieldValues>,
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
