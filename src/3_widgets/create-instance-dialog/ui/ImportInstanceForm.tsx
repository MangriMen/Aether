import type { DialogRootProps } from '@kobalte/core/dialog';
import type { SubmitHandler } from '@modular-forms/solid';
import type { Accessor, Component, ComponentProps } from 'solid-js';

import FileFindOutlineIcon from '@iconify/icons-mdi/file-find-outline';
import {
  createForm,
  getValue,
  reset,
  setValue,
  zodForm,
} from '@modular-forms/solid';
import { open } from '@tauri-apps/plugin-dialog';
import { createEffect, createMemo, For, splitProps } from 'solid-js';

import { type ImportHandler, useImportInstance } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  Button,
  CombinedTooltip,
  DialogFooter,
  IconButton,
  LabeledField,
  TextField,
  TextFieldErrorMessage,
  TextFieldInput,
  TextFieldLabel,
  ToggleGroup,
  ToggleGroupItem,
} from '@/shared/ui';

import type { ImportInstanceValues } from '../model';

import { ImportInstanceSchema } from '../model';

export type ImportInstanceFormProps = {
  importConfigs: Accessor<ImportHandler[] | undefined>;
} & Omit<ComponentProps<'form'>, 'onSubmit'> &
  Pick<DialogRootProps, 'onOpenChange'>;

export const ImportInstanceForm: Component<ImportInstanceFormProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'importConfigs',
    'onOpenChange',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const [form, { Field, Form }] = createForm({
    validate: zodForm(ImportInstanceSchema),
  });

  createEffect(() => {
    const configs = local.importConfigs();

    if (!configs?.length) {
      return;
    }

    reset(form, {
      initialValues: {
        packType: configs[0].packType,
      },
    });
  });

  const currentImportHandler = createMemo(() =>
    local
      .importConfigs()
      ?.find((h) => h.packType === getValue(form, 'packType')),
  );

  const { mutateAsync: importInstance } = useImportInstance();
  const handleSubmit: SubmitHandler<ImportInstanceValues> = (values) => {
    importInstance(values);
    local.onOpenChange?.(false);
  };

  const onBrowse = async () => {
    const importHandler = currentImportHandler();

    if (!importHandler) {
      return;
    }

    const file = await open({
      directory: false,
      filters: [
        {
          extensions: importHandler.fileExtensions,
          name: importHandler.fileName,
        },
      ],
      multiple: false,
    });

    if (file) {
      setValue(form, 'path', file);
    }
  };

  return (
    <Form
      class={cn('flex flex-col gap-4', local.class)}
      onSubmit={handleSubmit}
      {...others}
    >
      <LabeledField label={t('common.type')}>
        <Field name='packType'>
          {(field) => (
            <ToggleGroup
              class='justify-start'
              onChange={(value) => {
                if (value) {
                  setValue(form, 'packType', value);
                }
              }}
              value={field.value}
            >
              <For each={local.importConfigs()}>
                {(handler) => (
                  <ToggleGroupItem value={handler.packType}>
                    {handler.title}
                  </ToggleGroupItem>
                )}
              </For>
            </ToggleGroup>
          )}
        </Field>
      </LabeledField>
      <div class='flex w-full items-end gap-4'>
        <Field name='path'>
          {(field, props) => (
            <TextField
              class='flex w-full flex-col gap-2'
              validationState={field.error ? 'invalid' : 'valid'}
            >
              <TextFieldLabel class='relative flex flex-col gap-2'>
                {currentImportHandler()?.fieldLabel}
                <TextFieldInput
                  autocomplete='off'
                  class='pr-10'
                  required
                  type='text'
                  value={field.value ?? ''}
                  {...props}
                />
                <CombinedTooltip
                  as={IconButton}
                  class='absolute bottom-1 right-1 aspect-square h-3/6 p-0'
                  icon={FileFindOutlineIcon}
                  label={t('common.browse')}
                  onClick={onBrowse}
                  type='button'
                  variant='secondary'
                />
              </TextFieldLabel>
              <TextFieldErrorMessage>{field.error}</TextFieldErrorMessage>
            </TextField>
          )}
        </Field>
      </div>
      <DialogFooter class='mt-auto'>
        <Button type='submit'>{t('createInstance.import')}</Button>
        <Button onClick={() => props.onOpenChange?.(false)} variant='secondary'>
          {t('common.cancel')}
        </Button>
      </DialogFooter>
    </Form>
  );
};
