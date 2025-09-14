import type { Component, ComponentProps } from 'solid-js';

import MdiDeleteIcon from '@iconify/icons-mdi/delete';
import MdiMoreIcon from '@iconify/icons-mdi/dots-vertical';
import { splitProps } from 'solid-js';

import {
  type InstanceFile,
  useDisableContents,
  useEnableContents,
  useRemoveContent,
  useRevealInExplorer,
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

export type ContentActionsProps = {
  content: InstanceFile;
  instanceId: string;
  instancePath?: string;
} & ComponentProps<'div'>;

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

  const { mutateAsync: removeInstanceContent } = useRemoveContent();
  const { mutateAsync: revealInExplorer } = useRevealInExplorer();

  const handleToggleDisable = () => {
    if (local.content.disabled) {
      enableContents({
        id: local.instanceId,
        paths: [local.content.path],
      });
    } else {
      disableContents({
        id: local.instanceId,
        paths: [local.content.path],
      });
    }
  };

  const handleRemove = () => {
    removeInstanceContent({ id: local.instanceId, path: local.content.path });
  };

  const handleShowFile = () => {
    revealInExplorer({
      exact: false,
      path: `${local.instancePath}/${local.content.path}`,
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
        as={IconButton}
        class='p-0'
        icon={MdiDeleteIcon}
        label={t('common.remove')}
        onClick={handleRemove}
        variant='ghost'
      />
      <DropdownMenu>
        <DropdownMenuTrigger
          as={IconButton}
          class='p-0'
          icon={MdiMoreIcon}
          variant='ghost'
        />
        <DropdownMenuContent>
          <DropdownMenuItem
            disabled={!local.instancePath}
            onClick={handleShowFile}
          >
            {t('instance.showFile')}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
};
