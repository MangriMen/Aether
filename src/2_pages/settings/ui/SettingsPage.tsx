import type { RouteSectionProps } from '@solidjs/router';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { AppVersion } from './AppVersion';
import { PluginsPane } from './PluginsPane/PluginsPane';
import { LauncherPane } from './LauncherPane/LauncherPane';
import { UpdatePane } from './UpdatePane/UpdatePane';

export type SettingsPageProps = ComponentProps<'div'> & RouteSectionProps;

export const SettingsPage: Component<SettingsPageProps> = (props) => {
  const [_, others] = splitProps(props, ['params', 'location', 'data']);

  return (
    <div class='flex size-full flex-col gap-4 overflow-y-auto p-4' {...others}>
      <LauncherPane />
      <UpdatePane />
      <PluginsPane />
      <AppVersion class='mt-auto' />
    </div>
  );
};
