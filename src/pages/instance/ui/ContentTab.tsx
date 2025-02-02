import type { Instance } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslate } from '@/shared/model';
import {
  createMemo,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

export type ContentTabProps = ComponentProps<'div'> & {
  instance: Instance;
};

export const ContentTab: Component<ContentTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslate();

  const content = createMemo(() => undefined);

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <Show
        when={content()}
        fallback={
          <span class='mx-auto mt-20 text-lg text-muted-foreground'>
            {t('instance.noContent')}
          </span>
        }
      >
        Contents placeholder
      </Show>
    </div>
  );
};
