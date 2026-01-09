import { openUrl } from '@tauri-apps/plugin-opener';

import type { TFunction } from '@/shared/model';

import { showToast } from '@/shared/ui';

const maxTextLengthForTranslatorUrl = 5000;

export const openTranslationInBrowser = async (text: string, t: TFunction) => {
  const { processedText, isTruncated } = preprocessTextForTranslation(text);

  const sourceLang = 'en';
  const targetLang = 'ru';

  // Format the URL for Google Translate
  const encodedText = encodeURIComponent(processedText);
  const url = getGoogleTranslateUrl(encodedText, sourceLang, targetLang);

  try {
    await openUrl(url);

    if (isTruncated) {
      showToast({
        title: t('update.changelogIsTooLong'),
        description: t('update.changelogIsTooLongDescription'),
        variant: 'warningFilled',
      });
    }
  } catch (err) {
    showToast({
      title: t('common.failedToOpenBrowser'),
      description: t('update.failedToOpenBrowserWithChangelogDescription'),
      variant: 'destructive',
    });
    // eslint-disable-next-line no-console
    console.error('Browser open error:', err);
  }
};

const getGoogleTranslateUrl = (
  encodedText: string,
  sourceLang: string,
  targetLang: string,
) =>
  `https://translate.google.com/?sl=${sourceLang}&tl=${targetLang}&text=${encodedText}&op=translate`;

const preprocessTextForTranslation = (
  text: string,
): { processedText: string; isTruncated: boolean } => {
  if (text.length > maxTextLengthForTranslatorUrl) {
    const processedText = text.substring(0, maxTextLengthForTranslatorUrl);
    return { processedText, isTruncated: true };
  }

  return { processedText: text, isTruncated: false };
};
