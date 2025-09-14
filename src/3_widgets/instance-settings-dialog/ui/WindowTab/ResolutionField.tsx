import type { PhysicalSize } from '@tauri-apps/api/window';

import { Icon } from '@iconify-icon/solid';
import MdiMenuDownIcon from '@iconify/icons-mdi/menu-down';
import MdiMonitorIcon from '@iconify/icons-mdi/monitor-screenshot';
import {
  Field,
  type FormStore,
  getValue,
  setValues,
  validate,
} from '@modular-forms/solid';
import { currentMonitor } from '@tauri-apps/api/window';
import {
  type Component,
  type ComponentProps,
  createMemo,
  createSignal,
  For,
  onMount,
  splitProps,
} from 'solid-js';

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

import type { WindowSchemaValues } from '../../model';

import { RESOLUTION_FIELD_CLASS, RESOLUTION_OPTIONS } from '../../model/window';

export type ResolutionFieldProps = {
  defaultHeight?: number;
  defaultWidth?: number;
  disabled?: boolean;
  errorMessage?: string;
  form: FormStore<WindowSchemaValues>;
  onSubmit?: (width: number, height: number) => void;
} & Omit<ComponentProps<'div'>, 'onChange' | 'onSubmit'>;

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
        height: height.toString(),
        width: width.toString(),
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
          <Field name='resolution.width' of={local.form}>
            {(field, inputProps) => {
              return (
                <CombinedTextField
                  class={RESOLUTION_FIELD_CLASS}
                  disabled={local.disabled}
                  errorMessage={translateError(t, field.error)}
                  inputProps={{
                    maxLength: 5,
                    type: 'text',
                    ...inputProps,
                    onBlur: (e) => {
                      inputProps.onBlur(e);
                      handleResolutionSubmit();
                    },
                  }}
                  value={field.value ?? local.defaultWidth?.toString()}
                />
              );
            }}
          </Field>
          <span class='mt-1 text-xl text-muted-foreground'>&times;</span>
          <Field name='resolution.height' of={local.form}>
            {(field, inputProps) => (
              <CombinedTextField
                class={RESOLUTION_FIELD_CLASS}
                disabled={local.disabled}
                errorMessage={translateError(t, field.error)}
                inputProps={{
                  maxLength: 5,
                  type: 'text',
                  ...inputProps,
                  onBlur: (e) => {
                    inputProps.onBlur(e);
                    handleResolutionSubmit();
                  },
                }}
                value={field.value ?? local.defaultHeight?.toString()}
              />
            )}
          </Field>
        </div>
        <DropdownMenu onOpenChange={handleDropdownOpenChange}>
          <DropdownMenuTrigger
            as={IconButton}
            class='relative flex size-10 flex-col gap-0'
            disabled={local.disabled}
            variant='secondary'
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
