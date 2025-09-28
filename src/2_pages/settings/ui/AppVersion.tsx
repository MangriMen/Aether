import type { Component, ComponentProps } from 'solid-js';

import { createAsync } from '@solidjs/router';
import { getVersion } from '@tauri-apps/api/app';

export type AppVersionProps = ComponentProps<'span'>;

export const AppVersion: Component<AppVersionProps> = (props) => {
  const appVersion = createAsync(() => getVersion());
  return <span {...props}>Aether {appVersion()}</span>;
};
