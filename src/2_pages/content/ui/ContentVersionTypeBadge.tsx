import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { ContentVersion } from '@/entities/instances';

import { cn } from '@/shared/lib';

export interface ContentVersionTypeBadgeProps {
  type: ContentVersion['versionType'];
}

export const ContentVersionTypeBadge: Component<
  ComponentProps<'div'> & ContentVersionTypeBadgeProps
> = (props) => {
  const [local, others] = splitProps(props, ['type', 'class']);

  const data = createMemo(() => {
    const firstLetter = local.type.charAt(0).toUpperCase();
    const typeKey = local.type.toLowerCase();

    return {
      firstLetter,
      typeKey,
    };
  });

  return (
    <div
      class={cn(
        'flex items-center justify-center size-9 rounded bg-muted text-muted-foreground font-medium text-base',
        local.class,
      )}
      style={{
        background: `hsl(var(--${data().typeKey}))`,
        color: `hsl(var(--${data().typeKey}-foreground))`,
      }}
      {...others}
    >
      {data().firstLetter}
    </div>
  );
};
