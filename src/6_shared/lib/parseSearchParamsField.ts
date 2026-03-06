type SearchParam = string | string[] | undefined;

export const parseSearchParamToString = (
  field: SearchParam,
): string | undefined => {
  if (field === undefined || typeof field !== 'string') {
    return undefined;
  }

  return decodeURIComponent(field);
};

export const parseSearchParamToNumber = (
  field: SearchParam,
): number | undefined => {
  const string = parseSearchParamToString(field);

  if (string === undefined) {
    return undefined;
  }

  const number = Number.parseFloat(string);

  return Number.isNaN(number) ? undefined : number;
};

export const parseSearchParamToStringArray = (
  field: SearchParam,
): string[] | undefined => {
  if (field === undefined || !Array.isArray(field)) {
    return undefined;
  }

  return field.map(parseSearchParamToString).filter((x) => x !== undefined);
};
