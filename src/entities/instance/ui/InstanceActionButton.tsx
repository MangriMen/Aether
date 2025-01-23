import MdiStopIcon from '@iconify/icons-mdi/stop';
import { PolymorphicProps } from '@kobalte/core';
import {
  Component,
  splitProps,
  createMemo,
  Show,
  ValidComponent,
} from 'solid-js';

import { IconButton, IconButtonProps, PlayIcon } from '@/shared/ui';

import { InstanceInstallStage } from '@/entities/instance';

export type InstanceActionButtonProps = IconButtonProps & {
  installStage: InstanceInstallStage;
  onLaunchClick?: PolymorphicProps<'button'>['onClick'];
  onStopClick?: PolymorphicProps<'button'>['onClick'];
  isLoading?: boolean;
  isRunning?: boolean;
};

type InstanceButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>>;

const InstancePlayButton: Component<InstanceButtonProps> = (props) => {
  return (
    <IconButton title='Launch' variant='success' {...props}>
      <PlayIcon />
    </IconButton>
  );
};

const InstanceStopButton: Component<InstanceButtonProps> = (props) => {
  return (
    <IconButton
      title='Stop'
      variant='destructive'
      icon={MdiStopIcon}
      {...props}
    />
  );
};

export const InstanceActionButton: Component<InstanceActionButtonProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'installStage',
    'isLoading',
    'isRunning',
    'onLaunchClick',
    'onStopClick',
  ]);

  const isInstanceInstalled = createMemo(
    () => local.installStage === InstanceInstallStage.Installed,
  );

  const isInstanceInstalling = createMemo(
    () => local.installStage === InstanceInstallStage.Installing,
  );

  const isPlayButtonLoading = createMemo(
    () => isInstanceInstalling() || local.isLoading,
  );

  const isPlayButton = createMemo(() => !local.isRunning);

  const isPlayButtonDisabled = createMemo(
    () => !isInstanceInstalled() || local.isRunning || local.isLoading,
  );

  return (
    <Show
      when={isPlayButton()}
      fallback={<InstanceStopButton onClick={local.onStopClick} {...others} />}
    >
      <InstancePlayButton
        loading={isPlayButtonLoading()}
        disabled={isPlayButtonDisabled()}
        onClick={local.onLaunchClick}
        {...others}
      />
    </Show>
  );
};
