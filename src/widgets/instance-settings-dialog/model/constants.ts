import MdiCoffeeOutlineIcon from '@iconify/icons-mdi/coffee-outline';
import MdiInfoOutlineIcon from '@iconify/icons-mdi/info-outline';
import MdiMonitorIcon from '@iconify/icons-mdi/monitor';
import type { IconifyIcon } from '@iconify-icon/solid';
import type { Component } from 'solid-js';

import type { InstanceSettingsTabProps } from '@/entities/instances';

import { WindowTab } from '@/widgets/instance-settings-dialog/ui/WindowTab';
import { GeneralTab } from '../ui/GeneralTab';
import { JavaAndMemoryTab } from '../ui/JavaAndMemoryTab';

export enum InstanceSettingsDialogTabs {
  General = 'general',
  Window = 'window',
  JavaAndMemory = 'java-and-memory',
}

export const INSTANCE_SETTINGS_TABS_TRIGGER: {
  icon: IconifyIcon;
  title: 'general' | 'window' | 'javaAndMemory';
  value: InstanceSettingsDialogTabs;
}[] = [
  {
    icon: MdiInfoOutlineIcon,
    title: 'general',
    value: InstanceSettingsDialogTabs.General,
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
    component: WindowTab,
    value: InstanceSettingsDialogTabs.Window,
  },
  {
    component: JavaAndMemoryTab,
    value: InstanceSettingsDialogTabs.JavaAndMemory,
  },
];
