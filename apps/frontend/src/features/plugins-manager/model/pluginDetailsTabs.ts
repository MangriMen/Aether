import type { Plugin } from '@/features/plugins';
import type { TabConfig } from '@/shared/ui-kit';

import { PluginSettingsTab } from '@/features/plugins-settings';

import { PluginCapabilitiesTab } from '../ui/PluginCapabilitiesTab';
import { PluginDescriptionTab } from '../ui/PluginDescriptionTab';

export const PluginDetailsTabs = {
  Description: 'description',
  Capabilities: 'capabilities',
  Settings: 'settings',
} as const;

export interface PluginDetailsTabProps {
  plugin: Plugin;
}

export const PLUGIN_DETAILS_TABS = [
  {
    value: PluginDetailsTabs.Description,
    label: 'description',
    component: PluginDescriptionTab,
  },
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
