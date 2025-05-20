import { createAsync } from '@solidjs/router';
import {
  version as getOsVersion,
  platform as getOsPlatform,
} from '@tauri-apps/plugin-os';
import type { Component, ComponentProps } from 'solid-js';
import { createMemo, Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { getVersion } from '@tauri-apps/api/app';

export type AppVersionProps = ComponentProps<'div'>;

export const AppVersion: Component<AppVersionProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const appVersion = createAsync(() => getVersion());
  const osPlatform = createMemo(() => getOsPlatform());
  const osVersion = createMemo(() => getOsVersion());

  return (
    <div
      class={cn(
        'flex flex-col text-muted-foreground rounded-md p-2 w-max',
        local.class,
      )}
      {...others}
    >
      <p class='m-0'>Aether {appVersion()}</p>
      <p class='m-0'>
        <span class='capitalize'>
          <Show when={osPlatform() === 'macos'} fallback={osPlatform()}>
            MacOS
          </Show>
        </span>
        &nbsp;
        {osVersion()}
      </p>
    </div>
  );
};
