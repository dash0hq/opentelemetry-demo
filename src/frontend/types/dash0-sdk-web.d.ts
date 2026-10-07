// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// TypeScript's "node" module resolution (which Next.js 12 enforces) ignores package.json "exports",
// so subpath entrypoints need to be mapped to their type declarations by hand. Webpack resolves the
// actual module through "exports".
declare module '@dash0/sdk-web/session-recording' {
  export * from '@dash0/sdk-web/dist/types/entrypoint/session-recording';
}
