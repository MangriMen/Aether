import type { TabsProps } from '@/shared/ui';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/shared/ui';
import { splitProps, type Component } from 'solid-js';
import { InstanceContentTabs } from '../model/content';
import { useTranslation } from '@/shared/model';
import { ContentTab } from './ContentTab';
import type { Instance } from '@/entities/instances';
import { cn } from '@/shared/lib';

export type BodyProps = TabsProps & {
  instance: Instance;
  instancePath?: string;
};

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
        value={InstanceContentTabs.Content}
        as={ContentTab}
        class='flex-1 overflow-y-auto'
        instance={local.instance}
        instancePath={local.instancePath}
      />
    </Tabs>
  );
};
