import MdiClockIcon from '@iconify/icons-mdi/clock';
import MdiEngineIcon from '@iconify/icons-mdi/engine';
import MdiFolderIcon from '@iconify/icons-mdi/folder';
import MdiSettingsIcon from '@iconify/icons-mdi/settings';
import { Icon } from '@iconify-icon/solid';
import { useNavigate } from '@solidjs/router';
import {
  Component,
  ComponentProps,
  createMemo,
  Show,
  splitProps,
} from 'solid-js';

import { IconButton } from '@/shared/ui';

import {
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

  const {
    launchInstance: handleInstanceLaunch,
    stopInstance: handleInstanceStop,
    openFolder: handleOpenFolder,
  } = useInstanceActions();

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
        <span
          class='inline-flex items-center gap-1 capitalize text-muted-foreground'
          title='Modloader'
        >
          <Icon icon={MdiEngineIcon} />
          {local.instance.loader} {local.instance.loaderVersion}
        </span>
        <span class='inline-flex items-center gap-1 text-muted-foreground'>
          <Icon icon={MdiClockIcon} />
          <Show when={local.instance.timePlayed} fallback='Never played'>
            <span
              class='mt-auto inline-flex items-center gap-1 capitalize'
              title={`Last played: ${lastPlayedDate()?.toLocaleString()}`}
            >
              {formatTimePlayedHumanized(local.instance.timePlayed)}
            </span>
          </Show>
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
