import { SettingsEntry, SettingsPane } from '@/shared/ui';
import { splitProps, type Component, type ComponentProps } from 'solid-js';
import SelectThemeForColorModeEntry from './SelectThemeForColorModeEntry';
import { SelectActionOnInstanceLaunchEntry } from './SelectActionOnInstanceLaunchEntry';
import { useTranslate } from '@/shared/model';
import { cn } from '@/shared/lib';
import ChangeLanguageEntry from './ChangeLanguageEntry';
import { SelectTheme } from './SelectTheme';

export type LauncherPaneProps = ComponentProps<'div'>;

export const LauncherPane: Component<LauncherPaneProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslate();

  return (
    <SettingsPane
      class={cn('container max-w-screen-lg', local.class)}
      label={t('settings.launcher')}
      {...others}
    >
      <SettingsEntry
        title={t('settings.colorScheme')}
        description={t('settings.colorSchemeDescription')}
      >
        <SelectTheme />
      </SettingsEntry>
      <SelectThemeForColorModeEntry />
      <ChangeLanguageEntry />
      <SelectActionOnInstanceLaunchEntry />
    </SettingsPane>
  );
};
