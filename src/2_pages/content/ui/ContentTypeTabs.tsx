import type { Component, ComponentProps } from 'solid-js';

import { For, Show, splitProps } from 'solid-js';

import {
  CONTENT_TYPE_TO_TITLE,
  isContentType,
  type ContentType,
} from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import { Skeleton, Tabs, TabsList, TabsTrigger } from '@/shared/ui';

export type ContentTypeTabsProps = Omit<ComponentProps<'div'>, 'onChange'> & {
  items: readonly ContentType[];
  value?: ContentType;
  defaultValue?: ContentType;
  isLoading?: boolean;
  disabled?: boolean;
  onChange: (contentType: ContentType) => void;
};

export const ContentTypeTabs: Component<ContentTypeTabsProps> = (props) => {
  const [local, others] = splitProps(props, [
    'items',
    'value',
    'defaultValue',
    'isLoading',
    'onChange',
  ]);

  const [{ t }] = useTranslation();

  const handleChange = (value: string) => {
    if (isContentType(value)) {
      local.onChange(value);
    } else {
      // eslint-disable-next-line no-console
      console.error(`Unknown content type received: ${value}`);
    }
  };

  return (
    <Tabs
      value={local.value}
      defaultValue={local.defaultValue}
      onChange={handleChange}
      {...others}
    >
      <Show
        when={!local.isLoading}
        fallback={
          <div class='flex'>
            <Skeleton class='bg-secondary' width={360} radius={6} />
          </div>
        }
      >
        <TabsList>
          <For each={local.items}>
            {(contentType) => (
              <TabsTrigger value={contentType}>
                {t(`content.${CONTENT_TYPE_TO_TITLE[contentType]}`)}
              </TabsTrigger>
            )}
          </For>
        </TabsList>
      </Show>
    </Tabs>
  );
};
