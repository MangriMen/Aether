import { createForm, setValue, zodForm } from '@modular-forms/solid';
import { open } from '@tauri-apps/plugin-dialog';
import IconMdiFileFindOutline from '~icons/mdi/file-find-outline';
import { splitProps, type Component, type JSX } from 'solid-js';

import type { ImportInstance } from '@/entities/instances';
import type { ImporterCapability, PluginMetadata } from '@/entities/plugins';

import { useImportInstance } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTextField, CombinedTooltip, IconButton } from '@/shared/ui';

import type { ImportInstanceSchemaInput } from '../model';

import { ImportInstanceSchema } from '../model';

export type ImportInstanceFormProps = {
  pluginId: PluginMetadata['id'];
  importer: ImporterCapability;
  footerButtons: JSX.Element;
  onSubmit?: () => void;
  class?: string;
};

export const ImportInstanceForm: Component<ImportInstanceFormProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'pluginId',
    'importer',
    'footerButtons',
    'onSubmit',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const [form, { Form, Field }] = createForm({
    validate: zodForm(ImportInstanceSchema),
  });

  const importInstance = useImportInstance();

  const handleSubmit = async (values: ImportInstanceSchemaInput) => {
    const outValues = ImportInstanceSchema.safeParse(values);

    if (outValues.error) {
      return;
    }

    const dto: ImportInstance = {
      pluginId: local.pluginId,
      importerId: local.importer.id,
      path: outValues.data.path,
    };

    importInstance.mutateAsync(dto);

    local.onSubmit?.();
  };

  const handleBrowse = async () => {
    const file = await open({
      multiple: false,
      directory: false,
    });

    if (file) {
      setValue(form, 'path', file);
    }
  };

  return (
    <Form
      class={cn('flex flex-col h-full grow', local.class)}
      onSubmit={handleSubmit}
      {...others}
    >
      <Field name='path'>
        {(field, fieldProps) => (
          <CombinedTextField
            label={local.importer.fieldLabel}
            name={field.name}
            value={field.value}
            errorMessage={field.error}
            inputProps={{ type: 'text', class: 'pr-9', ...fieldProps }}
            leadingIcons={
              <div class='mr-1 flex h-full items-center justify-center'>
                <CombinedTooltip
                  label={t('common.browse')}
                  as={IconButton}
                  variant='ghost'
                  class='size-7 bg-secondary-dark/50 enabled:hover:bg-secondary-dark'
                  type='button'
                  size='sm'
                  onClick={handleBrowse}
                  icon={IconMdiFileFindOutline}
                />
              </div>
            }
          />
        )}
      </Field>
      <div class='mt-auto'>{local.footerButtons}</div>
    </Form>
  );
};
