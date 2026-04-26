import { type Component, type ComponentProps } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { SettingsEntry } from '@/shared/ui';

import { BackgroundOpacitySlider } from './BackgroundOpacitySlider';

export type BackgroundOpacityEntryProps = ComponentProps<'div'>;

export const BackgroundOpacityEntry: Component<BackgroundOpacityEntryProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  return (
    <SettingsEntry
      title={t('settings.themeBackgroundOpacity')}
      isTopLevel={false}
      {...props}
    >
      <BackgroundOpacitySlider />
    </SettingsEntry>
  );
};
