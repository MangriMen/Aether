import type { Accessor } from 'solid-js';

import { setValue, type PartialValues } from '@modular-forms/solid';
import { open } from '@tauri-apps/plugin-dialog';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn, useFieldOnChangeSync } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTextField } from '@/shared/ui';

import {
  useGeneralSettingsForm,
  useResetGeneralSettingsFormValues,
} from '../lib';
import {
  GeneralSettingsSchema,
  type GeneralSettingsSchemaInput,
  type GeneralSettingsSchemaOutput,
} from '../model';
import { InstanceIconDropdownButton } from './InstanceIconDropdownButton';

export type GeneralSettingsFormProps = Omit<
  ComponentProps<'form'>,
  'onSubmit' | 'children'
> & {
  realIconSrc?: string;
  initialValues: Accessor<
    PartialValues<GeneralSettingsSchemaInput> | undefined
  >;
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

  const [form, { Form, Field }] = useGeneralSettingsForm();
  useResetGeneralSettingsFormValues(form, () => local.initialValues());

  const updateName = useFieldOnChangeSync(
    GeneralSettingsSchema,
    form,
    'name',
    (value) => {
      local.onChangePartial?.({
        name: value,
      });
    },
  );

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

    setValue(form, 'icon', file);
    local.onChangePartial?.({ icon: file });
  };

  const handleRemoveIcon = () => {
    setValue(form, 'icon', null);
    local.onChangePartial?.({ icon: null });
  };

  return (
    <Form class={cn('flex flex-col', local.class)} {...others}>
      <div class='flex gap-4'>
        <Field name='icon' type='string'>
          {(_) => (
            <InstanceIconDropdownButton
              src={local.realIconSrc}
              onSelectIcon={handleSelectIcon}
              onRemoveIcon={handleRemoveIcon}
            />
          )}
        </Field>
        <Field name='name' type='string'>
          {(field, inputProps) => (
            <CombinedTextField
              class='grow'
              label={t('common.name')}
              value={field.value ?? ''}
              errorMessage={field.error}
              inputProps={{
                type: 'text',
                maxLength: 64,
                ...inputProps,
                onBlur: (e) => {
                  inputProps.onBlur(e);
                  updateName();
                },
              }}
            />
          )}
        </Field>
      </div>
    </Form>
  );
};
