import * as i18n from '@solid-primitives/i18n';
import { describe, expect, it } from 'vitest';

import bindings from '@/platform/bindings/application.ts?raw';
import pluginBindings from '@/platform/bindings/plugin.ts?raw';
import { enDictionary, ruDictionary } from '@/shared/i18n';

import type { LauncherError } from '../launcherError';

import { getTranslatedError } from '../launcherError';

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

  it('should_pick_the_io_message_by_kind_when_the_error_has_a_kind', () => {
    const t = i18n.translator(
      () => i18n.flatten(enDictionary),
      i18n.resolveTemplate,
    );
    const cases = [
      ['notFound', 'A file or folder was not found'],
      ['permissionDenied', 'Access to a file or folder was denied'],
      ['other', 'File system error: disk full'],
    ] as const;

    for (const [kind, expected] of cases) {
      const error = {
        type: 'application',
        payload: {
          code: 'IO',
          payload: { details: 'disk full', kind },
        },
      } as unknown as LauncherError;

      expect(getTranslatedError(error, t)).toBe(expected);
    }
  });
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
