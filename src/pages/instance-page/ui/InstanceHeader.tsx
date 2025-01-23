import MdiClockIcon from '@iconify/icons-mdi/clock';
import MdiFolderIcon from '@iconify/icons-mdi/folder';
import MdiSettingsIcon from '@iconify/icons-mdi/settings';
import { Icon } from '@iconify-icon/solid';
import { useNavigate } from '@solidjs/router';
import { Component, ComponentProps, createMemo, splitProps } from 'solid-js';

import { IconButton, Separator } from '@/shared/ui';

import {
  formatTimePlayed,
  formatTimePlayedHumanized,
  InstanceImage,
  InstanceActionButton,
  useInstanceActions,
  useRunningInstancesContext,
  Instance,
} from '@/entities/instance';

export type InstanceHeaderProps = ComponentProps<'div'> & {
  instance: Instance;
};

export const InstanceHeader: Component<InstanceHeaderProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const navigate = useNavigate();

  const [context, { get }] = useRunningInstancesContext();
  const runningInstance = createMemo(() => get(context, local.instance.id));

  const { handleInstanceLaunch, handleInstanceStop, handleOpenFolder } =
    useInstanceActions();

  const lastPlayedDate = createMemo(() => {
    return local.instance?.lastPlayed
      ? new Date(Date.parse(local.instance?.lastPlayed))
      : undefined;
  });

  const navigateToSettings = () => {
    navigate('settings');
  };

  return (
    <div class='flex gap-4' {...others}>
      <InstanceImage src={local.instance.iconPath} />
      <div class='flex flex-col gap-2'>
        <span class='text-2xl font-bold'>{local.instance.name}</span>
        <span class='capitalize text-muted-foreground'>
          {local.instance.loader} {local.instance.loaderVersion}
        </span>
        <span class='inline-flex gap-2'>
          <span
            class='mt-auto inline-flex items-center gap-1 text-muted-foreground'
            title={formatTimePlayed(local.instance.timePlayed)}
          >
            <Icon icon={MdiClockIcon} />
            {formatTimePlayedHumanized(local.instance.timePlayed)}
          </span>
          <Separator orientation='vertical' />
          <span class='mt-auto inline-flex items-center gap-1 text-muted-foreground'>
            Last played:&nbsp;
            {lastPlayedDate()?.toLocaleString() ?? 'Never'}
          </span>
        </span>
      </div>
      <div class='ml-auto flex items-center gap-2'>
        <InstanceActionButton
          class='w-20 p-2'
          installStage={local.instance.installStage}
          isRunning={runningInstance()?.isRunning}
          isLoading={runningInstance()?.isLoading}
          onLaunchClick={() => handleInstanceLaunch(local.instance)}
          onStopClick={() => handleInstanceStop(local.instance)}
        >
          Play
        </InstanceActionButton>
        <IconButton
          class='aspect-square p-2'
          variant='secondary'
          title='Open folder'
          icon={MdiFolderIcon}
          onClick={() => handleOpenFolder(local.instance)}
        />
        <IconButton
          class='aspect-square p-2'
          variant='secondary'
          title='Settings'
          icon={MdiSettingsIcon}
          onClick={navigateToSettings}
        />
      </div>
    </div>
  );
};
