import IconMdiDelete from '~icons/mdi/delete';
import IconMdiDownload from '~icons/mdi/download';
import IconMdiImageEditOutline from '~icons/mdi/image-edit-outline';
import {
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { InstanceIcon } from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import {
  Button,
  CombinedTooltip,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
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
          <InstanceIcon src={local.src} alt={t('instance.icon')} />
          <div class='absolute inset-0 flex size-full items-center justify-center opacity-0 transition-opacity hover:bg-popover/50 hover:opacity-100 focus:opacity-100'>
            <IconMdiImageEditOutline class='text-2xl text-muted-foreground' />
          </div>
        </CombinedTooltip>
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        <DropdownMenuItem
          as={Button}
          class='w-full justify-start'
          variant='ghost'
          leadingIcon={IconMdiDownload}
          onClick={local.onSelectIcon}
        >
          <Show
            when={local.src === undefined}
            fallback={t('instance.changeIcon')}
          >
            {t('instance.selectIcon')}
          </Show>
        </DropdownMenuItem>
        <Show when={local.src !== undefined}>
          <DropdownMenuItem
            as={Button}
            class='w-full justify-start text-destructive enabled:hover:text-destructive'
            variant='ghost'
            leadingIcon={IconMdiDelete}
            onClick={local.onRemoveIcon}
          >
            {t('instance.removeIcon')}
          </DropdownMenuItem>
        </Show>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
