import { describe, expect, it } from 'vitest';

import { enDictionary, ruDictionary } from '..';

// Parents whose children are values of a backend enum: the segment keeps the
// value's case, so the key can be built from the value. A new map goes here.
const ENUM_VALUE_PARENTS = [
  'errors',
  'events.stage',
  'instance.broken.reasons',
  'plugin.commandApproval.onceOnly',
  'plugin.hostRisk',
  'plugin.stateReason',
  'pluginSettings.ignoredPath',
  'plugins.updateDialog.capabilityKind',
  'warnings',
];

const CAMEL_CASE = /^[a-z][a-zA-Z0-9]*$/;
const ENUM_VALUE = /^[A-Za-z][A-Za-z0-9]*(?:_[A-Za-z0-9]+)*$/;
const ERROR_CODE = /^[A-Z][A-Z0-9]*(?:_[A-Z0-9]+)*$/;

const keyPaths = (value: unknown, path: string[] = []): string[][] => {
  const entries =
    value !== null && typeof value === 'object' ? Object.entries(value) : [];

  return entries.length > 0
    ? entries.flatMap(([k, v]) => keyPaths(v, [...path, k]))
    : [path];
};

const isAllowed = (path: string[], index: number): boolean => {
  const segment = path[index] ?? '';

  if (CAMEL_CASE.test(segment)) {
    return true;
  }

  const parent = path.slice(0, index).join('.');

  if (ENUM_VALUE_PARENTS.includes(parent) && ENUM_VALUE.test(segment)) {
    return true;
  }

  // Under errors.<domain>: error codes, their reasons and the enum values that
  // name an argument (errors.<domain>.field.<VALUE>).
  return path[0] === 'errors' && index >= 2 && ERROR_CODE.test(segment);
};

describe('i18n key format', () => {
  it.each([
    ['en', enDictionary],
    ['ru', ruDictionary],
  ])('uses camelCase segments in %s', (_name, dictionary) => {
    const offenders = keyPaths(dictionary)
      .filter((path) => path.some((_, i) => !isAllowed(path, i)))
      .map((path) => path.join('.'));

    expect(offenders).toEqual([]);
  });
});
