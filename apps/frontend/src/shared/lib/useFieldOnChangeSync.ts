import type {
  FieldValues,
  FormStore,
  FieldPath,
  FieldPathValue,
} from '@modular-forms/solid';
import type { AnyZodObject, z, ZodTypeAny } from 'zod';

import { getValue, setValue, validate } from '@modular-forms/solid';

export const useFieldOnChangeSync = <
  TFieldValues extends FieldValues,
  TFieldPath extends FieldPath<TFieldValues>,
  TFieldValue extends FieldPathValue<TFieldValues, TFieldPath>,
  // Извлекаем тип Output из переданной схемы
  TSchema extends ZodTypeAny,
  TTransformedValue = TSchema extends AnyZodObject
    ? z.output<TSchema>[TFieldPath]
    : z.output<TSchema>,
  TSyncValue = TTransformedValue,
>(
  schema: TSchema,
  form: FormStore<TFieldValues>,
  path: TFieldPath,
  onSync: (value: TSyncValue) => void,
  syncMapper?: (value: TTransformedValue) => TSyncValue,
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

    let fieldSchema: ZodTypeAny = schema;

    if ('shape' in schema) {
      const objectSchema = schema as unknown as AnyZodObject;
      const shapeField = objectSchema.shape[path as string] as
        | ZodTypeAny
        | undefined;

      if (shapeField) {
        fieldSchema = shapeField;
      }
    }

    const result = fieldSchema.safeParse(raw);

    if (!result.success) {
      return;
    }

    // Теперь parsedValue имеет корректный тип из схемы, а не unknown
    const parsedValue = result.data as TTransformedValue;

    const finalValue = syncMapper
      ? syncMapper(parsedValue)
      : (parsedValue as unknown as TSyncValue);

    onSync(finalValue);
  };
};
