import FileFindOutlineIcon from '@iconify/icons-mdi/file-find-outline';
import { Icon } from '@iconify-icon/solid';
import { open } from '@tauri-apps/plugin-dialog';
import { Component, createSignal, splitProps } from 'solid-js';

import { Button, TextField, TextFieldInput, TextFieldLabel } from '@/shared/ui';

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

export const PackwizPluginImportMenu: Component<
  PackwizPluginImportMenuProps
> = (props) => {
  const [local, others] = splitProps(props, ['onSubmit']);

  const [packwizUrl, setPackwizUrl] = createSignal('');

  const onSubmit = () => {
    const data: PackwizPluginData = {
      kind: 'import',
      data: {
        url: packwizUrl(),
      },
    };
    callPlugin('packwiz', JSON.stringify(data));
    local.onSubmit?.();
  };

  const onBrowse = async () => {
    const file = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'Packwiz modpack', extensions: ['toml'] }],
    });

    if (file) {
      setPackwizUrl(file);
    }
  };

  // TODO: fall in pipeline
  // const onChangeUrl: JSX.EventHandler<HTMLInputElement, InputEvent>
  const onChangeUrl = (e: Event) =>
    setPackwizUrl((e.currentTarget as HTMLInputElement).value);

  return (
    <div class='flex flex-col gap-4' {...others}>
      <div class='flex w-full items-end gap-4'>
        <TextField class='relative flex w-full flex-col gap-2'>
          <TextFieldLabel for='packwiz_url'>Packwiz URL or File</TextFieldLabel>
          <TextFieldInput
            value={packwizUrl()}
            onChange={onChangeUrl}
            type='text'
            class='pr-24'
            id='packwiz_url'
            required
            autocomplete='off'
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
        </TextField>
      </div>
      <Button onClick={onSubmit}>Import</Button>
    </div>
  );
};
