import { Show, type Component } from 'solid-js';

import type { SettingsEntryProps } from '@/shared/ui';

import { useThemeContext, useTranslation } from '@/shared/model';
import { SettingsEntry, Switch, SwitchControl, SwitchThumb } from '@/shared/ui';

export type DisableAnimationsEntryProps = SettingsEntryProps;

export const DisableAnimationsEntry: Component<DisableAnimationsEntryProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const [themeContext, { setDisableAnimations }] = useThemeContext();

  const handleSetDisableAnimations = (value: boolean) => {
    setDisableAnimations(!value);
  };

  return (
    <SettingsEntry
      title={t('settings.animations')}
      description={
        <div class='gap-2 flex flex-col'>
          <span>{t('settings.animationsDescription')}</span>
          <Show when={themeContext.prefersReducedMotion}>
            <span class='rounded-md bg-muted p-2 text-muted-foreground'>
              {t('settings.prefersReducedMotionInfo')}
            </span>
          </Show>
        </div>
      }
      {...props}
    >
      <Switch
        checked={!themeContext.disableAnimations}
        onChange={handleSetDisableAnimations}
      >
        <SwitchControl>
          <SwitchThumb />
        </SwitchControl>
      </Switch>
    </SettingsEntry>
  );
};
