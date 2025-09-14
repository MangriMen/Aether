import { type Component, splitProps } from 'solid-js';

import type { Instance } from '@/entities/instances';
import type { TabsProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/shared/ui';

import { InstanceContentTabs } from '../model/content';
import { ContentTab } from './ContentTab';

export type BodyProps = {
  instance: Instance;
  instancePath?: string;
} & TabsProps;

export const Body: Component<BodyProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'instancePath',
    'class',
  ]);

  const [{ t }] = useTranslation();

  return (
    <Tabs
      class={cn('flex flex-col overflow-hidden', local.class)}
      defaultValue={InstanceContentTabs.Content}
      {...others}
    >
      <TabsList class='self-start'>
        <TabsTrigger value={InstanceContentTabs.Content}>
          {t('instance.content')}
        </TabsTrigger>
      </TabsList>
      <TabsContent
        as={ContentTab}
        class='flex-1 overflow-y-auto'
        instance={local.instance}
        instancePath={local.instancePath}
        value={InstanceContentTabs.Content}
      />
    </Tabs>
  );
};
