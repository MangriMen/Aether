import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn } from '@/shared/lib';

export type ContentVersionListItemProps = ComponentProps<'div'>;

export const ContentVersionListItem: Component<ContentVersionListItemProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['class']);

  return (
    <div class={cn('', local.class)} {...others}>
      <div>
        <div>image</div>
        <div>name</div>
      </div>
      <div>version column</div>
      <div>platforms</div>
      <div>published</div>
      <div>downloads</div>
      <div>actions</div>
    </div>
  );
};
