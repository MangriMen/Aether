export const stringToNumber = (value: string): null | number => {
  const num = Number(value.replace(/\D/g, ''));
  if (Number.isNaN(num)) {
    return null;
  }

  return num;
};
