import { createForm, setInput, Form, Field } from '@formisch/solid';
import { open } from '@tauri-apps/plugin-dialog';
import { splitProps, type Component, type JSX } from 'solid-js';

import type { ImportInstance } from '@/features/instance';
import type {
  ImporterCapabilityMetadata,
  PluginMetadata,
} from '@/features/plugins';

import { useImportInstance } from '@/features/instance';
import { BrowseTrailingIcon, CombinedTextField } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { useEnsurePluginAccess } from '../lib/useEnsurePluginAccess';
import {
  ImportInstanceSchema,
  type ImportInstanceSchemaOutput,
} from '../model/importInstanceValidation';

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

  const form = createForm({
    schema: ImportInstanceSchema,
  });

  const importInstance = useImportInstance();
  const ensurePluginAccess = useEnsurePluginAccess();

  const handleSubmit = async (values: ImportInstanceSchemaOutput) => {
    // The importer reads the file or downloads the URL: the plugin needs the folder or the host
    // first, and the user's yes for it. A path typed by hand is asked about just like a picked
    // one; a no, or a failed grant, starts no import.
    if (!(await ensurePluginAccess(local.pluginId, values.path))) {
      return;
    }

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

      // Asked right after the pick, while the user knows which file it was: the plugin is to
      // read the folder it lies in, which also holds the rest of the pack. A no is asked again
      // on submit.
      void ensurePluginAccess(local.pluginId, file);
    }
  };

  return (
    <Form
      of={form}
      class={cn('gap-4 flex h-full grow flex-col', local.class)}
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
            trailingIcons={<BrowseTrailingIcon onBrowse={handleBrowse} />}
          />
        )}
      </Field>
      <div class='mt-auto'>{local.footerButtons}</div>
    </Form>
  );
};
