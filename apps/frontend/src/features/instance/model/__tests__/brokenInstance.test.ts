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
