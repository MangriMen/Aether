import FileFindOutlineIcon from '@iconify/icons-mdi/file-find-outline';

import type { Accessor, Component, ComponentProps } from 'solid-js';
import { createEffect, createMemo, For, splitProps } from 'solid-js';

import type { DialogRootProps } from '@kobalte/core/dialog';
import type { SubmitHandler } from '@modular-forms/solid';
import {
  createForm,
  getValue,
  reset,
  setValue,
  zodForm,
} from '@modular-forms/solid';
import type { ImportInstanceValues } from '../model';
import { ImportInstanceSchema } from '../model';
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
import { cn } from '@/shared/lib';
import { open } from '@tauri-apps/plugin-dialog';
import type { ImportHandler } from '@/entities/instances';
import { importInstance } from '@/entities/instances';
import { useTranslate } from '@/shared/model';

export type ImportInstanceFormProps = Omit<ComponentProps<'form'>, 'onSubmit'> &
  Pick<DialogRootProps, 'onOpenChange'> & {
    importHandlers: Accessor<ImportHandler[]>;
  };

export const ImportInstanceForm: Component<ImportInstanceFormProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'importHandlers',
    'onOpenChange',
    'class',
  ]);

  const [{ t }] = useTranslate();

  const [form, { Form, Field }] = createForm({
    validate: zodForm(ImportInstanceSchema),
  });

  createEffect(() => {
    const importConfigs = local.importHandlers();

    if (!importConfigs.length) {
      return;
    }

    reset(form, {
      initialValues: {
        packType: importConfigs[0].packType,
      },
    });
  });

  const currentImportHandler = createMemo(() =>
    local
      .importHandlers()
      .find((h) => h.packType === getValue(form, 'packType')),
  );

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
      <LabeledField label={t('common.type')}>
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
              <For each={local.importHandlers()}>
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
                {currentImportHandler()?.fieldLabel}
                <TextFieldInput
                  value={field.value ?? ''}
                  type='text'
                  class='pr-10'
                  required
                  autocomplete='off'
                  {...props}
                />
                <CombinedTooltip
                  label={t('common.browse')}
                  as={IconButton}
                  class='absolute bottom-1 right-1 aspect-square h-3/6 p-0'
                  variant='secondary'
                  type='button'
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
        <Button type='submit'>{t('createInstance.import')}</Button>
        <Button variant='secondary' onClick={() => props.onOpenChange?.(false)}>
          {t('common.cancel')}
        </Button>
      </DialogFooter>
    </Form>
  );
};
