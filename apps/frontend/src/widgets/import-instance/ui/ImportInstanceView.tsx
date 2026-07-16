import type { JSX } from 'solid-js';

import { createMemo, For, splitProps, type Component } from 'solid-js';

import type {
  CapabilityEntryDto,
  PackManagerCapabilityMetadataDto,
} from '@/shared/api/bindings/instance';
import type { TabsProps } from '@/shared/ui';

import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/shared/ui';

import type { PackManagerTabConfig } from '../model';

import { packManagerToTab } from '../model';

export type ImportInstanceViewProps = TabsProps & {
  packManagers: CapabilityEntryDto<PackManagerCapabilityMetadataDto>[];
  footerButtons: JSX.Element;
  onSubmit?: () => void;
};

export const ImportInstanceView: Component<ImportInstanceViewProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'packManagers',
    'footerButtons',
    'onSubmit',
    'class',
  ]);

  const tabs = createMemo(() => {
    return local.packManagers.map(
      packManagerToTab,
    ) satisfies PackManagerTabConfig[];
  });

  return (
    <Tabs class='gap-4 flex grow' orientation='vertical' {...others}>
      <TabsList class='max-h-full'>
        <For each={tabs()}>
          {(tab) => <TabsTrigger value={tab.value}>{tab.label}</TabsTrigger>}
        </For>
      </TabsList>
      <For each={tabs()}>
        {(tab) => (
          <TabsContent class='p-0.5 flex-1 overflow-hidden' value={tab.value}>
            <tab.component
              footerButtons={local.footerButtons}
              onSubmit={local.onSubmit}
            />
          </TabsContent>
        )}
      </For>
    </Tabs>
  );
};
