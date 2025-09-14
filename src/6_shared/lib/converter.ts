export const stringToNumber = (value: string): number | null => {
  const num = Number(value.replace(/\D/g, ''));
  if (Number.isNaN(num)) {
    return null;
  }

  return num;
};
