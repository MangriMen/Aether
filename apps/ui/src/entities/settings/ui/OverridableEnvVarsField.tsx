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

export type OverridableEnvVarsFieldProps = Omit<
  ComponentProps<'div'>,
  'onChange' | 'onBlur'
> & {
  value: string | null | undefined;
  defaultValue?: string;
  onChange?: (value: string | null) => void;
  errorMessage?: string;
  inputProps?: CombinedTextFieldProps['inputProps'];
  overridable?: boolean;
  onOverrideChange?: (value: boolean) => void;
  isOverridden?: boolean;
};

export const OverridableEnvVarsField: Component<
  OverridableEnvVarsFieldProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'defaultValue',
    'onChange',
    'errorMessage',
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
    <div class={cn('flex flex-col gap-1', local.class)} {...others}>
      <InheritanceLabel
        label={t('instanceSettings.environmentVariables')}
        inheritanceLabel={t('settings.usedFromDefaultSettings')}
        isInheritance={isInheritance()}
      />

      <Show when={local.overridable && local.isOverridden !== undefined}>
        <OverrideCheckbox
          class='mb-1'
          label={t('instanceSettings.customEnvironmentVariables')}
          checked={local.isOverridden}
          onOverrideChange={local.onOverrideChange}
        />
      </Show>

      <CombinedTextField
        disabled={isInheritance()}
        value={value()}
        onChange={local.onChange}
        errorMessage={local.errorMessage}
        inputProps={{
          type: 'text',
          placeholder: t('instanceSettings.enterVariables'),
          ...local.inputProps,
        }}
      />
    </div>
  );
};
