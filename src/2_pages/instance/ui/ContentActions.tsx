import type { Component, ComponentProps } from 'solid-js';

import IconMdiDelete from '~icons/mdi/delete';
import IconMdiMore from '~icons/mdi/dots-vertical';
import { splitProps } from 'solid-js';

import {
  useDisableContents,
  useEnableContents,
  useRemoveContents,
  useRevealInExplorer,
  type ContentFile,
} from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  CombinedTooltip,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
  Switch,
  SwitchControl,
  SwitchThumb,
} from '@/shared/ui';

export type ContentActionsProps = ComponentProps<'div'> & {
  instanceId: string;
  instancePath?: string;
  content: ContentFile;
};

export const ContentActions: Component<ContentActionsProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instanceId',
    'instancePath',
    'content',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const { mutateAsync: disableContents } = useDisableContents();
  const { mutateAsync: enableContents } = useEnableContents();

  const { mutateAsync: removeInstanceContents } = useRemoveContents();
  const { mutateAsync: revealInExplorer } = useRevealInExplorer();

  const handleToggleDisable = () => {
    if (local.content.disabled) {
      enableContents({
        id: local.instanceId,
        paths: [local.content.contentPath],
      });
    } else {
      disableContents({
        id: local.instanceId,
        paths: [local.content.contentPath],
      });
    }
  };

  const handleRemove = () => {
    removeInstanceContents({
      id: local.instanceId,
      paths: [local.content.contentPath],
    });
  };

  const handleShowFile = () => {
    revealInExplorer({
      path: `${local.instancePath}/${local.content.instanceRelativePath}`,
      exact: false,
    });
  };

  return (
    <div class={cn('flex items-center gap-2', local.class)} {...others}>
      <Switch checked={!local.content.disabled} onChange={handleToggleDisable}>
        <SwitchControl>
          <SwitchThumb class='bg-secondary-foreground' />
        </SwitchControl>
      </Switch>
      <CombinedTooltip
        label={t('common.remove')}
        as={IconButton}
        variant='ghost'
        class='p-0'
        icon={IconMdiDelete}
        onClick={handleRemove}
      />
      <DropdownMenu>
        <DropdownMenuTrigger
          as={IconButton}
          variant='ghost'
          class='p-0'
          icon={IconMdiMore}
        />
        <DropdownMenuContent>
          <DropdownMenuItem
            onClick={handleShowFile}
            disabled={!local.instancePath}
          >
            {t('instance.showFile')}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
};
