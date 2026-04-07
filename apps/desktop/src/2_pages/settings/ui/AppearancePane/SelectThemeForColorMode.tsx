import type { Component, ComponentProps } from 'solid-js';

import { createMemo, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useThemeContext, useTranslation } from '@/shared/model';
import { FieldLabel } from '@/shared/ui';

import SelectThemeByColorMode from './SelectThemeByColorMode';

export type SelectThemeForColorModeProps = ComponentProps<'div'>;

const SelectThemeForColorMode: Component<SelectThemeForColorModeProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  const [themeContext] = useThemeContext();

  const isSystemColorMode = createMemo(
    () => themeContext.rawTheme === 'system',
  );

  return (
    <div
      class={cn(
        'grid grid-cols-[max-content_auto] items-center gap-x-4 gap-y-2',
        local.class,
      )}
      {...others}
    >
      <FieldLabel class='text-sm'>{t('settings.light')}</FieldLabel>
      <SelectThemeByColorMode
        colorMode='light'
        disabled={!isSystemColorMode()}
      />

      <FieldLabel class='text-sm'>{t('settings.dark')}</FieldLabel>
      <SelectThemeByColorMode
        colorMode='dark'
        disabled={!isSystemColorMode()}
      />
    </div>
  );
};

export default SelectThemeForColorMode;
