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

import { InheritanceLabel } from './InheritanceLabel';
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
  onOverrideChange?: (value: boolean) => void;
  isOverridden?: boolean;
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
    'isOverridden',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const value = createMemo(() => local.value ?? local.defaultValue ?? '');

  const isInheritance = createMemo(
    () => local.overridable && !local.isOverridden,
  );

  return (
    <div class={cn('gap-1 flex flex-col', local.class)} {...others}>
      <InheritanceLabel
        label={t('instanceSettings.javaArguments')}
        inheritanceLabel={t('settings.usedFromDefaultSettings')}
        isInheritance={isInheritance()}
      />
      <Show when={local.overridable && local.isOverridden !== undefined}>
        <OverrideCheckbox
          class='mb-1'
          label={t('instanceSettings.customArguments')}
          checked={local.isOverridden}
          onOverrideChange={local.onOverrideChange}
        />
      </Show>
      <CombinedTextField
        disabled={isInheritance()}
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
