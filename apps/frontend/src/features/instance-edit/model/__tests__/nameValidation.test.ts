import { describe, expect, it } from 'vitest';

import { validateInstanceName } from '../nameValidation';

describe('validateInstanceName', () => {
  it('accepts a name and trims it', () => {
    expect(validateInstanceName('  My instance  ')).toEqual({
      valid: true,
      name: 'My instance',
    });
  });

  it('rejects an empty or whitespace-only name', () => {
    expect(validateInstanceName('')).toEqual({
      valid: false,
      errorKey: 'generalSettingsError.name.minLength',
    });
    expect(validateInstanceName('   ')).toEqual({
      valid: false,
      errorKey: 'generalSettingsError.name.minLength',
    });
  });

  it('rejects a name longer than 64 characters', () => {
    expect(validateInstanceName('a'.repeat(65))).toEqual({
      valid: false,
      errorKey: 'generalSettingsError.name.maxLength',
    });
  });

  it('accepts a name of exactly 64 characters', () => {
    expect(validateInstanceName('a'.repeat(64))).toEqual({
      valid: true,
      name: 'a'.repeat(64),
    });
  });
});
