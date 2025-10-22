import type { Plugin } from '@/entities/plugins';
import type { TabConfig } from '@/shared/model';

import { PluginCapabilitiesTab } from '../ui/PluginCapabilitiesTab';
import { PluginDescriptionTab } from '../ui/PluginDescriptionTab';
import { PluginSettingsTab } from '../ui/PluginSettingsTab';

export const PluginDetailsTabs = {
  Description: 'description',
  Capabilities: 'capabilities',
  Settings: 'settings',
} as const;

export type PluginDetailsTab =
  (typeof PluginDetailsTabs)[keyof typeof PluginDetailsTabs];

export interface PluginDetailsTabProps {
  plugin: Plugin;
  isSettingsDisabled?: boolean;
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
