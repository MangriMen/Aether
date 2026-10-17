import { describe, expect, it } from 'vitest';

import type { ContentFile } from '../contentFile';

import {
  getContentFileAuthor,
  getContentFileDisplayTitle,
  getContentFileIconSrc,
  getContentFileProjectHref,
  getContentFileTitle,
  isContentFileAuthorPending,
  getContentFileVersionSortValue,
  getContentFileVersionState,
  isLocalContentFile,
} from '../contentFile';

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

describe('isContentFileAuthorPending', () => {
  it('is pending only while the author is being fetched', () => {
    expect(
      isContentFileAuthorPending(makeFile({ metadataPending: true })),
    ).toBe(true);
    expect(
      isContentFileAuthorPending(makeFile({ metadataPending: false })),
    ).toBe(false);
    expect(
      isContentFileAuthorPending(
        makeFile({
          metadataPending: true,
          metadata: makeMetadata({ author: 'jellysquid3' }),
        }),
      ),
    ).toBe(false);
  });
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

const provider = { pluginId: 'modrinth', capabilityId: 'content' };

describe('local files', () => {
  it('has no metadata and no provider', () => {
    expect(isLocalContentFile(makeFile({}))).toBe(true);
    expect(isLocalContentFile(makeFile({ metadata: makeMetadata({}) }))).toBe(
      false,
    );
    expect(isLocalContentFile(makeFile({ updateProviderId: provider }))).toBe(
      false,
    );
  });

  it('shows the file name as the title', () => {
    expect(getContentFileDisplayTitle(makeFile({ name: 'Jar name' }))).toBe(
      'a.jar',
    );
  });

  it('shows the project title otherwise, then the file name', () => {
    expect(
      getContentFileDisplayTitle(
        makeFile({ metadata: makeMetadata({ name: 'Project' }) }),
      ),
    ).toBe('Project');
    expect(
      getContentFileDisplayTitle(makeFile({ updateProviderId: provider })),
    ).toBe('a.jar');
  });
});

describe('getContentFileAuthor', () => {
  it('returns the author or nothing', () => {
    expect(
      getContentFileAuthor(
        makeFile({ metadata: makeMetadata({ author: 'Alice' }) }),
      ),
    ).toBe('Alice');
    expect(
      getContentFileAuthor(makeFile({ metadata: makeMetadata({}) })),
    ).toBeUndefined();
    expect(getContentFileAuthor(makeFile({}))).toBeUndefined();
  });
});

describe('version helpers', () => {
  const versionMetadata = {
    versionId: 'v1',
    versionNumber: '1.2.3',
    name: 'N',
  };

  it('derives the state', () => {
    expect(getContentFileVersionState(makeFile({ versionMetadata }))).toBe(
      'known',
    );
    expect(
      getContentFileVersionState(
        makeFile({ updateProviderId: provider, versionMetadataPending: true }),
      ),
    ).toBe('pending');
    // A provider that has nothing more to give.
    expect(
      getContentFileVersionState(makeFile({ updateProviderId: provider })),
    ).toBe('unknown');
    expect(getContentFileVersionState(makeFile({}))).toBe('unknown');
  });

  it('sorts by version number, then file name', () => {
    expect(getContentFileVersionSortValue(makeFile({ versionMetadata }))).toBe(
      '1.2.3',
    );
    expect(getContentFileVersionSortValue(makeFile({}))).toBe('a.jar');
  });
});

describe('getContentFileProjectHref', () => {
  it('links to the project with provider and instance', () => {
    const file = makeFile({
      updateProviderId: provider,
      update: { 'modrinth#content': { contentId: 'abc', version: 'v' } },
    });
    expect(getContentFileProjectHref(file, 'inst')).toBe(
      '/content/abc?instance=inst&plugin=modrinth&capability=content',
    );
  });

  it('has no link without a provider or content id', () => {
    expect(getContentFileProjectHref(makeFile({}))).toBeUndefined();
    expect(
      getContentFileProjectHref(makeFile({ updateProviderId: provider })),
    ).toBeUndefined();
  });
});
