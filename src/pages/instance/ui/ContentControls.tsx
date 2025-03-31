import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { CombinedTextField } from '@/shared/ui';
import { cn } from '@/shared/lib';
import { InstallContentButton } from './InstallContentButton';

export type ContentControlsProps = ComponentProps<'div'> & {
  contentsCount: number;
  onSearch?: (query: string) => void;
  onInstallContentClick?: () => void;
};

export const ContentControls: Component<ContentControlsProps> = (props) => {
  const [local, others] = splitProps(props, [
    'contentsCount',
    'onSearch',
    'onInstallContentClick',
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
      <InstallContentButton
        onInstallContentClick={local.onInstallContentClick}
      />
      {/* <div class='flex'>
        <Button
          class='min-w-max rounded-r-none'
          onClick={local.onInstallContent}
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
            <For each={CONTENT_TYPES}>
              {(contentType) => (
                <DropdownMenuItem>Add {contentType}</DropdownMenuItem>
              )}
            </For>
          </DropdownMenuContent>
        </DropdownMenu>
      </div> */}
    </div>
  );
};
