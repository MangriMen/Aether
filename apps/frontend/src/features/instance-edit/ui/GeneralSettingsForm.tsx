import type { Accessor } from 'solid-js';

import { setInput, getInput, Form, Field } from '@formisch/solid';
import { open } from '@tauri-apps/plugin-dialog';
import {
  createSignal,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { logError } from '@/shared/log';
import { CombinedTextField } from '@/shared/ui-kit';
import { cn, noop } from '@/shared/utils';

import type {
  GeneralSettingsSchemaInput,
  GeneralSettingsSchemaOutput,
} from '../model/generalValidation';

import {
  useGeneralSettingsForm,
  useResetGeneralSettingsFormValues,
} from '../lib/useGeneralSettingsForm';
import {
  validateInstanceName,
  type InstanceNameValidation,
} from '../model/nameValidation';
import { InstanceIconDropdownButton } from './InstanceIconDropdownButton';

export type GeneralSettingsFormProps = Omit<
  ComponentProps<'form'>,
  'onSubmit' | 'children'
> & {
  realIconSrc?: string;
  initialValues: Accessor<Partial<GeneralSettingsSchemaInput> | undefined>;
  onChangePartial?: (values: Partial<GeneralSettingsSchemaOutput>) => void;
};

export const GeneralSettingsForm: Component<GeneralSettingsFormProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'realIconSrc',
    'initialValues',
    'onChangePartial',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const form = useGeneralSettingsForm();
  useResetGeneralSettingsFormValues(form, () => local.initialValues());

  const [nameErrorKey, setNameErrorKey] =
    createSignal<
      Extract<InstanceNameValidation, { valid: false }>['errorKey']
    >();

  // The name is saved as the field loses focus: only a valid, changed one is sent.
  const handleNameBlur = () => {
    const input = getInput(form, { path: ['name'] }) ?? '';
    const result = validateInstanceName(input);

    if (!result.valid) {
      setNameErrorKey(result.errorKey);

      const previousName = local.initialValues()?.name;

      if (previousName !== undefined) {
        setInput(form, { path: ['name'], input: previousName });
      }

      return;
    }

    setNameErrorKey(undefined);

    if (result.name !== local.initialValues()?.name) {
      local.onChangePartial?.({ name: result.name });
    }
  };

  const handleSelectIcon = async () => {
    let file: string | null;

    try {
      file = await open({
        multiple: false,
        directory: false,
        filters: [
          {
            name: 'Image',
            extensions: ['png', 'jpg', 'jpeg', 'svg', 'webp', 'gif'],
          },
        ],
      });
    } catch (error) {
      logError('Failed to pick an icon:', error);
      return;
    }

    if (file === null || !file) {
      return;
    }

    setInput(form, { path: ['icon'], input: file });
    local.onChangePartial?.({ icon: file });
  };

  const handleRemoveIcon = () => {
    setInput(form, { path: ['icon'], input: null });
    local.onChangePartial?.({ icon: null });
  };

  return (
    <Form
      of={form}
      class={cn('flex flex-col', local.class)}
      onSubmit={noop}
      {...others}
    >
      <div class='gap-4 flex'>
        <Field of={form} path={['icon']}>
          {(_) => (
            <InstanceIconDropdownButton
              src={local.realIconSrc}
              onSelectIcon={handleSelectIcon}
              onRemoveIcon={handleRemoveIcon}
            />
          )}
        </Field>
        <Field of={form} path={['name']}>
          {(field) => (
            <CombinedTextField
              class='grow'
              label={t('common.name')}
              value={(field.input as string) ?? ''}
              errorMessage={nameErrorKey() ? t(nameErrorKey()!) : undefined}
              inputProps={{
                type: 'text',
                maxLength: 64,
                ...field.props,
                onInput: (e) => {
                  field.props.onInput?.(e);

                  // Drop the stale error as soon as the typed name is valid.
                  if (validateInstanceName(e.currentTarget.value).valid) {
                    setNameErrorKey(undefined);
                  }
                },
                onBlur: () => {
                  field.props.onBlur?.();
                  handleNameBlur();
                },
              }}
            />
          )}
        </Field>
      </div>
    </Form>
  );
};
