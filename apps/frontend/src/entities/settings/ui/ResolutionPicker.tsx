import type { FormStore } from '@formisch/solid';

import { Field, setInput, getInput } from '@formisch/solid';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import type { WindowSettingsSchema } from '@/features/instance-settings/window';
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
  form: FormStore<typeof WindowSettingsSchema>;
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
  if (!error) {
    return error;
  }

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
    setInput(local.form, {
      path: ['resolution', 'width'],
      input: width.toString(),
    });
    setInput(local.form, {
      path: ['resolution', 'height'],
      input: height.toString(),
    });
    handleResolutionSubmit();
  };

  const handleResolutionSubmit = () => {
    const width = getInput(local.form, {
      path: ['resolution', 'width'],
    });
    const height = getInput(local.form, {
      path: ['resolution', 'height'],
    });

    if (!width || !height) {
      return;
    }

    local.onSubmit?.(Number(width), Number(height));
  };

  const getWidthDisplayValue = (value: string | undefined) => {
    const defaultWidth = local.defaultWidth?.toString() ?? '';

    if (local.disabled && local.forceDefaultValuesOnDisabled) {
      return defaultWidth;
    }

    return value ?? defaultWidth;
  };

  const getHeightDisplayValue = (value: string | undefined) => {
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
      <div class='gap-2 flex flex-col'>
        <div
          class='
            h-9 rounded-md bg-card/card pr-2
            focus-within:bg-background focus-within:ring-ring
            flex w-min items-center border
            focus-within:ring-2
          '
        >
          <div class='flex'>
            <Field of={local.form} path={['resolution', 'width']}>
              {(field) => (
                <ResolutionInput
                  value={getWidthDisplayValue(
                    field.input as string | undefined,
                  )}
                  disabled={local.disabled}
                  inputProps={{
                    ...field.props,
                    onBlur: (e) => {
                      field.props.onBlur?.(e);
                      handleResolutionSubmit();
                    },
                  }}
                />
              )}
            </Field>

            <span class='mt-1 text-xl text-muted-foreground select-none'>
              &times;
            </span>

            <Field of={local.form} path={['resolution', 'height']}>
              {(field) => (
                <ResolutionInput
                  value={getHeightDisplayValue(
                    field.input as string | undefined,
                  )}
                  disabled={local.disabled}
                  inputProps={{
                    ...field.props,
                    onBlur: (e) => {
                      field.props.onBlur?.(e);
                      handleResolutionSubmit();
                    },
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

        <Field of={local.form} path={['resolution', 'width']}>
          {(width) => (
            <Field of={local.form} path={['resolution', 'height']}>
              {(height) => (
                <StandaloneTextFieldErrorMessage
                  showError={
                    Boolean(width.errors?.length) ||
                    Boolean(height.errors?.length)
                  }
                >
                  {translateError(t, width.errors?.[0] || height.errors?.[0])}
                </StandaloneTextFieldErrorMessage>
              )}
            </Field>
          )}
        </Field>
      </div>
    </LabeledField>
  );
};
