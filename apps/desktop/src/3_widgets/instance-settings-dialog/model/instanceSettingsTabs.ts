import type { Component, ComponentProps } from 'solid-js';

import IconMdiArrowDecisionOutline from '~icons/mdi/arrow-decision-outline';
import IconMdiCoffeeOutline from '~icons/mdi/coffee-outline';
import IconMdiInformationOutline from '~icons/mdi/information-outline';
import IconMdiMonitor from '~icons/mdi/monitor';
import IconMdiTools from '~icons/mdi/tools';

import type { EditInstance, Instance } from '@/entities/instances';
import type { DefaultInstanceSettings } from '@/entities/settings/model/defaultInstanceSettings';

import { GeneralTab } from '../ui/GeneralTab';
import { HooksTab } from '../ui/HooksTab';
import { InstallationTab } from '../ui/InstallationTab';
import { JavaAndMemoryTab } from '../ui/JavaAndMemoryTab';
import { WindowTab } from '../ui/WindowTab';

export type InstanceSettingsTabProps = {
  instance: Instance;
  editInstance: (args: { id: string; edit: EditInstance }) => Promise<unknown>;
  defaultSettings?: DefaultInstanceSettings;
};

export enum InstanceSettingsDialogTabs {
  General = 'general',
  Installation = 'installation',
  Window = 'window',
  JavaAndMemory = 'java-and-memory',
  Hooks = 'hooks',
}

export const INSTANCE_SETTINGS_TABS_TRIGGER: {
  icon: Component<ComponentProps<'svg'>>;
  title: 'general' | 'installation' | 'window' | 'javaAndMemory' | 'hooks';
  value: InstanceSettingsDialogTabs;
}[] = [
  {
    icon: IconMdiInformationOutline,
    title: 'general',
    value: InstanceSettingsDialogTabs.General,
  },
  {
    icon: IconMdiTools,
    title: 'installation',
    value: InstanceSettingsDialogTabs.Installation,
  },
  {
    icon: IconMdiMonitor,
    title: 'window',
    value: InstanceSettingsDialogTabs.Window,
  },
  {
    icon: IconMdiCoffeeOutline,
    title: 'javaAndMemory',
    value: InstanceSettingsDialogTabs.JavaAndMemory,
  },
  {
    icon: IconMdiArrowDecisionOutline,
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
