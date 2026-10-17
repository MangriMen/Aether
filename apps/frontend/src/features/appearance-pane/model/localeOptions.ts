import type { Locale } from '@/shared/i18n';
import type { Option } from '@/shared/ui-kit';

import { LOCALES } from '@/shared/i18n';

export const LOCALE_OPTIONS: Option<Locale>[] = [
  { name: 'English', value: LOCALES.En },
  { name: 'Русский', value: LOCALES.Ru },
];
