/* eslint-disable solid/reactivity -- a React component rendered by Storybook, not a Solid one */
import { DocsContainer } from '@storybook/addon-docs/blocks';
import { jsx, themes } from 'storybook/theming';

import { getBackground } from './backgrounds';

type DocsContainerProps = Parameters<typeof DocsContainer>[0];

// Docs pages are rendered by Storybook itself, so they follow the background switcher only through this container
export const ThemedDocsContainer = (props: DocsContainerProps) => {
  const { context, children } = props;
  const { mode } = getBackground(
    context.getStoryContext(context.storyById()).globals,
  );

  return jsx(DocsContainer, {
    context,
    theme: mode === 'light' ? themes.light : themes.dark,
    children,
  });
};
