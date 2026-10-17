import type { Accessor } from 'solid-js';

import { setInput, getInput, Form, Field } from '@formisch/solid';
import { open } from '@tauri-apps/plugin-dialog';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { CombinedTextField } from '@/shared/ui-kit';
import { cn, noop } from '@/shared/utils';

import type { GeneralSettingsSchemaInput } from '../model/generalValidation';

import {
  useGeneralSettingsForm,
  useResetGeneralSettingsFormValues,
} from '../lib/useGeneralSettingsForm';
import { type GeneralSettingsSchemaOutput } from '../model/generalValidation';
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

  const handleSelectIcon = async () => {
    const file = await open({
      multiple: false,
      directory: false,
      filters: [
        {
          name: 'Image',
          extensions: ['png', 'jpg', 'jpeg', 'svg', 'webp', 'gif'],
        },
      ],
    });

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
              errorMessage={field.errors?.[0]}
              inputProps={{
                type: 'text',
                maxLength: 64,
                ...field.props,
                onBlur: () => {
                  field.props.onBlur?.();
                  local.onChangePartial?.({
                    name: getInput(form, { path: ['name'] }),
                  });
                },
              }}
            />
          )}
        </Field>
      </div>
    </Form>
  );
};
