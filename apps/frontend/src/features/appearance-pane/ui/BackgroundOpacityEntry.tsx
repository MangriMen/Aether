import { type Component, type ComponentProps } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { SettingsEntry } from '@/shared/settings-layout';

import { BackgroundOpacitySlider } from './BackgroundOpacitySlider';

export type BackgroundOpacityEntryProps = ComponentProps<'div'>;

export const BackgroundOpacityEntry: Component<BackgroundOpacityEntryProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  return (
    <SettingsEntry title={t('settings.themeBackgroundOpacity')} {...props}>
      <BackgroundOpacitySlider />
    </SettingsEntry>
  );
};
