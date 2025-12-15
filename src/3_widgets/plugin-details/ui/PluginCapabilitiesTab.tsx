import {
  For,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { PluginCapabilityCard } from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import type { PluginDetailsTabProps } from '../model';

export type PluginCapabilitiesTabProps = ComponentProps<'div'> &
  PluginDetailsTabProps;

export const PluginCapabilitiesTab: Component<PluginCapabilitiesTabProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const [{ t }] = useTranslation();

  return (
    <Show when={local.plugin.capabilities}>
      {(capabilities) => (
        <div class={cn('flex flex-col', local.class)} {...others}>
          <div class='flex flex-col gap-2'>
            <span class='text-lg font-medium'>{t('plugin.importers')}</span>
            <For each={capabilities().importers}>
              {(importer) => <PluginCapabilityCard capability={importer} />}
            </For>
          </div>
          <div class='flex flex-col gap-2'>
            <span class='text-lg font-medium'>{t('plugin.updaters')}</span>
            <For each={capabilities().updaters}>
              {(importer) => <PluginCapabilityCard capability={importer} />}
            </For>
          </div>
        </div>
      )}
    </Show>
  );
};
