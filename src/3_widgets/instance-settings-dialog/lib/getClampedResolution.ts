export const getClampedResolution = (value: number | null, min: number) => {
  if (value === null || value < 0) {
    return min;
  } else if (value > 65535) {
    return 65535;
  }

  return value;
};
