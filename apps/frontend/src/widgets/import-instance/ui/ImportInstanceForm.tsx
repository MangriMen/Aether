import { createForm, setInput, Form, Field } from '@formisch/solid';
import { open } from '@tauri-apps/plugin-dialog';
import IconMdiFileFindOutline from '~icons/mdi/file-find-outline';
import { splitProps, type Component, type JSX } from 'solid-js';

import { useInstallPack } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTextField, CombinedTooltip, IconButton } from '@/shared/ui';

import {
  ImportInstanceSchema,
  type ImportInstanceSchemaOutput,
} from '../model';

export type ImportInstanceFormProps = {
  pluginId: string;
  capabilityId: string;
  fieldLabel: string | null;
  supportedExtensions: string[];
  footerButtons: JSX.Element;
  onSubmit?: () => void;
  class?: string;
};

export const ImportInstanceForm: Component<ImportInstanceFormProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'pluginId',
    'capabilityId',
    'fieldLabel',
    'supportedExtensions',
    'footerButtons',
    'onSubmit',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const form = createForm({
    schema: ImportInstanceSchema,
  });

  const installPack = useInstallPack();

  const handleSubmit = (values: ImportInstanceSchemaOutput) => {
    installPack.mutateAsync({
      packSource: {
        source: values.path,
      },
      providerId: {
        pluginId: local.pluginId,
        capabilityId: local.capabilityId,
      },
    });

    local.onSubmit?.();
  };

  const handleBrowse = async () => {
    const extensions =
      local.supportedExtensions.length > 0
        ? local.supportedExtensions
        : undefined;

    const file = await open({
      multiple: false,
      directory: false,
      filters: extensions
        ? [{ name: local.fieldLabel ?? '', extensions }]
        : undefined,
    });

    if (file) {
      setInput(form, { path: ['path'], input: file });
    }
  };

  return (
    <Form
      of={form}
      class={cn('flex h-full grow flex-col gap-2', local.class)}
      onSubmit={handleSubmit}
      {...others}
    >
      <Field of={form} path={['path']}>
        {(field) => (
          <CombinedTextField
            label={local.fieldLabel ?? t('createInstance.import')}
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
                  class='size-7 bg-secondary/secondary enabled:hover:bg-secondary/hover'
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
