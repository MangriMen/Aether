import { describe, expect, it } from 'vitest';

import type { ContentFile } from '../contentFile';

import { getContentFileIconSrc, getContentFileTitle } from '../contentFile';

const makeFile = (overrides: Partial<ContentFile>): ContentFile =>
  ({
    fileName: 'a.jar',
    name: null,
    metadata: null,
    ...overrides,
  }) as ContentFile;

const makeMetadata = (
  overrides: Partial<NonNullable<ContentFile['metadata']>>,
): NonNullable<ContentFile['metadata']> => ({
  contentId: 'id',
  slug: 'slug',
  name: 'Project',
  description: null,
  author: null,
  iconUrl: null,
  iconPath: null,
  ...overrides,
});

describe('getContentFileTitle', () => {
  it('prefers the project name', () => {
    expect(
      getContentFileTitle(
        makeFile({ name: 'File', metadata: makeMetadata({ name: 'Project' }) }),
      ),
    ).toBe('Project');
  });

  it('falls back to the file name, then nothing', () => {
    expect(getContentFileTitle(makeFile({ name: 'File' }))).toBe('File');
    expect(getContentFileTitle(makeFile({}))).toBeUndefined();
  });
});

describe('getContentFileIconSrc', () => {
  it('prefers the cached path over the remote url', () => {
    const metadata = makeMetadata({
      iconPath: 'asset://p',
      iconUrl: 'https://u',
    });
    expect(getContentFileIconSrc(makeFile({ metadata }))).toBe('asset://p');
  });

  it('falls back to the url, then nothing', () => {
    const metadata = makeMetadata({ iconUrl: 'https://u' });
    expect(getContentFileIconSrc(makeFile({ metadata }))).toBe('https://u');
    expect(getContentFileIconSrc(makeFile({}))).toBeUndefined();
  });
});
