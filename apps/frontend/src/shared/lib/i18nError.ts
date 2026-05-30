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
  try {
    return JSON.parse(value);
  } catch {
    return;
  }
};
