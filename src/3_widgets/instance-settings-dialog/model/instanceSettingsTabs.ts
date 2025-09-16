import type { IconifyIcon } from '@iconify-icon/solid';
import type { Component } from 'solid-js';

import MdiArrowDecisionOutline from '@iconify/icons-mdi/arrow-decision-outline';
import MdiCoffeeOutlineIcon from '@iconify/icons-mdi/coffee-outline';
import MdiInfoOutlineIcon from '@iconify/icons-mdi/info-outline';
import MdiMonitorIcon from '@iconify/icons-mdi/monitor';
import MdiToolsIcon from '@iconify/icons-mdi/tools';

import type { EditInstance, Instance } from '@/entities/instances';
import type { DefaultInstanceSettings } from '@/entities/settings/model/defaultInstanceSettings';

import { WindowTab } from '@/widgets/instance-settings-dialog/ui/WindowTab';

import { GeneralTab } from '../ui/GeneralTab';
import { HooksTab } from '../ui/HooksTab';
import { InstallationTab } from '../ui/InstallationTab';
import { JavaAndMemoryTab } from '../ui/JavaAndMemoryTab';

export type InstanceSettingsTabProps = {
  instance: Instance;
  editInstance: (args: { id: string; edit: EditInstance }) => Promise<unknown>;
  globalSettings?: DefaultInstanceSettings;
};

export enum InstanceSettingsDialogTabs {
  General = 'general',
  Installation = 'installation',
  Window = 'window',
  JavaAndMemory = 'java-and-memory',
  Hooks = 'hooks',
}

export const INSTANCE_SETTINGS_TABS_TRIGGER: {
  icon: IconifyIcon;
  title: 'general' | 'installation' | 'window' | 'javaAndMemory' | 'hooks';
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
  {
    icon: MdiArrowDecisionOutline,
    title: 'hooks',
    value: InstanceSettingsDialogTabs.Hooks,
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
  {
    component: HooksTab,
    value: InstanceSettingsDialogTabs.Hooks,
  },
];
