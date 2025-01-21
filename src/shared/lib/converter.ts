export const stringToNumber = (value: string): number | undefined => {
  const num = Number(value.replace(/[^0-9]/g, ''));
  if (Number.isNaN(num)) {
    return undefined;
  }

  return num;
};
