import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { MarkdownRenderer } from '@/shared/ui';

export type WhatsNewProps = ComponentProps<'div'> & {
  changelogBody: string;
};

export const WhatsNew: Component<WhatsNewProps> = (props) => {
  const [local, others] = splitProps(props, ['changelogBody', 'class']);

  const [{ t }] = useTranslation();

  return (
    <div class={cn('mt-2 flex flex-col gap-2', local.class)} {...others}>
      <h3 class='text-xl font-medium'>{t('settings.whatsNew')}</h3>
      <div class='max-w-full overflow-auto rounded-md border border-secondary-dark dark:border-secondary'>
        <MarkdownRenderer class='p-4' children={local.changelogBody} />
      </div>
    </div>
  );
};
