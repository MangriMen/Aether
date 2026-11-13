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
    const t = (key: string, args?: Record<string, unknown>) => {
      if (key === 'errors.instance.reason.INVALID.cannot_be_empty') {
        return `empty:${String(args?.field)}`;
      }

      return key.startsWith('errors.instance.field.')
        ? undefined
        : `${key}|${JSON.stringify(args)}`;
    };

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

  it('should_name_the_field_in_the_text_of_a_reason', () => {
    const error = {
      type: 'instance',
      payload: {
        code: 'CONTENT_PROVIDER_ERROR',
        payload: {
          reason: {
            code: 'RESPONSE_INVALID_PAGINATION',
            payload: { field: 'PAGE_SIZE', value: '0' },
          },
        },
      },
    } as unknown as LauncherError;

    expect(getTranslatedError(error, tEn)).toBe(
      'Content provider error: the provider answered with an invalid page size 0',
    );
  });

  it('should_name_what_a_missing_path_is_the_path_of', () => {
    const error = {
      type: 'minecraft',
      payload: {
        code: 'PATH_NOT_FOUND',
        payload: { path: '/game/natives', entity_type: 'NATIVE_LIBRARY' },
      },
    } as unknown as LauncherError;

    expect(getTranslatedError(error, tEn)).toBe(
      'Not found: native libraries folder /game/natives',
    );
    expect(getTranslatedError(error, tRu)).toBe(
      'Не найдено: папка нативных библиотек /game/natives',
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

      expect(offenders).toEqual([]);
    },
  );
});

describe('plugin state reason translations', () => {
  const codes = getCodes('PluginStateReasonDto', pluginBindings);

  it('finds the reason codes in the bindings', () => {
    expect(codes.length).toBeGreaterThan(0);
  });

  it.each(dictionaries)(
    'no plugin state reason text in %s inserts raw backend text',
    (_lang, dictionary) => {
      const offenders = Object.entries(dictionary.plugin.stateReason ?? {})
        .filter(([, text]) => /\{\{(details|message|error)\}\}/.test(text))
        .map(([key]) => `plugin.stateReason.${key}`);

      expect(offenders).toEqual([]);
    },
  );

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

describe.each([
  ['appSettings', 'AppSettingsErrorDto', 'CAN_NOT_SET_EFFECT'],
  ['instance', 'InstanceErrorDto', 'CONTENT_PROVIDER_ERROR'],
  ['minecraft', 'MinecraftErrorDto', 'PROCESSOR_FAILED'],
  ['plugin', 'PluginErrorDto', 'LOAD_FAILED'],
  ['settings', 'SettingsErrorDto', 'INVALID_PLATFORM_PROVIDER'],
] as const)('%s reason translations', (group, errorDtoName, sampleCode) => {
  /** `{ code: "X"; payload: { ..., reason: SomeReasonDto } }` → `{ X: 'SomeReasonDto' }` */
  const getReasonDtos = (): Record<string, string> => {
    const start = bindings.indexOf(`export type ${errorDtoName} =`);
    const rest = bindings.slice(start + 1);
    const end = rest.search(/\nexport type /);
    const body = rest.slice(0, end === -1 ? undefined : end);
    const result: Record<string, string> = {};

    for (const m of body.matchAll(
      /\{ code: "(\w+)"; payload: \{[^}]*?reason: (\w+Dto)\b/gs,
    )) {
      result[m[1]!] = m[2]!;
    }

    return result;
  };

  const reasonDtos = getReasonDtos();

  it('finds the errors with a reason in the bindings', () => {
    expect(Object.keys(reasonDtos)).toContain(sampleCode);
  });

  it.each(dictionaries)(
    `every ${group} reason code is translated in %s`,
    (_lang, dictionary) => {
      const reasons = dictionary.errors[group]?.reason as unknown as
        | Record<string, Record<string, string>>
        | undefined;
      const missing: string[] = [];

      for (const [code, dtoName] of Object.entries(reasonDtos)) {
        for (const reasonCode of getCodes(dtoName)) {
          if (!reasons?.[code]?.[reasonCode]) {
            missing.push(`errors.${group}.reason.${code}.${reasonCode}`);
          }
        }
      }

      expect(missing).toEqual([]);
    },
  );
});

describe('minecraft processor failure', () => {
  const failed = (reason: unknown) =>
    ({
      type: 'minecraft',
      payload: { code: 'PROCESSOR_FAILED', payload: { reason } },
    }) as unknown as LauncherError;

  it('should_translate_the_reason_and_the_category_of_its_cause', () => {
    const error = failed({
      code: 'SPAWN_FAILED',
      payload: { cause: { type: 'io', kind: 'notFound' } },
    });

    expect(getTranslatedError(error, tEn)).toBe(
      'Version processing failed: the processor could not be started: file or folder not found',
    );
  });

  it('should_keep_the_output_of_the_processor_out_of_the_text', () => {
    const error = failed({
      code: 'EXITED',
      payload: { code: 1, cause: { type: 'unknown', details: 'stack trace' } },
    });

    expect(getTranslatedError(error, tRu)).toBe(
      'Ошибка обработки версии: процессор завершился с кодом 1',
    );
    expect(getErrorDetails(error)).toBe('stack trace');
  });
});

describe('plugin manifest errors', () => {
  const codes = getCodes('ManifestErrorDto');

  it('finds the manifest error codes in the bindings', () => {
    expect(codes).toContain('UNKNOWN_HOST_FUNCTIONS');
  });

  it.each(dictionaries)(
    'every manifest error code is translated in %s',
    (_lang, dictionary) => {
      const manifest = dictionary.errors.plugin?.manifest as unknown as
        | Record<string, string>
        | undefined;

      expect(
        codes
          .filter((code) => !manifest?.[code])
          .map((code) => `errors.plugin.manifest.${code}`),
      ).toEqual([]);
    },
  );

  const manifestError = (error: unknown) =>
    ({
      type: 'plugin',
      payload: { code: 'MANIFEST', payload: { error } },
    }) as unknown as LauncherError;

  it('should_build_the_text_of_a_manifest_error_from_its_code_and_arguments', () => {
    expect(
      getTranslatedError(
        manifestError({
          code: 'INVALID_ID',
          payload: { field: 'metadata.id', value: 'Bad Id' },
        }),
        tEn,
      ),
    ).toBe(
      'The plugin manifest has an invalid id in metadata.id: "Bad Id". An id may contain only lowercase Latin letters, digits, "_" and "-", and must start with a letter or digit',
    );
  });

  it('should_quote_every_unknown_host_function_in_the_text', () => {
    expect(
      getTranslatedError(
        manifestError({
          code: 'UNKNOWN_HOST_FUNCTIONS',
          payload: { names: ['a', 'b'] },
        }),
        tEn,
      ),
    ).toContain('`a`, `b`');
  });
});

describe('plugin load failure', () => {
  const loadFailed = (reason: unknown) =>
    ({
      type: 'plugin',
      payload: {
        code: 'LOAD_FAILED',
        payload: { plugin_id: 'my-plugin', reason },
      },
    }) as unknown as LauncherError;

  it('should_translate_the_reason_and_keep_the_raw_text_of_a_build_failure_out_of_the_message', () => {
    const error = loadFailed({
      code: 'BUILD',
      payload: { cause: { type: 'unknown', details: 'wasmtime trap' } },
    });

    expect(getTranslatedError(error, tEn)).toBe(
      'Failed to load plugin my-plugin: the plugin could not be built',
    );
    expect(getErrorDetails(error)).toBe('wasmtime trap');
  });

  it('should_find_the_raw_text_of_a_manifest_error_nested_in_a_plugin_error', () => {
    const error = {
      type: 'plugin',
      payload: {
        code: 'MANIFEST',
        payload: {
          error: {
            code: 'INVALID_SEMVER',
            payload: {
              field: 'version',
              value: '1.x',
              cause: { type: 'unknown', details: 'unexpected character' },
            },
          },
        },
      },
    } as unknown as LauncherError;

    expect(getTranslatedError(error, tEn)).toBe(
      'The plugin manifest has an invalid version in version: "1.x"',
    );
    expect(getErrorDetails(error)).toBe('unexpected character');
  });

  it('should_translate_a_load_failure_without_arguments', () => {
    expect(
      getTranslatedError(loadFailed({ code: 'CONTAINER_UNAVAILABLE' }), tRu),
    ).toBe(
      'Не удалось загрузить плагин my-plugin: лаунчер ещё не готов запускать плагины',
    );
  });
});

describe('plugin call failure', () => {
  const callFailed = (reason: unknown) =>
    ({
      type: 'plugin',
      payload: {
        code: 'FUNCTION_CALL_FAILED',
        payload: { function_name: 'search', plugin_id: 'my-plugin', reason },
      },
    }) as unknown as LauncherError;

  it('should_name_the_limit_the_plugin_hit', () => {
    expect(getTranslatedError(callFailed({ code: 'TIMEOUT' }), tEn)).toBe(
      'Failed to call function search of plugin my-plugin: the plugin exceeded its time limit',
    );
  });

  it('should_keep_the_text_of_the_plugin_out_of_the_message', () => {
    const error = callFailed({
      code: 'PLUGIN',
      payload: { cause: { type: 'unknown', details: 'mod not found' } },
    });

    expect(getTranslatedError(error, tRu)).toBe(
      'Ошибка вызова функции search плагина my-plugin: плагин сообщил об ошибке',
    );
    expect(getErrorDetails(error)).toBe('mod not found');
  });
});

describe('plugin provider failure', () => {
  it('should_translate_the_reason_and_the_category_of_its_cause', () => {
    const error = {
      type: 'plugin',
      payload: {
        code: 'PROVIDER_FETCH_ERROR',
        payload: {
          source_type: 'github',
          reason: {
            code: 'REQUEST',
            payload: {
              cause: { type: 'httpStatus', status: 403, url: 'https://x' },
            },
          },
        },
      },
    } as unknown as LauncherError;

    expect(getTranslatedError(error, tEn)).toBe(
      'Failed to fetch plugin info from provider github: the request failed: the server answered 403',
    );
  });
});

describe('app settings errors', () => {
  it('should_translate_the_window_error_behind_a_failed_effect', () => {
    const error = {
      type: 'appSettings',
      payload: {
        code: 'CAN_NOT_SET_EFFECT',
        payload: { reason: { code: 'PLATFORM_NOT_SUPPORTED' } },
      },
    } as unknown as LauncherError;

    expect(getTranslatedError(error, tEn)).toBe(
      'Unable to apply the requested effect: the platform does not support it',
    );
  });
});

describe('settings errors', () => {
  it('should_translate_the_reason_why_a_provider_choice_is_invalid', () => {
    const error = {
      type: 'settings',
      payload: {
        code: 'INVALID_PLATFORM_PROVIDER',
        payload: {
          platform: 'modrinth',
          reason: { code: 'FORMAT', payload: { value: 'nope' } },
        },
      },
    } as unknown as LauncherError;

    expect(getTranslatedError(error, tEn)).toBe(
      'Invalid content provider choice for platform "modrinth": "nope" is not a provider id: write it as plugin#capability',
    );
  });
});
