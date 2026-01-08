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

export const SettingsTab = {
  Appearance: 'appearance',
  DefaultInstanceSettings: 'defaultInstanceSettings',
  Update: 'update',
  Plugins: 'plugins',
  Experimental: 'experimental',
} as const;

export type SettingsTab = (typeof SettingsTab)[keyof typeof SettingsTab];

export const SettingsTabs = new Set(Object.values(SettingsTab));

export const SETTINGS_TABS_TRIGGER: {
  icon?: Component<ComponentProps<'svg'>>;
  title:
    | 'appearance'
    | 'update'
    | 'defaultInstanceSettings'
    | 'plugins'
    | 'experimental';
  value: SettingsTab;
}[] = [
  {
    icon: IconMdiBrush,
    title: 'appearance',
    value: SettingsTab.Appearance,
  },
  {
    icon: IconMdiGamepadSquare,
    title: 'defaultInstanceSettings',
    value: SettingsTab.DefaultInstanceSettings,
  },
  {
    icon: IconMdiPuzzle,
    title: 'plugins',
    value: SettingsTab.Plugins,
  },
  {
    icon: IconMdiUpdate,
    title: 'update',
    value: SettingsTab.Update,
  },
  {
    icon: IconMdiTestTube,
    title: 'experimental',
    value: SettingsTab.Experimental,
  },
] as const;

export const SETTINGS_TABS_CONTENT: {
  component: Component<ComponentProps<'div'>>;
  value: SettingsTab;
}[] = [
  {
    component: AppearancePane,
    value: SettingsTab.Appearance,
  },
  {
    component: DefaultInstanceSettingsPane,
    value: SettingsTab.DefaultInstanceSettings,
  },
  {
    component: PluginsPane,
    value: SettingsTab.Plugins,
  },
  {
    component: UpdatePane,
    value: SettingsTab.Update,
  },
  {
    component: ExperimentalPane,
    value: SettingsTab.Experimental,
  },
] as const;

export const isSettingsTab = (value: string): value is SettingsTab =>
  SettingsTabs.has(value as SettingsTab);
