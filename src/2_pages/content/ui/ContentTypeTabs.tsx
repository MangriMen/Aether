import type { Component, ComponentProps } from 'solid-js';

import { For, splitProps } from 'solid-js';

import { CONTENT_TYPE_TO_TITLE, type ContentType } from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import { Tabs, TabsList, TabsTrigger } from '@/shared/ui';

export type ContentTypeTabsProps = ComponentProps<'div'> & {
  items: ContentType[];
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
    local.onChange(value as (typeof local.items)[number]);
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
