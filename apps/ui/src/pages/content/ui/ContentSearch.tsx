import type { ComponentProps } from 'solid-js';

import { createMemo, splitProps, type Component } from 'solid-js';

import type { CombinedTextFieldProps } from '@/shared/ui';

import { CONTENT_TYPE_TO_TITLE, type ContentType } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTextField } from '@/shared/ui';

export type ContentSearchProps = CombinedTextFieldProps &
  ComponentProps<'div'> & {
    contentType?: ContentType;
    onSearch?: (query: string) => void;
  };

export const ContentSearch: Component<ContentSearchProps> = (props) => {
  const [local, others] = splitProps(props, [
    'contentType',
    'onSearch',
    'inputProps',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const searchPlaceholder = createMemo(() => {
    const searchText = t('common.search');
    const contentTypeText = local.contentType
      ? t(`content.${CONTENT_TYPE_TO_TITLE[local.contentType]}`)
      : t('content.title');

    return `${searchText} ${contentTypeText}`;
  });

  return (
    <CombinedTextField
      class={cn('w-full', local.class)}
      name='search'
      inputProps={{
        type: 'text',
        placeholder: searchPlaceholder(),
        ...local.inputProps,
      }}
      onChange={local.onSearch}
      {...others}
    />
  );
};
