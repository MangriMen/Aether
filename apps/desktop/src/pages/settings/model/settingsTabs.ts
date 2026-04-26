import type { ComponentProps } from 'solid-js';

import IconMdiBrush from '~icons/mdi/brush-variant';
import IconMdiGamepadSquare from '~icons/mdi/gamepad-square';
import IconMdiPuzzle from '~icons/mdi/puzzle';
import IconMdiTestTube from '~icons/mdi/test-tube';
import IconMdiUpdate from '~icons/mdi/update';

import type { TabConfig } from '@/shared/model';

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

export const SETTINGS_TABS_DEFINITION = [
  {
    value: SettingsTab.Appearance,
    label: 'appearance',
    icon: IconMdiBrush,
    component: AppearancePane,
  },
  {
    value: SettingsTab.DefaultInstanceSettings,
    label: 'defaultInstanceSettings',
    icon: IconMdiGamepadSquare,
    component: DefaultInstanceSettingsPane,
  },
  {
    value: SettingsTab.Plugins,
    label: 'plugins',
    icon: IconMdiPuzzle,
    component: PluginsPane,
  },
  {
    value: SettingsTab.Update,
    label: 'update',
    icon: IconMdiUpdate,
    component: UpdatePane,
  },
  {
    value: SettingsTab.Experimental,
    label: 'experimental',
    icon: IconMdiTestTube,
    component: ExperimentalPane,
  },
] as const satisfies TabConfig<SettingsTab, ComponentProps<'div'>>[];

export const SettingsTabs = new Set(Object.values(SettingsTab));

export const isSettingsTab = (value: string): value is SettingsTab =>
  SettingsTabs.has(value as SettingsTab);
