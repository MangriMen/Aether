import { cn, useIsCustomCheckbox } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Checkbox, CombinedTextField, LabeledField } from '@/shared/ui';
import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

export type ExtraLaunchArgsFieldProps = Omit<
  ComponentProps<'div'>,
  'onChange' | 'onBlur'
> & {
  value: string | null | undefined;
  defaultValue?: string;
  onChange?: (value: string | null) => void;
  onBlur?: (value: string | null) => void;
};

export const ExtraLaunchArgsField: Component<ExtraLaunchArgsFieldProps> = (
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
    isCustom: () => !!value(),
    onChange: (isCustom) => local.onBlur?.(isCustom ? value() : null),
  });

  return (
    <LabeledField
      class={cn('text-base', local.class)}
      label={t('instanceSettings.javaArguments')}
      {...others}
    >
      <Checkbox
        label={t('instanceSettings.customArguments')}
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
          placeholder: t('instanceSettings.enterArguments'),
          onBlur: (e) => local.onBlur?.(e.target.value),
        }}
      />
    </LabeledField>
  );
};
