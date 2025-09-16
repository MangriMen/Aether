import type { IconifyIcon } from '@iconify-icon/solid';
import type { Component, ComponentProps } from 'solid-js';

import MdiBrushIcon from '@iconify/icons-mdi/brush-variant';
import MdiGamepadSquare from '@iconify/icons-mdi/gamepad-square';
import MdiPuzzleIcon from '@iconify/icons-mdi/puzzle';
import MdiTestTubeIcon from '@iconify/icons-mdi/test-tube';
import MdiUpdateIcon from '@iconify/icons-mdi/update';

import { AppearancePane } from '../ui/AppearancePane/AppearancePane';
import { DefaultInstanceSettingsPane } from '../ui/DefaultInstanceSettingsPane/DefaultInstanceSettingsPane';
import { ExperimentalPane } from '../ui/ExperimentalPane/ExperimentalPane';
import { PluginsPane } from '../ui/PluginsPane/PluginsPane';
import { UpdatePane } from '../ui/UpdatePane/UpdatePane';

export enum SettingsTabs {
  Appearance = 'appearance',
  DefaultInstanceSettings = 'defaultInstanceSettings',
  Update = 'update',
  Plugins = 'plugins',
  Experimental = 'experimental',
}

export const SETTINGS_TABS_TRIGGER: {
  icon?: IconifyIcon;
  title:
    | 'appearance'
    | 'update'
    | 'defaultInstanceSettings'
    | 'plugins'
    | 'experimental';
  value: SettingsTabs;
}[] = [
  {
    icon: MdiBrushIcon,
    title: 'appearance',
    value: SettingsTabs.Appearance,
  },
  {
    icon: MdiGamepadSquare,
    title: 'defaultInstanceSettings',
    value: SettingsTabs.DefaultInstanceSettings,
  },
  {
    icon: MdiPuzzleIcon,
    title: 'plugins',
    value: SettingsTabs.Plugins,
  },
  {
    icon: MdiUpdateIcon,
    title: 'update',
    value: SettingsTabs.Update,
  },
  {
    icon: MdiTestTubeIcon,
    title: 'experimental',
    value: SettingsTabs.Experimental,
  },
] as const;

export const SETTINGS_TABS_CONTENT: {
  component: Component<ComponentProps<'div'>>;
  value: SettingsTabs;
}[] = [
  {
    component: AppearancePane,
    value: SettingsTabs.Appearance,
  },
  {
    component: DefaultInstanceSettingsPane,
    value: SettingsTabs.DefaultInstanceSettings,
  },
  {
    component: PluginsPane,
    value: SettingsTabs.Plugins,
  },
  {
    component: UpdatePane,
    value: SettingsTabs.Update,
  },
  {
    component: ExperimentalPane,
    value: SettingsTabs.Experimental,
  },
] as const;
