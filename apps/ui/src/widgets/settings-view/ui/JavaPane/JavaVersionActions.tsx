import IconMdiMore from '~icons/mdi/dots-vertical';
import IconMdiDownload from '~icons/mdi/download';
import IconMdiFileFindOutline from '~icons/mdi/file-find-outline';
import IconMdiMagnify from '~icons/mdi/magnify';
import {
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useTranslation } from '@/shared/model';
import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
} from '@/shared/ui';
export type JavaVersionActionsProps = ComponentProps<'div'> & {
  isInstalling?: boolean;
  onInstallRecommended?: () => void;
  onDetect?: () => void;
  onBrowse?: () => void;
};

export const JavaVersionActions: Component<JavaVersionActionsProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'isInstalling',
    'onInstallRecommended',
    'onDetect',
    'onBrowse',
  ]);

  const [{ t }] = useTranslation();

  return (
    <DropdownMenu {...others}>
      <DropdownMenuTrigger
        as={IconButton}
        variant='ghost'
        class='p-0'
        icon={IconMdiMore}
      />
      <DropdownMenuContent>
        <DropdownMenuItem
          as={Button}
          class='w-full justify-start'
          variant='ghost'
          leadingIcon={IconMdiDownload}
          onClick={local.onInstallRecommended}
          disabled={local.isInstalling}
        >
          {t('javaVersion.installRecommended')}
        </DropdownMenuItem>
        <Show when={local.onDetect}>
          <DropdownMenuItem
            as={Button}
            class='w-full justify-start'
            variant='ghost'
            leadingIcon={IconMdiMagnify}
            onClick={local.onInstallRecommended}
            disabled={local.isInstalling}
          >
            {t('javaVersion.detect')}
          </DropdownMenuItem>
        </Show>
        <Show when={local.onBrowse}>
          <DropdownMenuItem
            as={Button}
            class='w-full justify-start'
            variant='ghost'
            leadingIcon={IconMdiFileFindOutline}
            onClick={local.onInstallRecommended}
            disabled={local.isInstalling}
          >
            {t('javaVersion.browse')}
          </DropdownMenuItem>
        </Show>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
