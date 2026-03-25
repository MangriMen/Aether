import { describe, it, expect, vi } from 'vitest';

import * as instances from '@/entities/instances';

import {
  decodeContentSearchParams,
  encodeContentSearchParams,
} from '../contentSearchParams';

vi.spyOn(instances, 'isContentType').mockImplementation((val) =>
  ['mods', 'packs'].includes(val as string),
);

describe('Content Search Params Mapping', () => {
  describe('decodeContentSearchParams', () => {
    it('should correctly decode valid search params', () => {
      const raw = {
        instance: '123',
        page: '2',
        pageSize: '20',
        query: 'test-query',
        contentType: 'mods',
        gameVersions: ['1.20.1', '1.19.2'],
      };

      const result = decodeContentSearchParams(raw);

      expect(result).toEqual({
        instanceId: '123',
        page: 2,
        pageSize: 20,
        query: 'test-query',
        provider: undefined,
        contentType: 'mods',
        gameVersions: ['1.20.1', '1.19.2'],
        loaders: undefined,
      });
    });

    it('should return undefined for invalid numbers or types', () => {
      const raw = {
        page: 'not-a-number',
        contentType: 'invalid-type',
      };

      const result = decodeContentSearchParams(raw);

      expect(result.page).toBeUndefined();
      expect(result.contentType).toBeUndefined();
    });

    it('should handle single string in array fields', () => {
      const raw = { gameVersions: '1.20.1' };
      const result = decodeContentSearchParams(raw);

      expect(result.gameVersions).toEqual(['1.20.1']);
    });
  });

  describe('encodeContentSearchParams', () => {
    it('should encode state to flat object and filter undefined', () => {
      const filters = {
        instanceId: '123',
        page: 1,
        query: '',
        gameVersions: [],
      };

      const result = encodeContentSearchParams(filters);

      expect(result).toEqual({
        instance: '123',
        page: '1',
      });
      expect(result).not.toHaveProperty('query');
      expect(result).not.toHaveProperty('gameVersions');
    });

    it('should correctly map instanceId to instance key', () => {
      const result = encodeContentSearchParams({ instanceId: 'abc' });
      expect(result.instance).toBe('abc');
      expect(result.instanceId).toBeUndefined();
    });

    it('should keep arrays as they are (letting router handle them)', () => {
      const versions = ['1.20.1', '1.18.2'];
      const result = encodeContentSearchParams({ gameVersions: versions });
      expect(result.gameVersions).toEqual(versions);
    });
  });

  describe('Symmetry (Round-trip)', () => {
    it('should maintain data integrity through decode and encode', () => {
      const originalParams = {
        instance: 'my-id',
        page: '5',
        query: 'search',
      };

      const decoded = decodeContentSearchParams(originalParams);
      const encoded = encodeContentSearchParams(decoded);

      expect(encoded).toEqual(originalParams);
    });
  });
});
