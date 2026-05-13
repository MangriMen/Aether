import type {
  FieldElementProps,
  FieldPath,
  FieldValues,
  Maybe,
} from '@modular-forms/solid';

import {
  Field,
  getValue,
  setValues,
  validate,
  type FormStore,
} from '@modular-forms/solid';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import type { WindowSettingsSchemaInput } from '@/features/instance-settings/window';
import type { TFunction } from '@/shared/model';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { LabeledField, StandaloneTextFieldErrorMessage } from '@/shared/ui';

import { InheritanceLabel } from './InheritanceLabel';
import { ResolutionInput } from './ResolutionInput';
import { ResolutionSelectButton } from './ResolutionSelectButton';

export type ResolutionFieldProps = Omit<
  ComponentProps<'div'>,
  'onChange' | 'onSubmit'
> & {
  form: FormStore<WindowSettingsSchemaInput>;
  defaultWidth?: number | string;
  defaultHeight?: number | string;
  forceDefaultValuesOnDisabled?: boolean;
  disabled?: boolean;
  errorMessage?: string;
  onSubmit?: (width: number, height: number) => void;
};

const knownErrors = [
  'instanceSettings.windowError.resolution.emptyValue',
  'instanceSettings.windowError.resolution.invalidValue',
  'instanceSettings.windowError.resolution.invalidRange',
] as const;

type ResolutionErrorKeys = (typeof knownErrors)[number];

function translateError(
  t: TFunction,
  error: string | undefined,
): string | undefined {
  if (!error) return error;

  if (knownErrors.includes(error as ResolutionErrorKeys)) {
    return t(error as ResolutionErrorKeys);
  }

  return error;
}

export const ResolutionPicker: Component<ResolutionFieldProps> = (props) => {
  const [local, others] = splitProps(props, [
    'form',
    'defaultWidth',
    'defaultHeight',
    'onSubmit',
    'forceDefaultValuesOnDisabled',
    'disabled',
    'errorMessage',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const handlePredefinedResolutionSubmit = ([width, height]: [
    number,
    number,
  ]) => {
    setValues(local.form, {
      resolution: {
        width: width.toString(),
        height: height.toString(),
      },
    });
    handleResolutionSubmit();
  };

  const handleResolutionSubmit = async () => {
    const [isWidthValid, isHeightValid] = await Promise.all([
      validate(local.form, 'resolution.width', { shouldFocus: false }),
      validate(local.form, 'resolution.height', { shouldFocus: false }),
    ]);

    if (!isWidthValid || !isHeightValid) {
      return;
    }

    const width = getValue(local.form, 'resolution.width');
    const height = getValue(local.form, 'resolution.height');

    if (!width || !height) {
      return;
    }

    local.onSubmit?.(Number(width), Number(height));
  };

  const handleBlur = <
    TFieldValues extends FieldValues,
    TFieldName extends FieldPath<TFieldValues>,
  >(
    e: FocusEvent & {
      currentTarget: HTMLInputElement;
      target: HTMLInputElement;
    },
    inputProps: FieldElementProps<TFieldValues, TFieldName>,
  ) => {
    if (typeof inputProps.onBlur === 'function') {
      inputProps.onBlur(e);
    }
    handleResolutionSubmit();
  };

  const getWidthDisplayValue = (value: Maybe<string>) => {
    const defaultWidth = local.defaultWidth?.toString() ?? '';

    if (local.disabled && local.forceDefaultValuesOnDisabled) {
      return defaultWidth;
    }

    return value ?? defaultWidth;
  };

  const getHeightDisplayValue = (value: Maybe<string>) => {
    const defaultHeight = local.defaultHeight?.toString() ?? '';

    if (local.disabled && local.forceDefaultValuesOnDisabled) {
      return defaultHeight;
    }

    return value ?? defaultHeight;
  };

  return (
    <LabeledField
      class={cn('text-lg', local.class)}
      label={
        <InheritanceLabel
          label={t('instanceSettings.resolution')}
          inheritanceLabel={t('settings.usedFromDefaultSettings')}
          isInheritance={local.disabled && local.forceDefaultValuesOnDisabled}
        />
      }
      {...others}
    >
      <div class='flex flex-col gap-2'>
        <div class='flex h-10 w-min items-center rounded-md border bg-card/card pr-2 focus-within:bg-background focus-within:ring-2 focus-within:ring-ring'>
          <div class='flex'>
            <Field of={local.form} name='resolution.width'>
              {(field, inputProps) => (
                <ResolutionInput
                  value={getWidthDisplayValue(field.value)}
                  disabled={local.disabled}
                  inputProps={{
                    ...inputProps,
                    onBlur: (e) => handleBlur(e, inputProps),
                  }}
                />
              )}
            </Field>

            <span class='mt-1 select-none text-xl text-muted-foreground'>
              &times;
            </span>

            <Field of={local.form} name='resolution.height'>
              {(field, inputProps) => (
                <ResolutionInput
                  value={getHeightDisplayValue(field.value)}
                  disabled={local.disabled}
                  inputProps={{
                    ...inputProps,
                    onBlur: (e) => handleBlur(e, inputProps),
                  }}
                />
              )}
            </Field>
          </div>

          <ResolutionSelectButton
            disabled={local.disabled}
            onResolutionChange={handlePredefinedResolutionSubmit}
          />
        </div>

        <Field of={local.form} name='resolution.width'>
          {(width) => (
            <Field of={local.form} name='resolution.height'>
              {(height) => (
                <StandaloneTextFieldErrorMessage
                  showError={Boolean(width.error) || Boolean(height.error)}
                >
                  {translateError(t, width.error || height.error)}
                </StandaloneTextFieldErrorMessage>
              )}
            </Field>
          )}
        </Field>
      </div>
    </LabeledField>
  );
};
