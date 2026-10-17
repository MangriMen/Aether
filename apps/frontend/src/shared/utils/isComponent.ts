import type { Component } from 'solid-js';

export const isComponent = (variable: unknown): variable is Component =>
  typeof variable === 'function';
