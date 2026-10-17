import type { Plugin } from '@/features/plugins';
import type { TabConfig } from '@/shared/ui-kit';

import { PluginSettingsTab } from '@/features/plugins-settings';

import { PluginCapabilitiesTab } from '../ui/PluginCapabilitiesTab';

export const PluginDetailsTabs = {
  Capabilities: 'capabilities',
  Settings: 'settings',
} as const;

export type PluginDetailsTab =
  (typeof PluginDetailsTabs)[keyof typeof PluginDetailsTabs];

export interface PluginDetailsTabProps {
  plugin: Plugin;
}

export const PLUGIN_DETAILS_TABS = [
  // TODO: add a description tab, maybe in MD format
  {
    value: PluginDetailsTabs.Capabilities,
    label: 'capabilities',
    component: PluginCapabilitiesTab,
  },
  {
    value: PluginDetailsTabs.Settings,
    label: 'settings',
    component: PluginSettingsTab,
  },
] as const satisfies TabConfig<string, PluginDetailsTabProps>[];
