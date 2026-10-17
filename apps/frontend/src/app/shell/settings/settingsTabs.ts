import IconMdiBrush from '~icons/mdi/brush-variant';
import IconMdiCoffeeOutline from '~icons/mdi/coffee-outline';
import IconMdiDatabase from '~icons/mdi/database';
import IconMdiGamepadSquare from '~icons/mdi/gamepad-square';
import IconMdiPackageVariant from '~icons/mdi/package-variant';
import IconMdiPuzzle from '~icons/mdi/puzzle';
import IconMdiTestTube from '~icons/mdi/test-tube';
import IconMdiUpdate from '~icons/mdi/update';

import type { TabConfig } from '@/shared/model';
import type { SettingsPaneProps } from '@/shared/ui';

import { AppearancePane } from '@/features/appearance-pane';
import { ContentProvidersPane } from '@/features/content-providers';
import { DefaultInstanceSettingsPane } from '@/features/instance-defaults';
import { JavaPane } from '@/features/java';
import { PluginsPane } from '@/features/plugins-manager';
import { SettingsTab } from '@/features/settings';
import { UpdatePane } from '@/features/update';

import { ExperimentalPane } from './panes/ExperimentalPane';
import { StoragePane } from './panes/StoragePane';

export const SETTINGS_TABS_DEFINITION = [
  {
    value: SettingsTab.Appearance,
    label: 'appearance',
    icon: IconMdiBrush,
    component: AppearancePane,
  },
  {
    value: SettingsTab.Java,
    label: 'java',
    icon: IconMdiCoffeeOutline,
    component: JavaPane,
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
    value: SettingsTab.ContentProviders,
    label: 'contentProviders',
    icon: IconMdiPackageVariant,
    component: ContentProvidersPane,
  },
  {
    value: SettingsTab.Storage,
    label: 'storage',
    icon: IconMdiDatabase,
    component: StoragePane,
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
] as const satisfies TabConfig<SettingsTab, SettingsPaneProps>[];
