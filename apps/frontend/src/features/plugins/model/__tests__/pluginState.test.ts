import * as i18n from '@solid-primitives/i18n';
import { describe, expect, it } from 'vitest';

import type { PluginStateReasonDto } from '@/platform/bindings/plugin';

import { enDictionary, ruDictionary } from '@/shared/i18n';

import {
  getStateReasonDetails,
  getTranslatedStateReason,
} from '../pluginState';

const tEn = i18n.translator(
  () => i18n.flatten(enDictionary),
  i18n.resolveTemplate,
);
const tRu = i18n.translator(
  () => i18n.flatten(ruDictionary),
  i18n.resolveTemplate,
);

const stateReason = (code: string, payload: unknown) =>
  ({ code, payload }) as unknown as PluginStateReasonDto;

describe('getTranslatedStateReason', () => {
  it('should_translate_an_invalid_manifest_reason_with_the_manifest_error', () => {
    const reason = stateReason('INVALID_MANIFEST', {
      error: {
        code: 'UNKNOWN_HOST_FUNCTIONS',
        payload: { names: ['spawn_shell', 'exec'] },
      },
    });

    expect(getTranslatedStateReason(reason, tEn)).toBe(
      'The plugin manifest failed validation: The plugin declares host functions this launcher does not provide: `spawn_shell`, `exec`. It was probably built for a newer version of the launcher',
    );
  });

  it('should_translate_a_manifest_error_without_arguments', () => {
    const reason = stateReason('INVALID_MANIFEST', {
      error: { code: 'RESERVED_PLUGIN_ID' },
    });

    expect(getTranslatedStateReason(reason, tRu)).toBe(
      'Манифест плагина не прошёл проверку: Id плагина зарезервирован для провайдеров, встроенных в Aether',
    );
  });

  it('should_translate_a_load_failed_reason_and_keep_its_raw_text_out_of_the_message', () => {
    const reason = stateReason('LOAD_FAILED', {
      reason: {
        code: 'BUILD',
        payload: { cause: { type: 'unknown', details: 'bad import' } },
      },
    });

    expect(getTranslatedStateReason(reason, tRu)).toBe(
      'Не удалось загрузить плагин: плагин не удалось собрать',
    );
    expect(getStateReasonDetails(reason)).toBe('bad import');
  });

  it('should_name_the_source_when_the_hash_of_the_plugin_does_not_match', () => {
    const reason = stateReason('LOAD_FAILED', {
      reason: {
        code: 'HASH_MISMATCH',
        payload: {
          path: 'p/a.wasm',
          actual_sha256: 'aa',
          recorded_sha256: 'bb',
          source_type: 'github',
        },
      },
    });

    expect(getTranslatedStateReason(reason, tEn)).toBe(
      'The plugin failed to load: the plugin file changed on disk since it was installed from github; reinstall the plugin from its source before enabling it',
    );
    expect(getStateReasonDetails(reason)).toBeUndefined();
  });

  it('should_translate_the_versions_of_an_incompatible_reason', () => {
    const reason = stateReason('INCOMPATIBLE_API_VERSION', {
      required: '^2.0.0',
      host: '1.4.0',
    });

    expect(getTranslatedStateReason(reason, tEn)).toBe(
      'The plugin requires API ^2.0.0, this launcher provides 1.4.0',
    );
    expect(getStateReasonDetails(reason)).toBeUndefined();
  });
});
