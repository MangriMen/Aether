import type { Component, ComponentProps } from 'solid-js';

import IconMdiBrush from '~icons/mdi/brush-variant';
import IconMdiGamepadSquare from '~icons/mdi/gamepad-square';
import IconMdiPuzzle from '~icons/mdi/puzzle';
import IconMdiTestTube from '~icons/mdi/test-tube';
import IconMdiUpdate from '~icons/mdi/update';

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
  icon?: Component<ComponentProps<'svg'>>;
  title:
    | 'appearance'
    | 'update'
    | 'defaultInstanceSettings'
    | 'plugins'
    | 'experimental';
  value: SettingsTabs;
}[] = [
  {
    icon: IconMdiBrush,
    title: 'appearance',
    value: SettingsTabs.Appearance,
  },
  {
    icon: IconMdiGamepadSquare,
    title: 'defaultInstanceSettings',
    value: SettingsTabs.DefaultInstanceSettings,
  },
  {
    icon: IconMdiPuzzle,
    title: 'plugins',
    value: SettingsTabs.Plugins,
  },
  {
    icon: IconMdiUpdate,
    title: 'update',
    value: SettingsTabs.Update,
  },
  {
    icon: IconMdiTestTube,
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
