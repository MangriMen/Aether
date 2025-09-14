import type { IconifyIcon } from '@iconify-icon/solid';
import type { Component } from 'solid-js';

import MdiCoffeeOutlineIcon from '@iconify/icons-mdi/coffee-outline';
import MdiInfoOutlineIcon from '@iconify/icons-mdi/info-outline';
import MdiMonitorIcon from '@iconify/icons-mdi/monitor';
import MdiToolsIcon from '@iconify/icons-mdi/tools';

import type { Instance } from '@/entities/instances';
import type { Settings } from '@/entities/settings';

import { WindowTab } from '@/widgets/instance-settings-dialog/ui/WindowTab/WindowTab';

import { GeneralTab } from '../ui/GeneralTab';
import { InstallationTab } from '../ui/InstallationTab';
import { JavaAndMemoryTab } from '../ui/JavaAndMemoryTab/JavaAndMemoryTab';

export enum InstanceSettingsDialogTabs {
  General = 'general',
  Installation = 'installation',
  JavaAndMemory = 'java-and-memory',
  Window = 'window',
}

export type InstanceSettingsTabProps = {
  instance: Instance;
  settings?: Settings;
};

export const INSTANCE_SETTINGS_TABS_TRIGGER: {
  icon: IconifyIcon;
  title: 'general' | 'installation' | 'javaAndMemory' | 'window';
  value: InstanceSettingsDialogTabs;
}[] = [
  {
    icon: MdiInfoOutlineIcon,
    title: 'general',
    value: InstanceSettingsDialogTabs.General,
  },
  {
    icon: MdiToolsIcon,
    title: 'installation',
    value: InstanceSettingsDialogTabs.Installation,
  },
  {
    icon: MdiMonitorIcon,
    title: 'window',
    value: InstanceSettingsDialogTabs.Window,
  },
  {
    icon: MdiCoffeeOutlineIcon,
    title: 'javaAndMemory',
    value: InstanceSettingsDialogTabs.JavaAndMemory,
  },
];

export const INSTANCE_SETTINGS_TABS_CONTENT: {
  component: Component<InstanceSettingsTabProps>;
  value: InstanceSettingsDialogTabs;
}[] = [
  {
    component: GeneralTab,
    value: InstanceSettingsDialogTabs.General,
  },
  {
    component: InstallationTab,
    value: InstanceSettingsDialogTabs.Installation,
  },
  {
    component: WindowTab,
    value: InstanceSettingsDialogTabs.Window,
  },
  {
    component: JavaAndMemoryTab,
    value: InstanceSettingsDialogTabs.JavaAndMemory,
  },
];
