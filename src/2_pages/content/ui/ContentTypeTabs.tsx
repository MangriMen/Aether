import type { Component, ComponentProps } from 'solid-js';

import { For, splitProps } from 'solid-js';

import {
  CONTENT_TYPE_TO_TITLE,
  isContentType,
  type ContentType,
} from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import { Tabs, TabsList, TabsTrigger } from '@/shared/ui';

export type ContentTypeTabsProps = Omit<ComponentProps<'div'>, 'onChange'> & {
  items: readonly ContentType[];
  value?: ContentType;
  defaultValue?: ContentType;
  onChange: (contentType: ContentType) => void;
};

export const ContentTypeTabs: Component<ContentTypeTabsProps> = (props) => {
  const [local, others] = splitProps(props, [
    'items',
    'value',
    'defaultValue',
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
      <TabsList>
        <For each={local.items}>
          {(contentType) => (
            <TabsTrigger value={contentType}>
              {t(`content.${CONTENT_TYPE_TO_TITLE[contentType]}`)}
            </TabsTrigger>
          )}
        </For>
      </TabsList>
    </Tabs>
  );
};
