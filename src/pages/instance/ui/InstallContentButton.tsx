import { cn } from '@/shared/lib';
import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
  Separator,
} from '@/shared/ui';
import type { Component, ComponentProps } from 'solid-js';
import { splitProps, For, createMemo } from 'solid-js';

import MdiChevronDownIcon from '@iconify/icons-mdi/chevron-down';
import MdiPlusIcon from '@iconify/icons-mdi/plus';

import { useTranslate } from '@/shared/model';
import type { ContentType } from '@/entities/instances';
import { CONTENT_TYPES } from '@/entities/instances';

export type InstallContentButtonProps = ComponentProps<'div'> & {
  onInstallContentClick?: () => void;
  contentTypes?: ContentType[];
  disabled?: boolean;
};

export const InstallContentButton: Component<InstallContentButtonProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'onInstallContentClick',
    'disabled',
    'class',
  ]);

  const [{ t }] = useTranslate();

  const contentTypes = createMemo(() => props.contentTypes || CONTENT_TYPES);

  return (
    <div class={cn('flex', local.class)} {...others}>
      <Button
        class='min-w-max rounded-r-none'
        leadingIcon={MdiPlusIcon}
        onClick={local.onInstallContentClick}
      >
        {t('instance.installContent')}
      </Button>
      <Separator orientation='vertical' />
      <DropdownMenu>
        <DropdownMenuTrigger
          as={IconButton}
          class='rounded-l-none p-0 text-xl'
          size='sm'
          icon={MdiChevronDownIcon}
          disabled
        />
        <DropdownMenuContent>
          <For each={contentTypes()}>
            {(contentType) => (
              <DropdownMenuItem>Add {contentType}</DropdownMenuItem>
            )}
          </For>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
};
