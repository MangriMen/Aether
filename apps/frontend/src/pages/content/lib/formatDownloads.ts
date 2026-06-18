export const formatDownloads = (value: number | bigint, locale = 'en-US') => {
  return new Intl.NumberFormat(locale, {
    notation: 'compact',
    compactDisplay: 'short',
    maximumFractionDigits: 2,
  }).format(value);
};
