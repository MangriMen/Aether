export type HookFieldKey = 'preLaunch' | 'wrapper' | 'postExit';

export const FIELDS_CONFIG = [
  {
    key: 'preLaunch',
    labelKey: 'instanceSettings.hookSettings.preLaunch',
    subLabelKey: 'instanceSettings.hookSettings.preLaunchLabel',
  },
  {
    key: 'wrapper',
    labelKey: 'instanceSettings.hookSettings.wrapper',
    subLabelKey: 'instanceSettings.hookSettings.wrapperLabel',
  },
  {
    key: 'postExit',
    labelKey: 'instanceSettings.hookSettings.postExit',
    subLabelKey: 'instanceSettings.hookSettings.postExitLabel',
  },
] as const satisfies readonly {
  key: HookFieldKey;
  labelKey: string;
  subLabelKey: string;
}[];

export type HookFieldConfig = (typeof FIELDS_CONFIG)[number];
