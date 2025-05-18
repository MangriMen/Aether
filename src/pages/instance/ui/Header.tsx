import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';

import type { Instance } from '@/entities/instances';

import { InstanceActionButton } from '@/features/instance-action-button';

import { InstanceHeaderInfo } from './InstanceHeaderInfo';
import { OpenFolderButton } from './OpenFolderButton';
import { SettingsButton } from './SettingsButton';
import { Image } from '@/shared/ui';

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
    <div class='flex gap-3' {...others}>
      <Image src={local.instance.iconPath} />
      <InstanceHeaderInfo instance={local.instance} />
      <div class='ml-auto flex items-center gap-2'>
        <InstanceActionButton class='w-20 p-2' instance={local.instance} />
        <OpenFolderButton instancePath={local.instancePath} />
        <SettingsButton />
      </div>
    </div>
  );
};
