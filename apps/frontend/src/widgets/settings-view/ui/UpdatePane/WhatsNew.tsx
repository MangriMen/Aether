import IconMdiContentCopy from '~icons/mdi/content-copy';
import IconMdiTranslate from '~icons/mdi/translate';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  CombinedTooltip,
  IconButton,
  MarkdownRenderer,
  showToast,
} from '@/shared/ui';

import { openTranslationInBrowser } from '../../lib';

export type WhatsNewProps = ComponentProps<'div'> & {
  changelogBody: string;
};

export const WhatsNew: Component<WhatsNewProps> = (props) => {
  const [local, others] = splitProps(props, ['changelogBody', 'class']);

  const [{ t }] = useTranslation();

  const handleCopyText = async () => {
    try {
      await navigator.clipboard.writeText(local.changelogBody);
      showToast({
        title: t('update.changelogCopiedToClipboard'),
        variant: 'success',
      });
    } catch {
      showToast({
        title: t('update.failedToCopyChangelogToClipboard'),
        variant: 'destructive',
      });
    }
  };

  const handleTranslateInBrowser = () => {
    openTranslationInBrowser(local.changelogBody, t);
  };

  return (
    <div class={cn('mt-2 gap-2 flex flex-col', local.class)} {...others}>
      <div class='gap-2 flex items-end'>
        <h3 class='text-xl font-medium'>{t('settings.whatsNew')}</h3>
        <CombinedTooltip
          label={t('common.copyToClipboard')}
          as={IconButton}
          class='pr-0.75'
          size='sm'
          variant='secondary'
          icon={IconMdiContentCopy}
          onClick={handleCopyText}
        />
        <CombinedTooltip
          label={t('update.translateChangelogInBrowser')}
          as={IconButton}
          class='pr-0.75'
          size='sm'
          variant='secondary'
          icon={IconMdiTranslate}
          onClick={handleTranslateInBrowser}
        />
      </div>
      <div class='rounded-md bg-card/card max-w-full overflow-auto border'>
        <MarkdownRenderer class='p-4' children={local.changelogBody} />
      </div>
    </div>
  );
};
