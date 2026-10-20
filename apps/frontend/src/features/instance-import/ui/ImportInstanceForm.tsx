import { createForm, setInput, Form, Field } from '@formisch/solid';
import { open } from '@tauri-apps/plugin-dialog';
import {
  createSignal,
  splitProps,
  type Accessor,
  type Component,
  type JSX,
} from 'solid-js';

import type { ImportInstance } from '@/features/instance';
import type {
  ImporterCapabilityMetadata,
  PluginMetadata,
} from '@/features/plugins';

import { useImportInstance } from '@/features/instance';
import { useTranslation } from '@/shared/i18n';
import { logError } from '@/shared/log';
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
  footerButtons: (isSubmitting: Accessor<boolean>) => JSX.Element;
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
  const ensurePluginAccess = useEnsurePluginAccess();

  const [isSubmitting, setIsSubmitting] = createSignal(false);

  const handleSubmit = async (values: ImportInstanceSchemaOutput) => {
    // A second click while the import runs would only produce a DUPLICATE_REQUEST toast.
    if (isSubmitting()) {
      return;
    }

    setIsSubmitting(true);

    try {
      await submit(values);
    } finally {
      setIsSubmitting(false);
    }
  };

  const submit = async (values: ImportInstanceSchemaOutput) => {
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

    try {
      await importInstance.mutateAsync(dto);
    } catch (error) {
      // The mutation shows the error; the form stays open so the path is not lost.
      logError('Failed to import the instance:', error);
      return;
    }

    local.onSubmit?.();
  };

  const handleBrowse = async () => {
    let file: string | null;

    try {
      file = await open({
        multiple: false,
        directory: false,
      });
    } catch (error) {
      logError('Failed to pick a file to import:', error);
      return;
    }

    if (file) {
      setInput(form, { path: ['path'], input: file });

      // Asked right after the pick, while the user knows which file it was: the plugin is to
      // read the folder it lies in, which also holds the rest of the pack. A no is asked again
      // on submit.
      ensurePluginAccess(local.pluginId, file).catch(logError);
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
            errorMessage={
              field.errors?.[0] ? t('validation.pathRequired') : undefined
            }
            inputProps={{
              type: 'text',
              class: 'pr-9',
              ...field.props,
            }}
            trailingIcons={<BrowseTrailingIcon onBrowse={handleBrowse} />}
          />
        )}
      </Field>
      <div class='mt-auto'>{local.footerButtons(isSubmitting)}</div>
    </Form>
  );
};
