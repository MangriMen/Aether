import FileFindOutlineIcon from '@iconify/icons-mdi/file-find-outline';
import { Icon } from '@iconify-icon/solid';
import {
  createForm,
  setValue,
  SubmitHandler,
  zodForm,
} from '@modular-forms/solid';
import { open } from '@tauri-apps/plugin-dialog';
import { Component, splitProps } from 'solid-js';
import { z } from 'zod';

import { isAetherLauncherError } from '@/shared/model';
import {
  Button,
  showToast,
  TextField,
  TextFieldErrorMessage,
  TextFieldInput,
  TextFieldLabel,
} from '@/shared/ui';

import { callPlugin } from '@/entities/plugins';

import { PackwizPluginImportMenuProps } from './types';

type PackwizPluginImportData = {
  kind: 'import';
  data: {
    url: string;
  };
};

type PackwizPluginUpdateData = {
  kind: 'update';
  data: {
    instance_id: string;
  };
};

type PackwizPluginData = PackwizPluginImportData | PackwizPluginUpdateData;

const PackwizPluginImportSchema = z.object({
  packPath: z.string().min(1, {
    message: 'Pack path is required',
  }),
});

type PackwizPluginImportValues = z.infer<typeof PackwizPluginImportSchema>;

export const PackwizPluginImportMenu: Component<
  PackwizPluginImportMenuProps
> = (props) => {
  const [local, others] = splitProps(props, ['onSubmit']);

  const [form, { Form, Field }] = createForm({
    validate: zodForm(PackwizPluginImportSchema),
  });

  const handleSubmit: SubmitHandler<PackwizPluginImportValues> = async (
    values,
  ) => {
    const payload: PackwizPluginData = {
      kind: 'import',
      data: {
        url: values.packPath,
      },
    };

    callPlugin('packwiz', JSON.stringify(payload)).catch((error) => {
      if (isAetherLauncherError(error)) {
        showToast({
          description: error.message,
          variant: 'destructive',
        });
      }
    });

    local.onSubmit?.();
  };

  const onBrowse = async () => {
    const file = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'Packwiz modpack', extensions: ['toml'] }],
    });

    if (file) {
      setValue(form, 'packPath', file);
    }
  };

  return (
    <Form class='flex flex-col gap-4' onSubmit={handleSubmit} {...others}>
      <div class='flex w-full items-end gap-4'>
        <Field name='packPath'>
          {(field, props) => (
            <TextField
              validationState={field.error ? 'invalid' : 'valid'}
              class='flex w-full flex-col gap-2'
            >
              <TextFieldLabel class='relative flex flex-col gap-2'>
                Packwiz URL or File
                <TextFieldInput
                  value={field.value}
                  type='text'
                  class='pr-10'
                  required
                  autocomplete='off'
                  {...props}
                />
                <Button
                  class='absolute bottom-1 right-1 aspect-square h-3/6 p-0'
                  variant='secondary'
                  type='button'
                  title='Browse'
                  onClick={onBrowse}
                >
                  <Icon icon={FileFindOutlineIcon} class='text-xl' />
                </Button>
              </TextFieldLabel>
              <TextFieldErrorMessage>{field.error}</TextFieldErrorMessage>
            </TextField>
          )}
        </Field>
      </div>
      <Button type='submit'>Import</Button>
    </Form>
  );
};
