import dayjs from '@/shared/lib/dayjs';

export const formatTimePlayedHumanized = (value: number): string => {
  return dayjs.duration(value, 's').humanize();
};

export const formatTimePlayed = (value: number): string => {
  return dayjs.duration(value, 's').format('D[d] H[h] m[m] s[s]');
};
