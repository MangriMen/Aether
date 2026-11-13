import { describe, expect, it } from 'vitest';

import type { TFunction } from '@/shared/i18n';

import { getBrokenReasonText } from '../brokenInstance';

// The key and its arguments, as the translator got them.
const t = ((key: string, args?: unknown) =>
  `${key} ${JSON.stringify(args)}`) as unknown as TFunction;

describe('getBrokenReasonText', () => {
  it('should_name_the_key_by_the_code_and_pass_the_payload_when_the_reason_has_one', () => {
    expect(
      getBrokenReasonText(
        { code: 'INVALID_LOADER', payload: { value: 'x' } },
        t,
      ),
    ).toBe('instance.broken.reasons.INVALID_LOADER {"value":"x"}');
  });

  it('should_pass_no_arguments_when_the_reason_has_no_payload', () => {
    expect(getBrokenReasonText({ code: 'NOT_MIGRATED' }, t)).toBe(
      'instance.broken.reasons.NOT_MIGRATED undefined',
    );
  });
});

describe('getBrokenReasonText with a cause', () => {
  it('should_name_the_category_of_the_cause_when_the_file_is_unreadable', () => {
    expect(
      getBrokenReasonText(
        {
          code: 'UNREADABLE_FILE',
          payload: { cause: { type: 'io', kind: 'notFound' } },
        },
        t,
      ),
    ).toBe(
      'instance.broken.reasons.UNREADABLE_FILE undefined: errors.cause.io.notFound undefined',
    );
  });

  it('should_name_no_cause_when_it_is_unknown', () => {
    expect(
      getBrokenReasonText(
        {
          code: 'UNREADABLE_FILE',
          payload: { cause: { type: 'unknown', details: 'raw' } },
        },
        t,
      ),
    ).toBe('instance.broken.reasons.UNREADABLE_FILE undefined');
  });
});
