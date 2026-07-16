import {
  createMemo,
  For,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { FieldLabel } from '@/shared/ui';

import type {
  ContentProviderCapability,
  PackManagerCapability,
  PluginCapabilities,
} from '../model';

import { PluginCapabilityCard } from './PluginCapabilityCard';

export type PluginCapabilitiesSectionProps = ComponentProps<'div'> & {
  capabilities: PluginCapabilities | null;
};

export const PluginCapabilitiesSection: Component<
  PluginCapabilitiesSectionProps
> = (props) => {
  const [local, others] = splitProps(props, ['capabilities', 'class']);
  const [{ t }] = useTranslation();

  const caps = createMemo(() => local.capabilities);

  const hasPackManagers = createMemo(
    () => caps()?.packManagers != null && caps()!.packManagers!.length > 0,
  );
  const hasContentProviders = createMemo(
    () =>
      caps()?.contentProviders != null && caps()!.contentProviders!.length > 0,
  );

  return (
    <Show when={hasPackManagers() || hasContentProviders()}>
      <div class={cn('gap-4 flex flex-col', local.class)} {...others}>
        <Show when={hasPackManagers()}>
          <div class='gap-2 flex flex-col'>
            <FieldLabel>{t('plugin.packManagers')}</FieldLabel>
            <For each={caps()!.packManagers as PackManagerCapability[]}>
              {(manager) => <PluginCapabilityCard capability={manager} />}
            </For>
          </div>
        </Show>
        <Show when={hasContentProviders()}>
          <div class='gap-2 flex flex-col'>
            <FieldLabel>{t('plugin.contentProviders')}</FieldLabel>
            <For each={caps()!.contentProviders as ContentProviderCapability[]}>
              {(provider) => <PluginCapabilityCard capability={provider} />}
            </For>
          </div>
        </Show>
      </div>
    </Show>
  );
};
