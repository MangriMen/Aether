import { useParams } from '@solidjs/router';
import { For, splitProps, type Component, type ComponentProps } from 'solid-js';
import { Dynamic } from 'solid-js/web';

import { useContent } from '../../../entities/instances';
import { cn } from '../../../shared/lib';
import { useTranslation } from '../../../shared/model';
import {
  Separator,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '../../../shared/ui';
import { useContentContext } from '../model';
import { CONTENT_ITEM_PAGE_TABS_DEFINITION } from '../model/contentItemPageTabs';
import { ContentItemPageInfo } from './ContentItemPageInfo';

export type ContentItemPageProps = ComponentProps<'div'>;

export const ContentItemPage: Component<ContentItemPageProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  const [context] = useContentContext();

  const params = useParams();

  const item = useContent(() =>
    params.contentId && context.providerId
      ? {
          contentId: params.contentId,
          providerId: context.providerId,
        }
      : undefined,
  );

  return (
    <div
      class={cn('flex flex-col grow gap-4 overflow-hidden', local.class)}
      {...others}
    >
      <ContentItemPageInfo item={item.data} isLoading={item.isFetching} />
      <Separator />

      <Tabs
        class='flex flex-col overflow-hidden'
        defaultValue={CONTENT_ITEM_PAGE_TABS_DEFINITION[0].value}
      >
        <TabsList class='self-start'>
          <For each={CONTENT_ITEM_PAGE_TABS_DEFINITION}>
            {(tab) => (
              <TabsTrigger value={tab.value}>
                {t(`content.item.${tab.label}`)}
              </TabsTrigger>
            )}
          </For>
        </TabsList>

        <For each={CONTENT_ITEM_PAGE_TABS_DEFINITION}>
          {(tab) => (
            <TabsContent
              class='flex flex-1 flex-col overflow-hidden'
              value={tab.value}
            >
              <Dynamic
                class='flex flex-1 flex-col overflow-hidden'
                component={tab.component}
                item={item.data}
              />
            </TabsContent>
          )}
        </For>
      </Tabs>
    </div>
  );
};
