import type { DialogRootProps } from '@kobalte/core/dialog';
import type { SubmitHandler } from '@modular-forms/solid';
import type { Accessor, Component, ComponentProps } from 'solid-js';

import {
  createForm,
  getValue,
  reset,
  setValue,
  zodForm,
} from '@modular-forms/solid';
import { open } from '@tauri-apps/plugin-dialog';
import IconFileFindOutline from '~icons/mdi/file-find-outline';
import { createEffect, createMemo, For, splitProps } from 'solid-js';

import { useImportInstance, type ImportConfig } from '@/entities/instances';
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

export type ImportInstanceFormProps = Omit<ComponentProps<'form'>, 'onSubmit'> &
  Pick<DialogRootProps, 'onOpenChange'> & {
    importConfigs: Accessor<ImportConfig[] | undefined>;
  };

export const ImportInstanceForm: Component<ImportInstanceFormProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'importConfigs',
    'onOpenChange',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const [form, { Form, Field }] = createForm({
    validate: zodForm(ImportInstanceSchema),
  });

  createEffect(() => {
    const configs = local.importConfigs();

    if (!configs?.length) {
      return;
    }

    reset(form, {
      initialValues: {
        pluginId: configs[0].pluginId,
      },
    });
  });

  const currentImportHandler = createMemo(() =>
    local
      .importConfigs()
      ?.find((h) => h.pluginId === getValue(form, 'pluginId')),
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
      setValue(form, 'pathOrUrl', file);
    }
  };

  return (
    <Form
      class={cn('flex flex-col gap-4', local.class)}
      onSubmit={handleSubmit}
      {...others}
    >
      <LabeledField label={t('common.type')}>
        <Field name='pluginId'>
          {(field) => (
            <ToggleGroup
              class='justify-start'
              value={field.value}
              onChange={(value) => {
                if (value) {
                  setValue(form, 'pluginId', value);
                }
              }}
            >
              <For each={local.importConfigs()}>
                {(handler) => (
                  <ToggleGroupItem value={handler.pluginId}>
                    {handler.title}
                  </ToggleGroupItem>
                )}
              </For>
            </ToggleGroup>
          )}
        </Field>
      </LabeledField>
      <div class='flex w-full items-end gap-4'>
        <Field name='pathOrUrl'>
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
                  {...props}
                />
                <CombinedTooltip
                  label={t('common.browse')}
                  as={IconButton}
                  class='absolute bottom-1 right-1 aspect-square h-3/6 p-0'
                  variant='secondary'
                  type='button'
                  onClick={onBrowse}
                  icon={IconFileFindOutline}
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
