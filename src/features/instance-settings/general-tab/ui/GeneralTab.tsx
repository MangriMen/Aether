import { Component, ComponentProps, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { CombinedTextField } from '@/shared/ui';

import { InstanceSettingsTabProps } from '@/entities/instance';

export type GeneralTabProps = ComponentProps<'div'> & InstanceSettingsTabProps;

const GeneralTab: Component<GeneralTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <CombinedTextField label='Name' defaultValue={local.instance.name} />
    </div>
  );
};

export default GeneralTab;
