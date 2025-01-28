import {default as dayjs, extend as dayjsExtend} from 'dayjs';
import duration from 'dayjs/plugin/duration';
import localeData from 'dayjs/plugin/localeData';
import relativeTime from 'dayjs/plugin/relativeTime';
import timezone from 'dayjs/plugin/timezone';
import utc from 'dayjs/plugin/utc';
import 'dayjs/locale/en';
import 'dayjs/locale/ru';

dayjsExtend(duration);
dayjsExtend(localeData);
dayjsExtend(relativeTime);
dayjsExtend(timezone);
dayjsExtend(utc);

export default dayjs;
