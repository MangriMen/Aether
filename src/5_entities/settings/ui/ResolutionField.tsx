import type { PhysicalSize } from '@tauri-apps/api/window';

import { Icon } from '@iconify-icon/solid';
import MdiMenuDownIcon from '@iconify/icons-mdi/menu-down';
import MdiMonitorIcon from '@iconify/icons-mdi/monitor-screenshot';
import {
  Field,
  getValue,
  setValues,
  validate,
  type FormStore,
} from '@modular-forms/solid';
import { currentMonitor } from '@tauri-apps/api/window';
import {
  createMemo,
  createSignal,
  For,
  onMount,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { WindowSchemaValuesInput } from '@/features/instance-settings/window-settings-form';
import type { TFunction } from '@/shared/model';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  CombinedTextField,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
  LabeledField,
} from '@/shared/ui';

import { RESOLUTION_OPTIONS } from '../model';

export type ResolutionFieldProps = Omit<
  ComponentProps<'div'>,
  'onChange' | 'onSubmit'
> & {
  form: FormStore<WindowSchemaValuesInput>;
  defaultWidth?: number | string;
  defaultHeight?: number | string;
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

export const RESOLUTION_FIELD_CLASS = 'w-[7ch]';

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

export const ResolutionField: Component<ResolutionFieldProps> = (props) => {
  const [local, others] = splitProps(props, [
    'form',
    'defaultWidth',
    'defaultHeight',
    'onSubmit',
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

  const [maxResolution, setMaxResolution] = createSignal<
    PhysicalSize | undefined
  >();

  const updateMaxResolution = async () => {
    const monitor = await currentMonitor();
    setMaxResolution(monitor?.size);
  };

  const handleDropdownOpenChange = (isOpen: boolean) => {
    if (isOpen) {
      updateMaxResolution();
    }
  };

  const filteredResolutions = createMemo(() => {
    const max = maxResolution();

    if (max === undefined) {
      return RESOLUTION_OPTIONS;
    }

    return RESOLUTION_OPTIONS.filter(({ value: [width, height] }) => {
      return width <= max.width && height <= max.height;
    });
  });

  onMount(() => {
    updateMaxResolution();
  });

  return (
    <LabeledField
      class={cn('text-lg', local.class)}
      label={t('instanceSettings.resolution')}
      {...others}
    >
      <div class='flex gap-2'>
        <div class='flex items-start gap-2'>
          <Field of={local.form} name='resolution.width'>
            {(field, inputProps) => {
              return (
                <CombinedTextField
                  class={RESOLUTION_FIELD_CLASS}
                  value={field.value ?? local.defaultWidth?.toString() ?? ''}
                  disabled={local.disabled}
                  errorMessage={translateError(t, field.error)}
                  inputProps={{
                    type: 'text',
                    maxLength: 5,
                    ...inputProps,
                    onBlur: (e) => {
                      inputProps.onBlur(e);
                      handleResolutionSubmit();
                    },
                  }}
                />
              );
            }}
          </Field>
          <span class='mt-1 text-xl text-muted-foreground'>&times;</span>
          <Field of={local.form} name='resolution.height'>
            {(field, inputProps) => (
              <CombinedTextField
                class={RESOLUTION_FIELD_CLASS}
                value={field.value ?? local.defaultHeight?.toString() ?? ''}
                disabled={local.disabled}
                errorMessage={translateError(t, field.error)}
                inputProps={{
                  type: 'text',
                  maxLength: 5,
                  ...inputProps,
                  onBlur: (e) => {
                    inputProps.onBlur(e);
                    handleResolutionSubmit();
                  },
                }}
              />
            )}
          </Field>
        </div>
        <DropdownMenu onOpenChange={handleDropdownOpenChange}>
          <DropdownMenuTrigger
            as={IconButton}
            class='relative flex size-10 flex-col gap-0'
            variant='secondary'
            disabled={local.disabled}
          >
            <Icon class='absolute top-1' icon={MdiMonitorIcon} />
            <Icon class='absolute top-5' icon={MdiMenuDownIcon} />
          </DropdownMenuTrigger>
          <DropdownMenuContent class='max-h-[230px] overflow-auto'>
            <For each={filteredResolutions()}>
              {(option) => (
                <DropdownMenuItem
                  onClick={() => handlePredefinedResolutionSubmit(option.value)}
                >
                  {option.name}
                </DropdownMenuItem>
              )}
            </For>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </LabeledField>
  );
};
