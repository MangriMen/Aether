import { createForm, setInput, Form, Field } from '@formisch/solid';
import { open } from '@tauri-apps/plugin-dialog';
import IconMdiFileFindOutline from '~icons/mdi/file-find-outline';
import { splitProps, type Component, type JSX } from 'solid-js';

import type {
  ImportInstance,
  ImporterCapabilityMetadata,
} from '@/entities/instances';
import type { PluginMetadata } from '@/entities/plugins';

import { useImportInstance } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTextField, CombinedTooltip, IconButton } from '@/shared/ui';

import {
  ImportInstanceSchema,
  type ImportInstanceSchemaOutput,
} from '../model';

export type ImportInstanceFormProps = {
  pluginId: PluginMetadata['id'];
  importer: ImporterCapabilityMetadata;
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

  const form = createForm({
    schema: ImportInstanceSchema,
  });

  const importInstance = useImportInstance();

  const handleSubmit = (values: ImportInstanceSchemaOutput) => {
    const dto: ImportInstance = {
      pluginId: local.pluginId,
      importerId: local.importer.id,
      path: values.path,
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
      setInput(form, { path: ['path'], input: file });
    }
  };

  return (
    <Form
      of={form}
      class={cn('flex h-full grow flex-col', local.class)}
      onSubmit={handleSubmit}
      {...others}
    >
      <Field of={form} path={['path']}>
        {(field) => (
          <CombinedTextField
            label={local.importer.fieldLabel}
            name='path'
            value={field.input}
            errorMessage={field.errors?.[0]}
            inputProps={{
              type: 'text',
              class: 'pr-9',
              ...field.props,
            }}
            trailingIcons={
              <div class='mr-1 flex h-full items-center justify-center'>
                <CombinedTooltip
                  label={t('common.browse')}
                  as={IconButton}
                  variant='ghost'
                  class='
                    size-7 bg-secondary/secondary
                    enabled:hover:bg-secondary/hover
                  '
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
