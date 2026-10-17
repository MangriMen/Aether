import { describe, expect, it } from 'vitest';

import bindings from '@/platform/bindings/application.ts?raw';
import pluginBindings from '@/platform/bindings/plugin.ts?raw';
import { enDictionary, ruDictionary } from '@/shared/i18n';

/**
 * Error codes are only types in the generated bindings, so a new Rust error
 * variant is invisible at runtime. These tests read the bindings as text and
 * fail with a readable list of missing keys instead of a wall of TS errors
 * from the production build.
 */

type Dictionary = {
  errors: Record<string, Record<string, string>>;
  plugin: { stateReason?: Record<string, string> };
};

/** `{ type: "instance"; payload: InstanceErrorDto }` → `{ instance: 'InstanceErrorDto' }` */
const getErrorGroups = (): Record<string, string> => {
  const start = bindings.indexOf('export type FrontendError =');
  const tail = bindings.slice(start);
  const union = tail.slice(0, tail.search(/;\r?\n\r?\n/));
  const groups: Record<string, string> = {};

  for (const m of union.matchAll(/\{ type: "(\w+)"; payload: (\w+) \}/g)) {
    groups[m[1]!] = m[2]!;
  }

  return groups;
};

const getCodes = (dtoName: string, source = bindings): string[] => {
  const start = source.indexOf(`export type ${dtoName} =`);
  const rest = source.slice(start + 1);
  const end = rest.search(/\nexport type /);
  const body = rest.slice(0, end === -1 ? undefined : end);

  return [...body.matchAll(/\{ code: "(\w+)"/g)].map((m) => m[1]!);
};

const dictionaries: [string, Dictionary][] = [
  ['en', enDictionary as unknown as Dictionary],
  ['ru', ruDictionary as unknown as Dictionary],
];

describe('error translations', () => {
  const groups = getErrorGroups();

  it('finds the error groups in the bindings', () => {
    expect(Object.keys(groups).length).toBeGreaterThan(5);
  });

  it.each(dictionaries)(
    'every backend error code is translated in %s',
    (_lang, dictionary) => {
      const missing: string[] = [];

      for (const [group, dtoName] of Object.entries(groups)) {
        for (const code of getCodes(dtoName)) {
          if (!dictionary.errors[group]?.[code]) {
            missing.push(`errors.${group}.${code}`);
          }
        }
      }

      expect(missing).toEqual([]);
    },
  );
});

describe('plugin state reason translations', () => {
  const codes = getCodes('PluginStateReasonDto', pluginBindings);

  it('finds the reason codes in the bindings', () => {
    expect(codes.length).toBeGreaterThan(0);
  });

  it.each(dictionaries)(
    'every plugin state reason code is translated in %s',
    (_lang, dictionary) => {
      const missing = codes
        .filter((code) => !dictionary.plugin.stateReason?.[code])
        .map((code) => `plugin.stateReason.${code}`);

      expect(missing).toEqual([]);
    },
  );
});
