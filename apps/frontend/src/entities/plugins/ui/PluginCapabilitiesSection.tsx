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
  ImporterCapability,
  PluginCapabilities,
  UpdaterCapability,
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

  const hasImporters = createMemo(
    () => caps()?.importers != null && caps()!.importers!.length > 0,
  );
  const hasUpdaters = createMemo(
    () => caps()?.updaters != null && caps()!.updaters!.length > 0,
  );
  const hasContentProviders = createMemo(
    () =>
      caps()?.contentProviders != null && caps()!.contentProviders!.length > 0,
  );

  return (
    <Show when={hasImporters() || hasUpdaters() || hasContentProviders()}>
      <div class={cn('flex flex-col gap-4', local.class)} {...others}>
        <Show when={hasImporters()}>
          <div class='flex flex-col gap-2'>
            <FieldLabel>{t('plugin.importers')}</FieldLabel>
            <For each={caps()!.importers as ImporterCapability[]}>
              {(importer) => <PluginCapabilityCard capability={importer} />}
            </For>
          </div>
        </Show>
        <Show when={hasUpdaters()}>
          <div class='flex flex-col gap-2'>
            <FieldLabel>{t('plugin.updaters')}</FieldLabel>
            <For each={caps()!.updaters as UpdaterCapability[]}>
              {(updater) => <PluginCapabilityCard capability={updater} />}
            </For>
          </div>
        </Show>
        <Show when={hasContentProviders()}>
          <div class='flex flex-col gap-2'>
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
