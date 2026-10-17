import type { JSX } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import {
  UiKitLabelsProvider,
  type UiKitLabelsProviderProps,
} from '@/shared/ui-kit';

export type UiKitLabelsFromI18nProps = { children?: JSX.Element };

/** Gives ui-kit its texts in the current locale. */
export const UiKitLabelsFromI18n = (props: UiKitLabelsFromI18nProps) => {
  const [{ t }] = useTranslation();

  const labels: UiKitLabelsProviderProps['labels'] = {
    get close() {
      return t('common.close');
    },
    get noData() {
      return t('common.noData');
    },
    get morePages() {
      return t('pagination.morePages');
    },
    get previous() {
      return t('pagination.previous');
    },
    get next() {
      return t('pagination.next');
    },
    get ok() {
      return t('common.ok');
    },
    get cancel() {
      return t('common.cancel');
    },
    get browse() {
      return t('common.browse');
    },
  };

  return (
    <UiKitLabelsProvider labels={labels}>{props.children}</UiKitLabelsProvider>
  );
};
