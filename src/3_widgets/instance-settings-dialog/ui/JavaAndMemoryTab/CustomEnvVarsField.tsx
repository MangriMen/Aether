import {
  type Component,
  type ComponentProps,
  createMemo,
  splitProps,
} from 'solid-js';

import type { CombinedTextFieldProps } from '@/shared/ui';

import { cn, useIsCustomCheckbox } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Checkbox, CombinedTextField, LabeledField } from '@/shared/ui';

export type CustomEnvVarsFieldProps = {
  inputProps?: CombinedTextFieldProps['inputProps'];
  onChange?: (value: null | string) => void;
  onIsCustomChange?: (value: null | string) => void;
  value: null | string | undefined;
} & Omit<ComponentProps<'div'>, 'onBlur' | 'onChange'>;

export const CustomEnvVarsField: Component<CustomEnvVarsFieldProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'value',
    'onChange',
    'onIsCustomChange',
    'inputProps',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const value = createMemo(() => local.value ?? '');

  const [isCustom, setIsCustom] = useIsCustomCheckbox({
    isCustom: () => local.value !== null,
    onChange: (isCustom) => local.onIsCustomChange?.(isCustom ? value() : null),
  });

  return (
    <LabeledField
      class={cn('text-base', local.class)}
      label={t('instanceSettings.environmentVariables')}
      {...others}
    >
      <Checkbox
        checked={isCustom()}
        label={t('instanceSettings.customEnvironmentVariables')}
        onChange={setIsCustom}
      />
      <CombinedTextField
        disabled={!isCustom()}
        inputProps={{
          ...local.inputProps,
          placeholder: t('instanceSettings.enterVariables'),
          type: 'text',
        }}
        onChange={local.onChange}
        value={value()}
      />
    </LabeledField>
  );
};
