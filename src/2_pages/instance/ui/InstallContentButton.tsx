import type { Component, ComponentProps } from 'solid-js';

import { open } from '@tauri-apps/plugin-dialog';
import IconMdiChevronDown from '~icons/mdi/chevron-down';
import IconMdiPlus from '~icons/mdi/plus';
import { splitProps, For, createMemo } from 'solid-js';

import type { ContentType, Instance } from '@/entities/instances';

import {
  CONTENT_TYPE_TO_TITLE,
  CONTENT_TYPES,
  useImportContents,
} from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
  Separator,
} from '@/shared/ui';

import { OPEN_FILTERS_BY_CONTENT_TYPE } from '../model';

export type InstallContentButtonProps = ComponentProps<'div'> & {
  instanceId: Instance['id'];
  onInstallContentClick?: () => void;
  contentTypes?: ContentType[];
  disabled?: boolean;
};

export const InstallContentButton: Component<InstallContentButtonProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'instanceId',
    'onInstallContentClick',
    'disabled',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const contentTypes = createMemo(() => props.contentTypes || CONTENT_TYPES);

  const { mutateAsync: importContents } = useImportContents();
  const handleAddContents = async (contentType: ContentType) => {
    const paths = await open({
      multiple: true,
      directory: false,
      filters: OPEN_FILTERS_BY_CONTENT_TYPE[contentType],
    });

    if (!paths) {
      return;
    }

    importContents({ id: local.instanceId, paths, type: contentType });
  };

  return (
    <div class={cn('flex', local.class)} {...others}>
      <Button
        class='min-w-max rounded-r-none'
        leadingIcon={IconMdiPlus}
        onClick={local.onInstallContentClick}
        disabled={local.disabled}
      >
        {t('instance.installContent')}
      </Button>
      <Separator orientation='vertical' />
      <DropdownMenu>
        <DropdownMenuTrigger
          as={IconButton}
          class='rounded-l-none'
          icon={IconMdiChevronDown}
          disabled={local.disabled}
        />
        <DropdownMenuContent>
          <For each={contentTypes()}>
            {(contentType) => (
              <DropdownMenuItem onClick={() => handleAddContents(contentType)}>
                <span>
                  {t('common.add')}
                  &nbsp;
                  <span class='lowercase'>
                    {t(`content.${CONTENT_TYPE_TO_TITLE[contentType]}`)}
                  </span>
                </span>
              </DropdownMenuItem>
            )}
          </For>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
};
