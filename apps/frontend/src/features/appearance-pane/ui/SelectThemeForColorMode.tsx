import type { Component, ComponentProps } from 'solid-js';

import { createMemo, splitProps } from 'solid-js';

import { useThemeContext } from '@/features/appearance';
import { useTranslation } from '@/shared/i18n';
import { FieldLabel } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import SelectThemeByColorMode from './SelectThemeByColorMode';

type SelectThemeForColorModeProps = ComponentProps<'div'>;

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
        'gap-x-4 gap-y-2 grid grid-cols-[max-content_auto] items-center',
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
