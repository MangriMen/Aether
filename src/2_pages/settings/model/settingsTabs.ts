import type { IconifyIcon } from '@iconify-icon/solid';
import type { Component, ComponentProps } from 'solid-js';

import MdiBrushIcon from '@iconify/icons-mdi/brush-variant';
import MdiPuzzleIcon from '@iconify/icons-mdi/puzzle';
import MdiTestTubeIcon from '@iconify/icons-mdi/test-tube';
import MdiUpdateIcon from '@iconify/icons-mdi/update';

import { AppearancePane } from '../ui/AppearancePane/AppearancePane';
import { ExperimentalPane } from '../ui/ExperimentalPane/ExperimentalPane';
import { PluginsPane } from '../ui/PluginsPane/PluginsPane';
import { UpdatePane } from '../ui/UpdatePane/UpdatePane';

export enum SettingsTabs {
  Appearance = 'appearance',
  Experimental = 'experimental',
  Plugins = 'plugins',
  Update = 'update',
}

export const SETTINGS_TABS_TRIGGER: {
  icon?: IconifyIcon;
  title: 'appearance' | 'experimental' | 'plugins' | 'update';
  value: SettingsTabs;
}[] = [
  {
    icon: MdiBrushIcon,
    title: 'appearance',
    value: SettingsTabs.Appearance,
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
];

export const SETTINGS_TABS_CONTENT: {
  component: Component<ComponentProps<'div'>>;
  value: SettingsTabs;
}[] = [
  {
    component: AppearancePane,
    value: SettingsTabs.Appearance,
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
];
