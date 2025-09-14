import type { Platform } from '@tauri-apps/plugin-os';
import type { Component, ComponentProps } from 'solid-js';

import {
  platform as getOsPlatform,
  version as getOsVersion,
} from '@tauri-apps/plugin-os';
import { createMemo } from 'solid-js';

const OS_PLATFORM_TO_DISPLAY: Record<Platform, string> = {
  android: 'Android',
  dragonfly: 'DragonFly',
  freebsd: 'FreeBSD',
  ios: 'IOS',
  linux: 'Linux',
  macos: 'MacOS',
  netbsd: 'NetBSD',
  openbsd: 'OpenBSD',
  solaris: 'Solaris',
  windows: 'Windows',
};

export type OsVersionProps = ComponentProps<'span'>;

export const OsVersion: Component<OsVersionProps> = (props) => {
  const osPlatform = createMemo(() => getOsPlatform());
  const osVersion = createMemo(() => getOsVersion());

  return (
    <span {...props}>
      {OS_PLATFORM_TO_DISPLAY[osPlatform()]}
      &nbsp;
      {osVersion()}
    </span>
  );
};
