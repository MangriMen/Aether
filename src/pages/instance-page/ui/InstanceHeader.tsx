import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';

import type { Instance } from '@/entities/instance';
import { InstanceImage } from '@/entities/instance';

import { InstanceActionButton } from '@/features/instance-action-button';

import InstanceHeaderInfo from './InstanceHeaderInfo';
import InstanceOpenFolderButton from './InstanceOpenFolderButton';
import InstanceSettingsButton from './InstanceSettingsButton';

export type InstanceHeaderProps = ComponentProps<'div'> & {
  instance: Instance;
};

export const InstanceHeader: Component<InstanceHeaderProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  return (
    <div class='flex gap-3' {...others}>
      <InstanceImage src={local.instance.iconPath} />
      <InstanceHeaderInfo instance={local.instance} />
      <div class='ml-auto flex items-center gap-2'>
        <InstanceActionButton class='w-20 p-2' instance={local.instance} />
        <InstanceOpenFolderButton instance={local.instance} />
        <InstanceSettingsButton />
      </div>
    </div>
  );
};
