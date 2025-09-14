import type { Locale, Option } from '@/shared/model';
import { LOCALES } from '@/shared/model';

export const LOCALE_OPTIONS: Option<Locale>[] = [
  { name: 'English', value: LOCALES.En },
  { name: 'Русский', value: LOCALES.Ru },
];
