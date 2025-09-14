import { createMemo } from 'solid-js';
import type { Component, ComponentProps } from 'solid-js';
import type { Platform } from '@tauri-apps/plugin-os';
import {
  version as getOsVersion,
  platform as getOsPlatform,
} from '@tauri-apps/plugin-os';

const OS_PLATFORM_TO_DISPLAY: Record<Platform, string> = {
  linux: 'Linux',
  macos: 'MacOS',
  ios: 'IOS',
  freebsd: 'FreeBSD',
  dragonfly: 'DragonFly',
  netbsd: 'NetBSD',
  openbsd: 'OpenBSD',
  solaris: 'Solaris',
  android: 'Android',
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
