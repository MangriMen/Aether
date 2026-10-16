import { splitProps, type Component, type ComponentProps } from 'solid-js';

import {
  PluginCapabilitiesSection,
  PluginHostFunctionsSection,
} from '@/entities/plugins';
import { cn } from '@/shared/lib';

import type { PluginDetailsTabProps } from '../model';

export type PluginCapabilitiesTabProps = ComponentProps<'div'> &
  PluginDetailsTabProps;

export const PluginCapabilitiesTab: Component<PluginCapabilitiesTabProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  return (
    <div
      class={cn('gap-4 flex flex-col overflow-y-auto', local.class)}
      {...others}
    >
      <PluginCapabilitiesSection
        capabilities={local.plugin.capabilities ?? null}
      />
      <PluginHostFunctionsSection
        hostFunctions={local.plugin.manifest.api.hostFunctions}
      />
    </div>
  );
};
