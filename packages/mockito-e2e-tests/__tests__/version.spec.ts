/**
 * Version and API Tests
 *
 * Tests for binding version and API availability.
 *
 * @see bindings/node_binding/binding.d.ts - exported API
 */
import {describe, expect, it} from '@rstest/core';
import {version, MocksManager, MocksController, HttpMethod, Transport} from '@mockito/binding';

describe('Binding API', () => {
    describe('version', () => {
        /**
         * Tests that version function is exported.
         * @see bindings/node_binding/binding.d.ts - version export
         */
        it('should export version function', () => {
            expect(typeof version).toBe('function');
        });

        /**
         * Tests that version returns a string.
         * @see bindings/node_binding/binding.d.ts - version(): string
         */
        it('should return version string', () => {
            const v = version();
            expect(typeof v).toBe('string');
            expect(v.length).toBeGreaterThan(0);
        });

        /**
         * Tests version format (semver-like).
         * @see bindings/node_binding/package.json - version field
         */
        it('should return semver-like version', () => {
            const v = version();
            // Should contain at least one dot for semver
            expect(v).toMatch(/^\d+\.\d+/);
        });
    });

    describe('exports', () => {
        /**
         * Tests MocksManager class export.
         * @see bindings/node_binding/binding.d.ts - MocksManager export
         */
        it('should export MocksManager class', () => {
            expect(MocksManager).toBeDefined();
            expect(typeof MocksManager).toBe('function');
        });

        /**
         * Tests MocksController class export.
         * @see bindings/node_binding/binding.d.ts - MocksController export
         */
        it('should export MocksController class', () => {
            expect(MocksController).toBeDefined();
            expect(typeof MocksController).toBe('function');
        });

        /**
         * Tests HttpMethod enum export.
         * @see bindings/node_binding/binding.d.ts - HttpMethod enum
         */
        it('should export HttpMethod enum', () => {
            expect(HttpMethod).toBeDefined();
            expect(HttpMethod.Get).toBe(0);
            expect(HttpMethod.Post).toBe(1);
            expect(HttpMethod.Put).toBe(2);
            expect(HttpMethod.Patch).toBe(3);
            expect(HttpMethod.Delete).toBe(4);
            expect(HttpMethod.Head).toBe(5);
            expect(HttpMethod.Options).toBe(6);
        });

        /**
         * Tests Transport enum export.
         * @see bindings/node_binding/binding.d.ts - Transport enum
         */
        it('should export Transport enum', () => {
            expect(Transport).toBeDefined();
            expect(Transport.Http).toBe(0);
            expect(Transport.WebSocket).toBe(1);
        });
    });

    describe('enum values', () => {
        /**
         * Tests all HTTP method enum values.
         * @see example/mocker/consts.ts - HTTP_METHODS array
         */
        it('should have all HTTP methods', () => {
            const methods = [
                HttpMethod.Get,
                HttpMethod.Post,
                HttpMethod.Put,
                HttpMethod.Patch,
                HttpMethod.Delete,
                HttpMethod.Head,
                HttpMethod.Options,
            ];

            expect(methods).toHaveLength(7);
            methods.forEach((method, index) => {
                expect(method).toBe(index);
            });
        });

        /**
         * Tests all transport types.
         * @see example/mocker/consts.ts - Transport types
         */
        it('should have all transport types', () => {
            expect(Transport.Http).toBe(0);
            expect(Transport.WebSocket).toBe(1);
        });
    });
});
