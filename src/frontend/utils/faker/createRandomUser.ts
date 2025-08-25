// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

import { faker } from '@faker-js/faker';
import { persistedCall } from './persistedCall';

export const createRandomUser = persistedCall('random_user', (seedEmail?: string) => {
    let firstName: string;
    if (seedEmail) {
        const name = seedEmail.split("@")[0];
        firstName = capitalizeNames(name.replace("_", " "))
    } else {
        firstName = faker.person.firstName();
    }
  const lastName = faker.person.lastName();

  return {
    userId: faker.string.uuid(),
    username: faker.internet.username(), // before version 9.1.0, use userName()
    fullName: `${firstName} ${lastName}`,
    email: seedEmail || faker.internet.email({ firstName: firstName.toLowerCase(), lastName: lastName.toLowerCase() }),
    userAgent: faker.internet.userAgent(),
  };
});

function capitalizeNames(name: string) {
    return name.split(" ")
        .map(part => part.charAt(0).toUpperCase() + part.slice(1))
        .join(" ")
}
