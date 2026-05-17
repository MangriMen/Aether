import IconMdiBrush from '~icons/mdi/brush-variant';
import IconMdiCoffeeOutline from '~icons/mdi/coffee-outline';
import IconMdiGamepadSquare from '~icons/mdi/gamepad-square';
import IconMdiPuzzle from '~icons/mdi/puzzle';
import IconMdiTestTube from '~icons/mdi/test-tube';
import IconMdiUpdate from '~icons/mdi/update';

import type { TabConfig } from '@/shared/model';
import type { SettingsPaneProps } from '@/shared/ui';

import { SettingsTab } from '@/entities/settings';

import { AppearancePane } from '../ui/AppearancePane/AppearancePane';
import { DefaultInstanceSettingsPane } from '../ui/DefaultInstanceSettingsPane/DefaultInstanceSettingsPane';
import { ExperimentalPane } from '../ui/ExperimentalPane/ExperimentalPane';
import { JavaPane } from '../ui/JavaPane/JavaPane';
import { PluginsPane } from '../ui/PluginsPane/PluginsPane';
import { UpdatePane } from '../ui/UpdatePane/UpdatePane';

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
