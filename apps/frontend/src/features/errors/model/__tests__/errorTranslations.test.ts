import * as i18n from '@solid-primitives/i18n';
import { describe, expect, it } from 'vitest';

import bindings from '@/platform/bindings/application.ts?raw';
import pluginBindings from '@/platform/bindings/plugin.ts?raw';
import { enDictionary, ruDictionary } from '@/shared/i18n';

import type { LauncherError } from '../launcherError';

import { getErrorDetails, getTranslatedError } from '../launcherError';

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

const tEn = i18n.translator(
  () => i18n.flatten(enDictionary),
  i18n.resolveTemplate,
);
const tRu = i18n.translator(
  () => i18n.flatten(ruDictionary),
  i18n.resolveTemplate,
);

const io = (cause: unknown) =>
  ({
    type: 'application',
    payload: { code: 'IO', payload: { cause } },
  }) as unknown as LauncherError;

/** Texts not yet moved off raw backend text; the list shrinks to empty as DTOs get a `cause`. */
const PENDING_RAW_TEXT_KEYS = ['errors.plugin.MANIFEST'];

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

  it('should_append_the_translated_cause_when_the_error_has_one', () => {
    const cases = [
      [
        { type: 'io', kind: 'notFound' },
        'File system error: file or folder not found',
      ],
      [
        { type: 'io', kind: 'readOnlyFilesystem' },
        'File system error: the disk is read-only',
      ],
      [
        { type: 'network', kind: 'timeout', url: null },
        'File system error: the server did not respond in time',
      ],
      [
        { type: 'httpStatus', status: 503, url: 'https://x' },
        'File system error: the server answered 503',
      ],
      [
        { type: 'parse', line: 3, column: 7 },
        'File system error: the data has an invalid format (line 3, column 7)',
      ],
      [
        { type: 'parse', line: null, column: null },
        'File system error: the data has an invalid format',
      ],
      [{ type: 'database' }, 'File system error: launcher database error'],
    ] as const;

    for (const [cause, expected] of cases) {
      expect(getTranslatedError(io(cause), tEn)).toBe(expected);
    }
  });

  it('should_translate_the_cause_into_russian', () => {
    const text = getTranslatedError(io({ type: 'io', kind: 'notFound' }), tRu);

    expect(text).toBe('Ошибка файловой системы: файл или папка не найдены');
  });

  it('should_leave_the_main_text_alone_when_the_cause_is_unknown', () => {
    const text = getTranslatedError(
      io({ type: 'unknown', details: 'secret raw text' }),
      tEn,
    );

    expect(text).toBe('File system error');
  });

  it('should_translate_an_internal_error_without_its_raw_text', () => {
    const error = {
      type: 'application',
      payload: {
        code: 'INTERNAL',
        payload: { cause: { type: 'unknown', details: 'boom' } },
      },
    } as unknown as LauncherError;

    expect(getTranslatedError(error, tEn)).toBe('Unexpected error');
  });

  it('should_replace_the_reason_argument_with_its_translation', () => {
    const t = (key: string, args?: Record<string, unknown>) =>
      key === 'errors.instance.reason.INVALID.cannot_be_empty'
        ? `empty:${String(args?.field)}`
        : `${key}|${JSON.stringify(args)}`;
    const error = {
      type: 'instance',
      payload: {
        code: 'INVALID',
        payload: {
          reason: { code: 'cannot_be_empty', payload: { field: 'name' } },
        },
      },
    } as unknown as LauncherError;

    expect(getTranslatedError(error, t as never)).toBe(
      'errors.instance.INVALID|{"reason":"empty:name"}',
    );
  });
});

describe('field names', () => {
  const pagination = (field: string): LauncherError =>
    ({
      type: 'instance',
      payload: {
        code: 'INVALID_PAGINATION',
        payload: {
          field,
          value: '5000',
          reason: { code: 'TOO_LARGE', payload: { max: 1000 } },
        },
      },
    }) as unknown as LauncherError;

  it('should_name_the_field_in_the_text_when_it_has_a_translation', () => {
    expect(getTranslatedError(pagination('PAGE_SIZE'), tEn)).toBe(
      'Invalid page size value 5000: it must be at most 1000',
    );
    expect(getTranslatedError(pagination('PAGE'), tRu)).toContain('страница');
  });

  it('should_keep_the_raw_name_when_the_field_has_no_translation', () => {
    expect(getTranslatedError(pagination('OTHER'), tEn)).toContain(
      'Invalid OTHER value',
    );
  });
});

describe('the cause of a reason', () => {
  const download = (cause: unknown): LauncherError =>
    ({
      type: 'instance',
      payload: {
        code: 'CONTENT_DOWNLOAD_ERROR',
        payload: {
          reason: {
            code: 'DOWNLOAD',
            payload: { url: 'https://x.test/a.jar', cause },
          },
        },
      },
    }) as unknown as LauncherError;

  it('should_append_the_category_of_the_cause_of_the_reason', () => {
    expect(
      getTranslatedError(
        download({ type: 'httpStatus', status: 404, url: 'https://x.test' }),
        tEn,
      ),
    ).toBe(
      'Failed to download instance content: could not download https://x.test/a.jar: the server answered 404',
    );
  });

  it('should_find_the_raw_text_of_an_unknown_cause_of_the_reason', () => {
    expect(
      getErrorDetails(download({ type: 'unknown', details: 'denied' })),
    ).toBe('denied');
  });
});

describe('getErrorDetails', () => {
  it('should_return_the_raw_text_of_an_unknown_cause', () => {
    expect(getErrorDetails(io({ type: 'unknown', details: 'raw' }))).toBe(
      'raw',
    );
  });

  it('should_find_the_raw_text_in_a_nested_error', () => {
    const nested = {
      type: 'instance',
      payload: {
        code: 'FAILED',
        payload: { error: io({ type: 'unknown', details: 'deep' }) },
      },
    };

    expect(getErrorDetails(nested)).toBe('deep');
  });

  it('should_return_nothing_for_a_categorized_cause_or_a_foreign_value', () => {
    expect(
      getErrorDetails(io({ type: 'io', kind: 'notFound' })),
    ).toBeUndefined();
    expect(getErrorDetails(new Error('x'))).toBeUndefined();
  });
});

describe('error dictionaries', () => {
  it.each(dictionaries)(
    'no errors.* text in %s inserts raw backend text',
    (_lang, dictionary) => {
      const offenders = Object.entries(i18n.flatten(dictionary.errors as never))
        .filter(([, text]) =>
          /\{\{(details|message|error)\}\}/.test(String(text)),
        )
        .map(([key]) => `errors.${key}`);

      expect(offenders).toEqual(PENDING_RAW_TEXT_KEYS);
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

describe('instance reason translations', () => {
  /** `{ code: "X"; payload: { ..., reason: SomeReasonDto } }` → `{ X: 'SomeReasonDto' }` */
  const getReasonDtos = (): Record<string, string> => {
    const start = bindings.indexOf('export type InstanceErrorDto =');
    const rest = bindings.slice(start + 1);
    const end = rest.search(/\nexport type /);
    const body = rest.slice(0, end === -1 ? undefined : end);
    const result: Record<string, string> = {};

    for (const m of body.matchAll(
      /\{ code: "(\w+)"; payload: \{[^}]*?reason: (\w+Dto),/gs,
    )) {
      result[m[1]!] = m[2]!;
    }

    return result;
  };

  const reasonDtos = getReasonDtos();

  it('finds the errors with a reason in the bindings', () => {
    expect(Object.keys(reasonDtos)).toContain('INVALID_RELATIVE_PATH');
    expect(Object.keys(reasonDtos)).toContain('CONTENT_PROVIDER_ERROR');
    expect(Object.keys(reasonDtos)).toContain('CONTENT_DOWNLOAD_ERROR');
  });

  it.each(dictionaries)(
    'every instance reason code is translated in %s',
    (_lang, dictionary) => {
      const reasons = dictionary.errors.instance?.reason as unknown as
        | Record<string, Record<string, string>>
        | undefined;
      const missing: string[] = [];

      for (const [code, dtoName] of Object.entries(reasonDtos)) {
        for (const reasonCode of getCodes(dtoName)) {
          if (!reasons?.[code]?.[reasonCode]) {
            missing.push(`errors.instance.reason.${code}.${reasonCode}`);
          }
        }
      }

      expect(missing).toEqual([]);
    },
  );
});
