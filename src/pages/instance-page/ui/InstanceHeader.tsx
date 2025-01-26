import MdiFolderIcon from '@iconify/icons-mdi/folder';
import MdiSettingsIcon from '@iconify/icons-mdi/settings';
import { useNavigate } from '@solidjs/router';
import type { Component, ComponentProps } from 'solid-js';
import { createMemo, splitProps } from 'solid-js';

import { IconButton } from '@/shared/ui';

import type { Instance } from '@/entities/instance';
import {
  InstanceImage,
  InstanceActionButton,
  useInstanceActions,
  useRunningInstancesContext,
} from '@/entities/instance';

import InstanceHeaderInfo from './InstanceHeaderInfo';

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

  const navigateToSettings = () => {
    navigate('settings');
  };

  return (
    <div class='flex gap-3' {...others}>
      <InstanceImage src={local.instance.iconPath} />
      <InstanceHeaderInfo instance={local.instance} />
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
