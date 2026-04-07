import { writeText } from '@tauri-apps/plugin-clipboard-manager';
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

  const handleCopyText = () => {
    try {
      writeText(local.changelogBody);
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
    <div class={cn('mt-2 flex flex-col gap-2', local.class)} {...others}>
      <div class='flex items-end gap-2'>
        <h3 class='text-xl font-medium'>{t('settings.whatsNew')}</h3>
        <CombinedTooltip
          label={t('common.copyToClipboard')}
          as={IconButton}
          class='pr-[3px]'
          size='sm'
          variant='secondary'
          icon={IconMdiContentCopy}
          onClick={handleCopyText}
        />
        <CombinedTooltip
          label={t('update.translateChangelogInBrowser')}
          as={IconButton}
          class='pr-[3px]'
          size='sm'
          variant='secondary'
          icon={IconMdiTranslate}
          onClick={handleTranslateInBrowser}
        />
      </div>
      <div class='max-w-full overflow-auto rounded-md border border-secondary-dark dark:border-secondary'>
        <MarkdownRenderer class='p-4' children={local.changelogBody} />
      </div>
    </div>
  );
};
