import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import type { Instance } from '@/entities/instances';

import { InstanceActionButton } from '@/features/instance-action-button';
import { Image } from '@/shared/ui';

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
    <div class='flex gap-6' {...others}>
      <div class='flex gap-3 overflow-hidden'>
        <Image class='min-w-max' src={local.instance.iconPath} />
        <InstanceHeaderInfo class='w-full' instance={local.instance} />
      </div>
      <div class='ml-auto flex items-center gap-2'>
        <InstanceActionButton class='w-20' instance={local.instance} />
        <OpenFolderButton instancePath={local.instancePath} />
        <SettingsButton />
      </div>
    </div>
  );
};
