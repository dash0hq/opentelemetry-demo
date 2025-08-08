// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

const prefix = 'persisted_call';

export function persistedCall<P, T>(key: string, fn: (args: P) => T): (args: P) => T {
  return (args: P) => {
    const persistedResult = sessionStorage.getItem(`${prefix}_${key}`);

    if (persistedResult) {
      try {
        return JSON.parse(persistedResult) as T;
      } catch (err) {
        // ignoring
      }
    }

    const result = fn(args);
    sessionStorage.setItem(`${prefix}_${key}`, JSON.stringify(result));
    return result;
  };
}
