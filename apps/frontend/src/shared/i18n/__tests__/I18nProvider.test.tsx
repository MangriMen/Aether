import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { LOCALE_LS_KEY } from '../i18nConstants';
import { I18nProvider } from '../I18nProvider';
import { useTranslation } from '../index';
import { LOCALE_RESOURCES } from '../locales';

const Probe = () => {
  const [{ locale }] = useTranslation();
  return <span id='locale'>{locale()}</span>;
};

const renderProvider = () => {
  const host = document.createElement('div');
  document.body.append(host);
  render(
    () => (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale='en'>
        <Probe />
      </I18nProvider>
    ),
    host,
  );
  return host.querySelector('#locale')?.textContent;
};

describe('I18nProvider', () => {
  afterEach(() => {
    localStorage.clear();
    document.body.replaceChildren();
  });

  it.each(['"xx"', 'not json', '42', '"constructor"'])(
    'should fall back to a known locale when the stored value is %s',
    (stored) => {
      localStorage.setItem(LOCALE_LS_KEY, stored);

      const locale = renderProvider();

      expect(Object.keys(LOCALE_RESOURCES)).toContain(locale);
    },
  );

  it('should keep a stored locale when it is one of ours', () => {
    localStorage.setItem(LOCALE_LS_KEY, '"ru"');

    const locale = renderProvider();

    expect(locale).toBe('ru');
    expect(document.documentElement.lang).toBe('ru');
  });
});
