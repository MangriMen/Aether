import { cn } from '@/shared/lib';
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
import { splitProps } from 'solid-js';
import type { Component, ComponentProps } from 'solid-js';
import MdiDeleteIcon from '@iconify/icons-mdi/delete';
import MdiMoreIcon from '@iconify/icons-mdi/dots-vertical';
import {
  useDisableContents,
  useEnableContents,
  useRemoveContent,
  useRevealInExplorer,
  type InstanceFile,
} from '@/entities/instances';
import { useTranslation } from '@/shared/model';

export type ContentActionsProps = ComponentProps<'div'> & {
  instanceId: string;
  instancePath?: string;
  content: InstanceFile;
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
      path: `${local.instancePath}/${local.content.path}`,
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
        icon={MdiDeleteIcon}
        onClick={handleRemove}
      />
      <DropdownMenu>
        <DropdownMenuTrigger
          as={IconButton}
          variant='ghost'
          class='p-0'
          icon={MdiMoreIcon}
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
