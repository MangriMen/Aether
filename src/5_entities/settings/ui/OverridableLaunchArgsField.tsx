import {
  createMemo,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { CombinedTextFieldProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTextField } from '@/shared/ui';

import { OverrideCheckbox } from './OverrideCheckbox';

export type OverridableLaunchArgsFieldProps = Omit<
  ComponentProps<'div'>,
  'onChange' | 'onBlur'
> & {
  value: string | null | undefined;
  defaultValue?: string;
  onChange?: (value: string | null) => void;
  inputProps?: CombinedTextFieldProps['inputProps'];
  overridable?: boolean;
  onOverrideChange?: (value: string | null) => void;
};

export const OverridableLaunchArgsField: Component<
  OverridableLaunchArgsFieldProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'defaultValue',
    'onChange',
    'inputProps',
    'overridable',
    'onOverrideChange',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const value = createMemo(() => local.value ?? local.defaultValue ?? '');

  const isOverride = createMemo(() => local.value !== null);

  return (
    <div class={cn('flex flex-col gap-1', local.class)} {...others}>
      <span class='text-lg font-medium'>
        {t('instanceSettings.javaArguments')}
      </span>
      <Show when={local.overridable}>
        <OverrideCheckbox
          class='mb-1'
          label={t('instanceSettings.customArguments')}
          enabledValue={value}
          disabledValue={() => null}
          checked={isOverride()}
          onOverrideChange={local.onOverrideChange}
        />
      </Show>
      <CombinedTextField
        disabled={!isOverride()}
        value={value()}
        onChange={local.onChange}
        inputProps={{
          type: 'text',
          placeholder: t('instanceSettings.enterArguments'),
          ...local.inputProps,
        }}
      />
    </div>
  );
};
