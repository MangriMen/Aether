export const SettingsTab = {
  Appearance: 'appearance',
  Java: 'java',
  DefaultInstanceSettings: 'defaultInstanceSettings',
  Update: 'update',
  Plugins: 'plugins',
  Experimental: 'experimental',
} as const;

export type SettingsTab = (typeof SettingsTab)[keyof typeof SettingsTab];

export const SettingsTabs = new Set(Object.values(SettingsTab));

export const isSettingsTab = (value: string): value is SettingsTab =>
  SettingsTabs.has(value as SettingsTab);

export const DEFAULT_SETTINGS_TAB = SettingsTab.Appearance;

export const resolveSettingsTab = (
  tab: string | undefined,
): SettingsTab | undefined => {
  if (tab === undefined) {
    return;
  }

  return isSettingsTab(tab) ? tab : DEFAULT_SETTINGS_TAB;
};
