import type { JSX } from 'solid-js';

import { createMemo, For, splitProps, type Component } from 'solid-js';

import type { ImporterCapabilityEntry } from '@/entities/plugins';
import type { TabsProps } from '@/shared/ui';

import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/shared/ui';

import type { ImporterTabConfig } from '../model';

import { importerToTab } from '../model';

export type ImportInstanceViewProps = TabsProps & {
  importers: ImporterCapabilityEntry[];
  footerButtons: JSX.Element;
  onSubmit?: () => void;
};

export const ImportInstanceView: Component<ImportInstanceViewProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'importers',
    'footerButtons',
    'onSubmit',
    'class',
  ]);

  const tabs = createMemo(() => {
    return local.importers.map(importerToTab) satisfies ImporterTabConfig[];
  });

  return (
    <Tabs class='flex grow gap-4' orientation='vertical' {...others}>
      <TabsList class='max-h-max'>
        <For each={tabs()}>
          {(tab) => <TabsTrigger value={tab.value}>{tab.label}</TabsTrigger>}
        </For>
      </TabsList>
      <For each={tabs()}>
        {(tab) => (
          <TabsContent
            class='flex-1 overflow-hidden p-0.5'
            value={tab.value}
            as={tab.component}
            footerButtons={local.footerButtons}
            onSubmit={local.onSubmit}
          />
        )}
      </For>
    </Tabs>
  );
};
