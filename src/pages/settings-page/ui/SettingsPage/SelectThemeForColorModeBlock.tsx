import type { Component, ComponentProps } from 'solid-js';
import { createMemo, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { FieldLabel } from '@/shared/ui';

import { SelectThemeByColorMode } from '@/features/select-color-mode';


import { useThemeContext, useTranslate } from '@/app/model';

export type SelectThemeForColorModeBlockProps = ComponentProps<'div'>;

const SelectThemeForColorModeBlock: Component<
  SelectThemeForColorModeBlockProps
> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslate();

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

export default SelectThemeForColorModeBlock;
