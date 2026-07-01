import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { type Instance } from '@/entities/instances';
import { InstanceIcon } from '@/entities/instances';
import { InstanceActionButton } from '@/features/instance-action-button';

import { InstanceHeaderInfo } from './InstanceHeaderInfo';
import { OpenFolderButton } from './OpenFolderButton';
import { SettingsButton } from './SettingsButton';

export type HeaderProps = ComponentProps<'div'> & {
  instance: Instance;
  instancePath?: string;
};

export const Header: Component<HeaderProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'instancePath',
    'class',
  ]);

  return (
    <div class='gap-6 flex' {...others}>
      <div class='gap-3 flex overflow-hidden'>
        <InstanceIcon src={props.instance.iconPath ?? undefined} />
        <InstanceHeaderInfo class='w-full' instance={local.instance} />
      </div>
      <div class='gap-2 ml-auto flex items-center'>
        <InstanceActionButton class='w-20' instance={local.instance} />
        <OpenFolderButton instancePath={local.instancePath} />
        <SettingsButton instanceId={local.instance.id} />
      </div>
    </div>
  );
};
