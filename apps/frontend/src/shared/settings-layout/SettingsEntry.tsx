import type { Component, ComponentProps, JSX } from 'solid-js';

import {
  createContext,
  createMemo,
  createUniqueId,
  Show,
  splitProps,
  useContext,
} from 'solid-js';

import { FieldLabel } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

export type SettingsEntryProps = ComponentProps<'div'> & {
  title?: string | JSX.Element;
  description?: string | JSX.Element;
  variant?: 'card';
};

const SettingsEntryTitleContext = createContext<() => string | undefined>();

/** Props naming a control after the title of the settings entry it sits in. */
export const useSettingsEntryLabel = (): { 'aria-labelledby'?: string } => {
  const titleId = useContext(SettingsEntryTitleContext);

  return {
    get 'aria-labelledby'() {
      return titleId?.();
    },
  };
};

export const SettingsEntry: Component<SettingsEntryProps> = (props) => {
  const [local, others] = splitProps(props, [
    'title',
    'description',
    'variant',
    'class',
    'children',
  ]);

  const titleId = createUniqueId();
  const labelId = createMemo(() => (local.title ? titleId : undefined));

  return (
    <SettingsEntryTitleContext.Provider value={labelId}>
      <div
        class={cn(
          `
          gap-8
          @sm/settings-pane:flex-row @sm/settings-pane:items-center
          flex w-full flex-col justify-between
        `,
          local.class,
          {
            'bg-card/card p-4 rounded-md border': local.variant === 'card',
          },
        )}
        {...others}
      >
        <div class='text-pretty'>
          <Show when={local.title}>
            <FieldLabel id={titleId} class='text-lg font-medium'>
              {local.title}
            </FieldLabel>
          </Show>
          <Show when={local.description}>
            <p class='text-sm text-muted-foreground whitespace-pre-line'>
              {local.description}
            </p>
          </Show>
        </div>
        {local.children}
      </div>
    </SettingsEntryTitleContext.Provider>
  );
};
