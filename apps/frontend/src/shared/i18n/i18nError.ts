import * as v from 'valibot';

const I18nErrorSchema = v.object({
  key: v.string(),
  values: v.optional(
    v.record(v.string(), v.union([v.string(), v.number(), v.boolean()])),
  ),
});

export const createI18nError = (
  key: string,
  values?: Record<string, string | number | boolean>,
) => JSON.stringify({ key, values });

export const parseI18nError = (
  value: string,
):
  | {
      key: string;
      values: Record<string, string | number | boolean> | undefined;
    }
  | undefined => {
  let parsed: unknown;

  try {
    parsed = JSON.parse(value);
  } catch {
    return;
  }

  const result = v.safeParse(I18nErrorSchema, parsed);

  if (!result.success) {
    return;
  }

  return { key: result.output.key, values: result.output.values };
};
