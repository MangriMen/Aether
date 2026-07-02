import type { JSX } from 'solid-js';

export const ComponentShelf = (props: {
  title: string;
  children: JSX.Element;
}) => (
  <section class='mb-12'>
    <h2
      class='
        mb-4 text-sm font-bold text-muted-foreground tracking-[0.2em] uppercase
      '
    >
      {props.title}
    </h2>

    <div
      class='
      gap-6 rounded-xl bg-card/card p-8 shadow-sm flex flex-wrap border
    '
    >
      {props.children}
    </div>
  </section>
);
