import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { ContentVersion } from '../../../entities/instances';

import { cn } from '../../../shared/lib';

export interface ContentVersionTypeBadgeProps {
  type: ContentVersion['versionType'];
}

const variants = {
  release: 'bg-release text-release-foreground',
  beta: 'bg-beta text-beta-foreground',
  alpha: 'bg-alpha text-alpha-foreground',
} as const;

type Variant = keyof typeof variants;

const variantsKeys = Object.keys(variants);

const isVariant = (value: unknown): value is Variant =>
  typeof value === 'string' && variantsKeys.includes(value as Variant);

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

  const variantClass = createMemo(() => {
    const key = data().typeKey;
    if (!key || !isVariant(key)) {
      return;
    }

    return variants[key];
  });

  return (
    <div
      class={cn(
        'flex items-center justify-center size-9 rounded bg-muted text-muted-foreground font-medium text-base',
        local.class,
        {
          [variantClass() ?? '']: Boolean(variantClass()),
        },
      )}
      {...others}
    >
      {data().firstLetter}
    </div>
  );
};
