import dayjs from 'dayjs';
import duration from 'dayjs/plugin/duration';
import localeData from 'dayjs/plugin/localeData';
import relativeTime from 'dayjs/plugin/relativeTime';
import timezone from 'dayjs/plugin/timezone';
import utc from 'dayjs/plugin/utc';
import 'dayjs/locale/en';
import 'dayjs/locale/ru';

dayjs.extend(duration);
dayjs.extend(localeData);
dayjs.extend(relativeTime);
dayjs.extend(timezone);
dayjs.extend(utc);

export default dayjs;
