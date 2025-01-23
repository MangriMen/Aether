export const stringToNumber = (value: string): number | null => {
  const num = Number(value.replace(/[^0-9]/g, ''));
  if (Number.isNaN(num)) {
    return null;
  }

  return num;
};
