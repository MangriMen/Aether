import IconMdiDelete from '~icons/mdi/delete';
import IconMdiDownload from '~icons/mdi/download';
import IconMdiImageEditOutline from '~icons/mdi/image-edit-outline';
import {
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useTranslation } from '@/shared/model';
import {
  Button,
  CombinedTooltip,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Image,
} from '@/shared/ui';

export type InstanceIconDropdownButtonProps = ComponentProps<'div'> & {
  src?: string | undefined;
  onSelectIcon?: () => void;
  onRemoveIcon?: () => void;
};

export const InstanceIconDropdownButton: Component<
  InstanceIconDropdownButtonProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'src',
    'onSelectIcon',
    'onRemoveIcon',
    'class',
  ]);

  const [{ t }] = useTranslation();

  return (
    <DropdownMenu modal={false}>
      <DropdownMenuTrigger>
        <CombinedTooltip
          as={Button}
          label={t('instance.changeIcon')}
          class='relative size-full max-h-max max-w-max p-0 active:animate-bump-out'
          variant='ghost'
          placement='top'
          {...others}
        >
          <Image src={local.src} alt='Instance icon' />
          <div class='absolute inset-0 flex size-full items-center justify-center opacity-0 transition-opacity hover:bg-popover/50 hover:opacity-100 focus:opacity-100'>
            <IconMdiImageEditOutline class='text-2xl text-muted-foreground' />
          </div>
        </CombinedTooltip>
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        <DropdownMenuItem
          as={Button}
          class='w-full'
          variant='ghost'
          leadingIcon={IconMdiDownload}
          onClick={local.onSelectIcon}
        >
          <Show when={local.src === undefined} fallback='Change icon'>
            Select icon
          </Show>
        </DropdownMenuItem>
        <Show when={local.src !== undefined}>
          <DropdownMenuItem
            as={Button}
            class='w-full text-destructive enabled:hover:text-destructive'
            variant='ghost'
            leadingIcon={IconMdiDelete}
            onClick={local.onRemoveIcon}
          >
            Remove icon
          </DropdownMenuItem>
        </Show>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
