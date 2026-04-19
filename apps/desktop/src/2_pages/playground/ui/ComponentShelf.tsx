import type { JSX } from 'solid-js';

export const ComponentShelf = (props: {
  title: string;
  children: JSX.Element;
}) => (
  <section class='mb-12'>
    <h2 class='mb-4 text-sm font-bold uppercase tracking-[0.2em] text-muted-foreground'>
      {props.title}
    </h2>

    <div class='flex flex-wrap gap-6 rounded-xl border bg-card/card p-8 shadow-sm'>
      {props.children}
    </div>
  </section>
);
