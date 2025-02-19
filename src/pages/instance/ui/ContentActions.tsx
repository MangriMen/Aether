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
import type { Instance, InstanceFile } from '@/entities/instances';
import {
  revealInExplorer,
  removeInstanceContent,
  toggleDisableInstanceContent,
} from '@/entities/instances';

export type ContentActionsProps = ComponentProps<'div'> & {
  instanceId: string;
  instancePath: Instance['path'];
  content: InstanceFile;
};

export const ContentActions: Component<ContentActionsProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instanceId',
    'instancePath',
    'content',
    'class',
  ]);

  const handleToggleDisable = () => {
    toggleDisableInstanceContent(local.instanceId, local.content.path);
  };

  const handleRemove = () => {
    removeInstanceContent(local.instanceId, local.content.path);
  };

  const handleShowFile = () => {
    revealInExplorer(`${local.instancePath}/${local.content.path}`, false);
  };

  return (
    <div class={cn('flex items-center gap-2', local.class)} {...others}>
      <Switch checked={!local.content.disabled} onChange={handleToggleDisable}>
        <SwitchControl>
          <SwitchThumb class='bg-secondary-foreground' />
        </SwitchControl>
      </Switch>
      <CombinedTooltip
        label='Remove'
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
          <DropdownMenuItem onClick={handleShowFile}>
            Show file
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
};
