import { useMatch, useNavigate, useParams } from '@solidjs/router';
import { For, splitProps, type Component, type ComponentProps } from 'solid-js';
import { Dynamic } from 'solid-js/web';

import {
  CONTENT_ROUTES,
  createContentPageHref,
  createContentVersionsHref,
  useContent,
  useContentContext,
} from '@/features/content';
import { useTranslation } from '@/shared/i18n';
import {
  Separator,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import {
  CONTENT_ITEM_PAGE_TABS_DEFINITION,
  ContentItemPageTab,
} from '../model/contentItemPageTabs';
import { ContentItemPageInfo } from './ContentItemPageInfo';

export type ContentItemPageProps = ComponentProps<'div'>;

export const ContentItemPage: Component<ContentItemPageProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  const [context] = useContentContext();

  const params = useParams();
  const navigate = useNavigate();

  const versionsMatch = useMatch(
    () => `${CONTENT_ROUTES.CONTENT_ITEM_VERSIONS(params.contentId ?? '')}/*`,
  );

  // The address picks the tab, so a link can land on a version and Back works.
  const activeTab = (): ContentItemPageTab =>
    versionsMatch()
      ? ContentItemPageTab.Versions
      : ContentItemPageTab.Description;

  const handleTabChange = (value: string) => {
    if (value === activeTab()) {
      return;
    }

    const hrefParams = {
      instanceId: context.instanceId,
      providerId: context.providerId,
    };
    const contentId = params.contentId ?? '';

    navigate(
      value === ContentItemPageTab.Versions
        ? createContentVersionsHref(contentId, hrefParams)
        : createContentPageHref(contentId, hrefParams),
      { replace: true },
    );
  };

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
      class={cn('gap-4 p-0.5 flex grow flex-col overflow-hidden', local.class)}
      {...others}
    >
      <ContentItemPageInfo item={item.data} isLoading={item.isFetching} />
      <Separator />

      <Tabs
        class='p-0.5 flex flex-col overflow-hidden'
        value={activeTab()}
        onChange={handleTabChange}
      >
        <TabsList class='p-0 self-start'>
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
