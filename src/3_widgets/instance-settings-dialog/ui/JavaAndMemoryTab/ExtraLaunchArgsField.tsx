import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Checkbox, CombinedTextField, LabeledField } from '@/shared/ui';
import {
  createEffect,
  createMemo,
  createSignal,
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

  const [isCustom, setIsCustom] = createSignal(false);

  createEffect(() => {
    if (local.value) {
      setIsCustom(true);
    }
  });

  const disabled = createMemo(() => !isCustom());

  const handleChangeIsCustom = (isChecked: boolean) => {
    setIsCustom(isChecked);
    local.onBlur?.(isChecked ? (local.value ?? null) : null);
  };

  return (
    <LabeledField
      class={cn('text-base', local.class)}
      label={t('instanceSettings.javaArguments')}
      {...others}
    >
      <Checkbox
        label={t('instanceSettings.customArguments')}
        checked={isCustom()}
        onChange={handleChangeIsCustom}
      />
      <CombinedTextField
        disabled={disabled()}
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
