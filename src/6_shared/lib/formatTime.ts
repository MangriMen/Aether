import type { Dayjs } from 'dayjs';

import dayjs from './dayjs';

export const formatTime = (time: Dayjs) => {
  const tz = dayjs.tz.guess();
  return time.tz(tz).format('LLL');
};
