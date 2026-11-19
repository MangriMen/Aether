import { describe, expect, it } from 'vitest';

const files = import.meta.glob('../*.ts', {
  eager: true,
  import: 'default',
  query: '?raw',
}) as Record<string, string>;

describe('generated bindings', () => {
  it('has bindings to check', () => {
    expect(Object.keys(files).length).toBeGreaterThan(5);
  });

  // ARCHITECTURE §12.3: fields are camelCase. A snake_case field means a struct lost its
  // `#[serde(rename_all = "camelCase")]`.
  it.each(Object.entries(files))('has no snake_case field in %s', (_name, text) => {
    const fields = [...text.matchAll(/^\s*"?([a-z]+(?:_[a-z0-9]+)+)"?\??:/gm)].map(
      (m) => m[1],
    );

    expect(fields).toEqual([]);
  });
});
