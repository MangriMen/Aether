import type { IconifyIcon } from '@iconify-icon/solid';
import type { Component, ComponentProps } from 'solid-js';
import { AppearancePane } from '../ui/AppearancePane/AppearancePane';
import { UpdatePane } from '../ui/UpdatePane/UpdatePane';
import { PluginsPane } from '../ui/PluginsPane/PluginsPane';
import MdiBrushIcon from '@iconify/icons-mdi/brush-variant';
import MdiUpdateIcon from '@iconify/icons-mdi/update';
import MdiPuzzleIcon from '@iconify/icons-mdi/puzzle';
import MdiTestTubeIcon from '@iconify/icons-mdi/test-tube';
import { ExperimentalPane } from '../ui/ExperimentalPane/ExperimentalPane';

export enum SettingsTabs {
  Appearance = 'appearance',
  Update = 'update',
  Plugins = 'plugins',
  Experimental = 'experimental',
}

export const SETTINGS_TABS_TRIGGER: {
  icon?: IconifyIcon;
  title: 'appearance' | 'update' | 'plugins' | 'experimental';
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
