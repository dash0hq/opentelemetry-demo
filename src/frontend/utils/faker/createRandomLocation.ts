// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

import { faker } from '@faker-js/faker';
import { persistedCall } from './persistedCall';

type LocationSeed = {
    countryCode: string;
    continentCode: string;
    locality: string
}

export const createRandomLocation = persistedCall('random_location', (seed?: LocationSeed) => {
  const continent = faker.location.continent();
  // @ts-expect-error we simply don't care.
  const continentCode = continentCodes[continent] ?? 'EU';
  const country = faker.location.countryCode();
  const locality = faker.location.city();

  return {
    'geo.continent.code': seed?.continentCode || continentCode,
    'geo.country.iso_code': seed?.countryCode || country,
    'geo.locality.name': seed?.locality || locality,
  };
});

const continentCodes = {
  Africa: 'AF',
  Antarctica: 'AN',
  Asia: 'AS',
  Europe: 'EU',
  'North America': 'NA',
  Oceania: 'OC',
  'South America': 'SA',
};
