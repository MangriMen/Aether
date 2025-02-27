import FileFindOutlineIcon from '@iconify/icons-mdi/file-find-outline';

import type { Component, ComponentProps } from 'solid-js';
import { createMemo, For, splitProps } from 'solid-js';

import type { DialogRootProps } from '@kobalte/core/dialog';
import type { SubmitHandler } from '@modular-forms/solid';
import { createForm, getValue, setValue, zodForm } from '@modular-forms/solid';
import type { ImportInstanceValues } from '../model';
import { ImportInstanceSchema } from '../model';
import {
  Button,
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
import { cn } from '@/shared/lib';
import { open } from '@tauri-apps/plugin-dialog';
import { importInstance, useImportHandlers } from '@/entities/instances';
import { useTranslate } from '@/shared/model';

export type ImportInstanceProps = Omit<ComponentProps<'form'>, 'onSubmit'> &
  Pick<DialogRootProps, 'onOpenChange'>;

export const ImportInstance: Component<ImportInstanceProps> = (props) => {
  const [local, others] = splitProps(props, ['onOpenChange', 'class']);

  const importHandlers = useImportHandlers();

  const [{ t }] = useTranslate();

  const [form, { Form, Field }] = createForm({
    initialValues: { packType: importHandlers()[0].packType },
    validate: zodForm(ImportInstanceSchema),
  });

  const currentImportHandler = createMemo(
    () =>
      importHandlers().find((h) => h.packType === getValue(form, 'packType')) ??
      importHandlers()[0],
  );

  const handleSubmit: SubmitHandler<ImportInstanceValues> = (values) => {
    importInstance(values);
    local.onOpenChange?.(false);
  };

  const onBrowse = async () => {
    const importHandler = currentImportHandler();

    const file = await open({
      multiple: false,
      directory: false,
      filters: [
        {
          name: importHandler.fileName,
          extensions: importHandler.fileExtensions,
        },
      ],
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
      <LabeledField label='Type'>
        <Field name='packType'>
          {(field) => (
            <ToggleGroup
              class='justify-start'
              value={field.value}
              onChange={(value) => {
                if (value) {
                  setValue(form, 'packType', value);
                }
              }}
            >
              <For each={importHandlers()}>
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
              validationState={field.error ? 'invalid' : 'valid'}
              class='flex w-full flex-col gap-2'
            >
              <TextFieldLabel class='relative flex flex-col gap-2'>
                {currentImportHandler().fieldLabel}
                <TextFieldInput
                  value={field.value}
                  type='text'
                  class='pr-10'
                  required
                  autocomplete='off'
                  {...props}
                />
                <IconButton
                  class='absolute bottom-1 right-1 aspect-square h-3/6 p-0'
                  variant='secondary'
                  type='button'
                  title='Browse'
                  onClick={onBrowse}
                  icon={FileFindOutlineIcon}
                />
              </TextFieldLabel>
              <TextFieldErrorMessage>{field.error}</TextFieldErrorMessage>
            </TextField>
          )}
        </Field>
      </div>
      <DialogFooter class='mt-auto'>
        <Button type='submit'>Import</Button>
        <Button variant='secondary' onClick={() => props.onOpenChange?.(false)}>
          {t('common.cancel')}
        </Button>
      </DialogFooter>
    </Form>
  );
};
