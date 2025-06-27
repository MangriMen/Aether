import { SettingsPane } from '@/shared/ui';
import { splitProps, type Component, type ComponentProps } from 'solid-js';
import SelectThemeForColorModeEntry from './SelectThemeForColorModeEntry';
import { SelectActionOnInstanceLaunchEntry } from './SelectActionOnInstanceLaunchEntry';
import { useTranslation } from '@/shared/model';
import { cn } from '@/shared/lib';
import ChangeLanguageEntry from './ChangeLanguageEntry';
import { SelectThemeEntry } from './SelectThemeEntry';
import { DisableAnimationsEntry } from './DisableAnimationsEntry';

export type AppearancePaneProps = ComponentProps<'div'>;

export const AppearancePane: Component<AppearancePaneProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  return (
    <SettingsPane
      class={cn('container max-w-screen-lg', local.class)}
      label={t('settings.appearance')}
      {...others}
    >
      <SelectThemeEntry />
      <SelectThemeForColorModeEntry />
      <DisableAnimationsEntry />
      <ChangeLanguageEntry />
      <SelectActionOnInstanceLaunchEntry />
    </SettingsPane>
  );
};
