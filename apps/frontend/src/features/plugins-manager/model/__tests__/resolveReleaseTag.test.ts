import { describe, expect, it } from 'vitest';

import { resolveReleaseTag } from '../resolveReleaseTag';

const releases = [{ tag_name: 'v2' }, { tag_name: 'v1' }];

describe('resolveReleaseTag', () => {
  it('keeps the picked tag when the repository has it', () => {
    expect(resolveReleaseTag(releases, 'v1')).toBe('v1');
  });

  it('falls back to the newest release when nothing is picked', () => {
    expect(resolveReleaseTag(releases, '')).toBe('v2');
  });

  it('drops a tag that belongs to another repository', () => {
    expect(resolveReleaseTag(releases, 'v9-from-other-repo')).toBe('v2');
  });

  it('returns an empty tag when there are no releases', () => {
    expect(resolveReleaseTag([], 'v1')).toBe('');
  });
});
