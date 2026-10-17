import { dayjs } from '@/shared/format';

export const formatTimePlayedHumanized = (value: number): string => {
  return dayjs.duration(value, 's').humanize();
};
