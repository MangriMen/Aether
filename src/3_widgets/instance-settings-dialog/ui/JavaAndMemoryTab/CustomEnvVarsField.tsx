import { cn, useIsCustomCheckbox } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import type { CombinedTextFieldProps } from '@/shared/ui';
import { Checkbox, CombinedTextField, LabeledField } from '@/shared/ui';
import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

export type CustomEnvVarsFieldProps = Omit<
  ComponentProps<'div'>,
  'onChange' | 'onBlur'
> & {
  value: string | null | undefined;
  inputProps?: CombinedTextFieldProps['inputProps'];
  onChange?: (value: string | null) => void;
  onIsCustomChange?: (value: string | null) => void;
};

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
        label={t('instanceSettings.customEnvironmentVariables')}
        checked={isCustom()}
        onChange={setIsCustom}
      />
      <CombinedTextField
        disabled={!isCustom()}
        value={value()}
        onChange={local.onChange}
        inputProps={{
          ...local.inputProps,
          type: 'text',
          placeholder: t('instanceSettings.enterVariables'),
        }}
      />
    </LabeledField>
  );
};
