import { cn } from '@/shared/lib';
import { SettingsPane } from '@/shared/ui';
import { useTranslation } from '@/shared/model';
import { splitProps, type Component, type ComponentProps } from 'solid-js';
import { SelectWindowEffect } from './SelectWindowEffect';
import { ToggleWindowTransparencyEntry } from './ToggleWindowTransparencyEntry';
import { ThemeBackgroundOpacityEntry } from './ThemeBackgroundOpacityEntry';

export type ExperimentalPaneProps = ComponentProps<'div'>;

export const ExperimentalPane: Component<ExperimentalPaneProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  return (
    <SettingsPane
      class={cn('container max-w-screen-lg', local.class)}
      label={t('settings.tab.experimental')}
      {...others}
    >
      <ToggleWindowTransparencyEntry />
      <ThemeBackgroundOpacityEntry />
      <SelectWindowEffect />
    </SettingsPane>
  );
};
