/* eslint-disable solid/reactivity -- a React component rendered by Storybook, not a Solid one */
import { DocsContainer } from '@storybook/addon-docs/blocks';
import { jsx, themes } from 'storybook/theming';

import { THEMES } from '../src/features/appearance/model/theme';

type DocsContainerProps = Parameters<typeof DocsContainer>[0];

// Docs pages are rendered by Storybook itself, so they follow the toolbar theme only through this container
export const ThemedDocsContainer = (props: DocsContainerProps) => {
  const { context, children } = props;
  const value = context.getStoryContext(context.storyById()).globals['theme'];
  const mode = THEMES.find((theme) => theme.value === value)?.mode ?? 'dark';

  return jsx(DocsContainer, {
    context,
    theme: mode === 'light' ? themes.light : themes.dark,
    children,
  });
};
