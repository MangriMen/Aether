import { cn, useIsCustomCheckbox } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
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
  defaultValue?: string;
  onChange?: (value: string) => void;
  onBlur?: (value: string) => void;
};

export const CustomEnvVarsField: Component<CustomEnvVarsFieldProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'value',
    'defaultValue',
    'onChange',
    'onBlur',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const value = createMemo(() => local.value ?? '');

  const [isCustom, setIsCustom] = useIsCustomCheckbox({
    value: () => value(),
    resetValue: () => '',
    onOverrideValue: () => local.onBlur,
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
        defaultValue={local.defaultValue}
        onChange={local.onChange}
        inputProps={{
          type: 'text',
          placeholder: t('instanceSettings.enterVariables'),
          onBlur: (e) => local.onBlur?.(e.target.value),
        }}
      />
    </LabeledField>
  );
};
