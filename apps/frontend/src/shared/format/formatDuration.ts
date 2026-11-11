const formatUnit = (
  value: number,
  unit: 'minute' | 'second',
  locale?: string,
) =>
  new Intl.NumberFormat(locale, {
    style: 'unit',
    unit,
    unitDisplay: 'long',
  }).format(value);

/**
 * A length of time in whole minutes and seconds, declined by the language of the interface:
 * `15 seconds`, `10 minutes`, `1 minute 30 seconds` — `1 минута 30 секунд` in Russian.
 * A part that is zero is left out; fractions of a second are rounded.
 */
export const formatDuration = (
  totalSeconds: number,
  locale?: string,
): string => {
  const rounded = Math.max(0, Math.round(totalSeconds));
  const minutes = Math.floor(rounded / 60);
  const seconds = rounded % 60;

  const parts = [
    ...(minutes > 0 ? [formatUnit(minutes, 'minute', locale)] : []),
    ...(seconds > 0 || minutes === 0
      ? [formatUnit(seconds, 'second', locale)]
      : []),
  ];

  return parts.join(' ');
};
