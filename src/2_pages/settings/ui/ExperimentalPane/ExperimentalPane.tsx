import { cn } from '@/shared/lib';
import { SettingsPane } from '@/shared/ui';
import { useTranslate } from '@/shared/model';
import { splitProps, type Component, type ComponentProps } from 'solid-js';
import { SelectWindowEffect } from './SelectWindowEffect';
import { ToggleWindowTransparencyEntry } from './ToggleWindowTransparencyEntry';
import { ToggleThemeTransparencyEntry } from './ToggleThemeTransparencyEntry';

export type ExperimentalPaneProps = ComponentProps<'div'>;

export const ExperimentalPane: Component<ExperimentalPaneProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslate();

  return (
    <SettingsPane
      class={cn('container max-w-screen-lg', local.class)}
      label={t('settings.experimental')}
      {...others}
    >
      <ToggleWindowTransparencyEntry />
      <ToggleThemeTransparencyEntry />
      <SelectWindowEffect />
    </SettingsPane>
  );
};
