import { Show, type Component, type JSX } from 'solid-js';

export type AllowedItemLeadingItemsProps = {
  items?: JSX.Element;
};

/** Row actions, revealed when the row is hovered. */
export const AllowedItemLeadingItems: Component<
  AllowedItemLeadingItemsProps
> = (props) => (
  <Show when={props.items}>
    {(items) => (
      <div
        class='
          flex shrink-0 items-center opacity-0 transition-opacity
          group-hover:opacity-100
        '
      >
        {items()}
      </div>
    )}
  </Show>
);
