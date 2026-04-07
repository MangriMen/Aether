type SearchParam = string | string[] | undefined;

export const parseSearchParamToString = (
  field: SearchParam,
): string | undefined => {
  return typeof field === 'string' ? field : undefined;
};

export const parseSearchParamToNumber = (
  field: SearchParam,
): number | undefined => {
  const val = parseSearchParamToString(field);
  if (val === undefined) return undefined;

  const number = Number.parseFloat(val);
  return Number.isNaN(number) ? undefined : number;
};

export const parseSearchParamToStringArray = (
  field: SearchParam,
): string[] | undefined => {
  if (field === undefined) {
    return undefined;
  }

  const array = Array.isArray(field) ? field : [field];
  return array.filter((x): x is string => typeof x === 'string');
};
