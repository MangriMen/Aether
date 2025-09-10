import type {
  FieldValues,
  FormStore,
  FieldPath,
  FieldPathValue,
} from '@modular-forms/solid';

import { getValue, setValue, validate } from '@modular-forms/solid';
import type { ZodObject, ZodTypeAny } from 'zod';

export const useFieldOnChangeSync = <
  TFieldValues extends FieldValues,
  TFieldPath extends FieldPath<TFieldValues>,
  TFieldValue extends FieldPathValue<TFieldValues, TFieldPath>,
  TSyncValue = unknown,
>(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  schema: ZodTypeAny | ZodObject<any>,
  form: FormStore<TFieldValues>,
  path: TFieldPath,
  syncMapper: (value: TFieldValue) => TSyncValue,
  onSync: (value: TSyncValue) => unknown,
) => {
  return async (value?: TFieldValue) => {
    if (value !== undefined) {
      setValue(form, path, value);
    }

    const valid = await validate(form, path, {
      shouldActive: false,
      shouldFocus: false,
    });

    if (!valid) {
      return;
    }

    const raw = getValue(form, path, { shouldValid: true });

    let finalValue: TFieldValue;

    if ('pick' in schema && typeof schema.pick === 'function') {
      const parsed = schema.pick({ [path]: true }).safeParse({ [path]: raw });
      if (!parsed.success) return;
      finalValue = parsed.data[path];
    } else {
      const parsed = (schema as ZodTypeAny).safeParse(raw);
      if (!parsed.success) return;
      finalValue = parsed.data;
    }

    onSync(syncMapper(finalValue));
  };
};
