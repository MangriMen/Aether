import { open } from '@tauri-apps/plugin-dialog';
import IconMdiFileFindOutline from '~icons/mdi/file-find-outline';
import {
  untrack,
  createSignal,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  Button,
  Checkbox,
  CombinedTextField,
  CombinedTooltip,
  IconButton,
} from '@/shared/ui';

import type {
  AllowedPathErrors,
  AllowedPathValue,
  EditAllowedItemProps,
} from '../model';

export type EditAllowedPathProps = ComponentProps<'div'> &
  EditAllowedItemProps<AllowedPathValue, AllowedPathErrors>;

export const EditAllowedPath: Component<EditAllowedPathProps> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'error',
    'onSave',
    'onCancel',
    'class',
  ]);

  const [{ t }] = useTranslation();

  // Taken once: the form re-renders its value on every change, which would overwrite the
  // input the user is typing.
  const initial = untrack(() => local.value);
  const [src, setSrc] = createSignal(initial?.hostPath ?? '');
  const [dest, setDest] = createSignal(initial?.pluginPath ?? '');
  const [readOnly, setReadOnly] = createSignal(initial?.readOnly ?? true);

  const handleSubmit = () => {
    local.onSave?.({
      hostPath: src(),
      pluginPath: dest(),
      readOnly: readOnly(),
    });
  };

  const handleBrowse = async () => {
    const file = await open({
      multiple: false,
      directory: true,
    });

    if (file) {
      setSrc(file);
      setDest(file);
    }
  };

  return (
    <div
      class={cn('gap-2 flex size-full items-start', local.class)}
      {...others}
    >
      <CombinedTextField
        class='w-full'
        inputProps={{
          type: 'text',
          class: 'h-max py-1 px-1',
          placeholder: t('pluginSettings.hostPathPlaceholder'),
        }}
        value={src()}
        onChange={setSrc}
        errorMessage={local.error?.hostPath}
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
      <CombinedTextField
        class='w-full'
        inputProps={{
          type: 'text',
          class: 'h-max py-1 px-1',
          placeholder: t('pluginSettings.pluginPathPlaceholder'),
        }}
        value={dest()}
        onChange={setDest}
        errorMessage={local.error?.pluginPath}
      />
      <Checkbox
        class='h-8 shrink-0 text-sm'
        label={t('pluginSettings.readOnly')}
        checked={readOnly()}
        onChange={setReadOnly}
      />
      <Button size='sm' onClick={handleSubmit}>
        {t('common.ok')}
      </Button>
      <Button variant='secondary' size='sm' onClick={local.onCancel}>
        {t('common.cancel')}
      </Button>
    </div>
  );
};
