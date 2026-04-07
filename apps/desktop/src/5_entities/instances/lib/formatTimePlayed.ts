import { dayjs } from '@/shared/lib';

export const formatTimePlayedHumanized = (value: number): string => {
  return dayjs.duration(value, 's').humanize();
};
