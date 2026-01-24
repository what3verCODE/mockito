import {defineConfig} from '@rstest/core';

export default defineConfig({
    include: ['__tests__/**/*.spec.ts'],
    testTimeout: 10000,
});
