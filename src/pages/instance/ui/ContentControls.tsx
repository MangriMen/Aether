import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
  For,
} from 'solid-js';

import MdiChevronDownIcon from '@iconify/icons-mdi/chevron-down';
import {
  Button,
  CombinedTextField,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
  Separator,
} from '@/shared/ui';
import { cn } from '@/shared/lib';
import { CONTENT_TYPES } from '@/entities/instances';

export type ContentControlsProps = ComponentProps<'div'> & {
  contentsCount: number;
  onSearch?: (query: string) => void;
};

export const ContentControls: Component<ContentControlsProps> = (props) => {
  const [local, others] = splitProps(props, [
    'contentsCount',
    'onSearch',
    'class',
  ]);

  const searchPlaceholder = createMemo(
    () => `Search ${local.contentsCount} contents`,
  );

  return (
    <div class={cn('flex items-center gap-2', local.class)} {...others}>
      <CombinedTextField
        class='w-full'
        name='search'
        inputProps={{ type: 'text', placeholder: searchPlaceholder() }}
        onChange={local.onSearch}
      />
      <div class='flex h-9'>
        <Button class='min-w-max rounded-r-none' disabled>
          Install content
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
            <For each={CONTENT_TYPES}>
              {(contentType) => (
                <DropdownMenuItem>Add {contentType}</DropdownMenuItem>
              )}
            </For>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  );
};
