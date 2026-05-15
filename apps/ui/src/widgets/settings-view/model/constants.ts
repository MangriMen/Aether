import type { Locale, Option } from '@/shared/model';

import { LOCALES } from '@/shared/model';

import { SettingsTab } from './settingsTabs';

export const DEFAULT_TAB = SettingsTab.Appearance;

export const LOCALE_OPTIONS: Option<Locale>[] = [
  { name: 'English', value: LOCALES.En },
  { name: 'Русский', value: LOCALES.Ru },
];

export const IS_SETTINGS_MODAL_FULLSCREEN_KEY =
  'aether:settings-modal-fullscreen';
