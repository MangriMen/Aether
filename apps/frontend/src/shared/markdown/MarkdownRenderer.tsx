import type { Component, ComponentProps } from 'solid-js';

import { createResource, Show, splitProps } from 'solid-js';

import { ProgressCircle } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { lazyParseMarkdown, lazySanitizeText } from './parseMarkdown';

export type MarkdownRendererProps = ComponentProps<'div'> & {
  children: string;
};

export const MarkdownRenderer: Component<MarkdownRendererProps> = (props) => {
  const [local, others] = splitProps(props, ['children', 'class']);

  const [sanitizedMarkdown] = createResource(
    () => local.children,
    async (text) => {
      const parsed = await lazyParseMarkdown(text);
      return lazySanitizeText(parsed);
    },
  );

  return (
    <Show
      when={Boolean(sanitizedMarkdown.latest)}
      fallback={<ProgressCircle />}
    >
      <div
        class={cn('prose dark:prose-invert max-w-none markdown', local.class)}
        // Mute warning because we are use sanitized text in inner html
        // eslint-disable-next-line solid/no-innerhtml
        innerHTML={sanitizedMarkdown.latest}
        {...others}
      />
    </Show>
  );
};
