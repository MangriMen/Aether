import { useThemeContext, useTranslation } from '@/shared/model';
import { SettingsEntry, Switch, SwitchControl, SwitchThumb } from '@/shared/ui';
import { Show, type Component, type ComponentProps } from 'solid-js';

export type DisableAnimationsEntryProps = ComponentProps<'div'>;

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
        <div class='flex flex-col gap-2'>
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
      <div class='flex min-w-44 flex-col items-end'>
        <Switch
          checked={!themeContext.disableAnimations}
          onChange={handleSetDisableAnimations}
        >
          <SwitchControl>
            <SwitchThumb />
          </SwitchControl>
        </Switch>
      </div>
    </SettingsEntry>
  );
};
