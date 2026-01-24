/**
 * Edge Cases and Error Handling Tests
 *
 * These tests verify error handling and boundary conditions.
 * Some behaviors may differ between JS and Rust implementations.
 *
 * @see example/mocker/__specs__/loader.spec.ts for file loading tests
 * @see example/mocker/__specs__/validation.spec.ts for validation edge cases
 */
import path from 'node:path';
import {describe, expect, it} from '@rstest/core';
import {MocksManager, MocksController} from '@mockito/binding';

const FIXTURES_PATH = path.join(import.meta.dirname, '..', '__fixtures__');
const COLLECTIONS_PATH = path.join(FIXTURES_PATH, 'collections.yaml');
const ROUTES_PATH = path.join(FIXTURES_PATH, 'routes', '*.yaml');

describe('Edge Cases', () => {
    describe('file loading', () => {
        /**
         * Tests error when collections file doesn't exist.
         * @see example/mocker/__specs__/mocks.spec.ts - "should throw if no collection found"
         * @see example/mocker/__specs__/loader.spec.ts - file loading tests
         */
        it('should throw error for non-existent collections file', () => {
            expect(
                () => new MocksManager('/non/existent/collections.yaml', ROUTES_PATH)
            ).toThrow();
        });

        /**
         * Tests behavior for non-existent routes glob pattern.
         * Note: Rust implementation may not throw for empty glob results.
         * @see example/mocker/__specs__/loader.spec.ts - "should load file based on data from ctor"
         */
        it('should not throw error for non-existent routes directory (empty result)', () => {
            // Rust implementation returns empty routes array for non-existent glob pattern
            // This is by design - glob patterns can match zero files
            const manager = new MocksManager(COLLECTIONS_PATH, '/non/existent/routes/*.yaml');
            
            // Resolving collection should throw because routes don't exist
            expect(() => manager.resolveCollection('base')).toThrow();
        });

        /**
         * Tests error for invalid collections path in MocksController.
         * @see example/mocker/__specs__/mocks.spec.ts - "should throw if no collection found"
         */
        it('should throw error for invalid collections path', () => {
            expect(
                () => new MocksController('/invalid/path.yaml', ROUTES_PATH)
            ).toThrow();
        });
    });

    describe('collection resolution errors', () => {
        /**
         * Tests MocksManager error for non-existent collection.
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if no collection found"
         */
        it('MocksManager should throw for non-existent collection', () => {
            const manager = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);

            expect(() => manager.resolveCollection('does-not-exist')).toThrow();
        });

        /**
         * Tests MocksController error for non-existent collection.
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if no collection found"
         */
        it('MocksController should throw for non-existent collection', () => {
            const controller = new MocksController(COLLECTIONS_PATH, ROUTES_PATH);

            expect(() => controller.useCollection('does-not-exist')).toThrow();
        });
    });

    describe('multiple instances', () => {
        /**
         * Tests that multiple MocksManager instances can coexist.
         * @see example/mocker/mocks.ts - Mocks class is stateless after load()
         */
        it('should allow creating multiple MocksManager instances', () => {
            const manager1 = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);
            const manager2 = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);

            const routes1 = manager1.resolveCollection('base');
            const routes2 = manager2.resolveCollection('base');

            expect(routes1).toHaveLength(routes2.length);
        });

        /**
         * Tests that multiple MocksController instances can coexist.
         * @see example/mocker/controller.ts - Controller maintains its own state
         */
        it('should allow creating multiple MocksController instances', () => {
            const controller1 = new MocksController(COLLECTIONS_PATH, ROUTES_PATH);
            const controller2 = new MocksController(COLLECTIONS_PATH, ROUTES_PATH);

            controller1.useCollection('base');
            controller2.useCollection('extended');

            expect(controller1.currentCollection).toBe('base');
            expect(controller2.currentCollection).toBe('extended');
        });

        /**
         * Tests that controllers are independent of each other.
         * @see example/mocker/controller.ts - Controller has own selectedCollection state
         */
        it('controllers should be independent', () => {
            const controller1 = new MocksController(COLLECTIONS_PATH, ROUTES_PATH);
            const controller2 = new MocksController(COLLECTIONS_PATH, ROUTES_PATH);

            controller1.useCollection('base');
            controller2.useCollection('extended');

            // Changing controller2 should not affect controller1
            controller2.useCollection('standalone');

            expect(controller1.currentCollection).toBe('base');
            expect(controller2.currentCollection).toBe('standalone');
        });
    });

    describe('route data integrity', () => {
        /**
         * Tests that response body structure is preserved through YAML parsing.
         * @see example/mocker/__specs__/loader.spec.ts - YAML content preservation test
         */
        it('should preserve response body structure', () => {
            const manager = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);
            const routes = manager.resolveCollection('base');

            const usersRoute = routes.find(r => r.route.id === 'users-api');
            const body = usersRoute?.variant.body as {
                data: Array<{id: number; name: string; email: string}>;
                total: number;
            };

            expect(body).toBeDefined();
            expect(body.data).toBeInstanceOf(Array);
            expect(body.data[0]).toHaveProperty('id');
            expect(body.data[0]).toHaveProperty('name');
            expect(body.data[0]).toHaveProperty('email');
            expect(typeof body.total).toBe('number');
        });

        /**
         * Tests that header values are preserved.
         * @see example/mocker/validation.ts - headers field in variant
         */
        it('should preserve header values', () => {
            const manager = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);
            const routes = manager.resolveCollection('base');

            const usersRoute = routes.find(r => r.route.id === 'users-api');
            const headers = usersRoute?.variant.headers;

            expect(headers).toBeDefined();
            expect(headers?.['Content-Type']).toBe('application/json');
        });

        /**
         * Tests that status codes are preserved.
         * @see example/mocker/validation.ts - status field in variant (VARIANT_SCHEMA)
         */
        it('should preserve status codes', () => {
            const manager = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);

            // Success response
            const baseRoutes = manager.resolveCollection('base');
            const successRoute = baseRoutes.find(r => r.route.id === 'users-api');
            expect(successRoute?.variant.status).toBe(200);

            // Error response
            const extendedRoutes = manager.resolveCollection('extended');
            const errorRoute = extendedRoutes.find(r => r.route.id === 'users-api');
            expect(errorRoute?.variant.status).toBe(404);
        });
    });

    describe('preset matching conditions', () => {
        /**
         * Tests expression-based header conditions parsing.
         * @see example/mocker/validation.ts - STRING_OR_NON_EMPTY_STRING_RECORD for headers
         * @see example/mocker/consts.ts - EXPRESSION_REGEX for ${} syntax
         */
        it('should parse expression-based header conditions', () => {
            const manager = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);
            const routes = manager.resolveCollection('deeply-nested');

            const paymentsRoute = routes.find(r => r.route.id === 'payments-api');
            expect(paymentsRoute?.preset.headers).toBeDefined();
        });

        /**
         * Tests object-based payload conditions parsing.
         * @see example/mocker/validation.ts - STRING_OR_NON_EMPTY_OBJECT for payload
         */
        it('should parse object-based payload conditions', () => {
            const manager = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);
            const routes = manager.resolveCollection('extended');

            const ordersRoute = routes.find(r => r.route.id === 'orders-api');
            expect(ordersRoute?.preset.payload).toBeDefined();
        });
    });

    describe('URL patterns', () => {
        /**
         * Tests API URL path preservation.
         * @see example/mocker/validation.ts - url field is required string
         */
        it('should correctly parse API URL paths', () => {
            const manager = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);
            const routes = manager.resolveCollection('base');

            const usersRoute = routes.find(r => r.route.id === 'users-api');
            expect(usersRoute?.route.url).toBe('/api/users');

            const productsRoute = routes.find(r => r.route.id === 'products-api');
            expect(productsRoute?.route.url).toBe('/api/products');
        });

        /**
         * Tests WebSocket URL path preservation.
         * @see example/mocker/validation.ts - url field for WS routes
         */
        it('should correctly parse WebSocket URL paths', () => {
            const manager = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);
            const routes = manager.resolveCollection('with-websocket');

            const wsRoute = routes.find(r => r.route.id === 'ws-notifications');
            expect(wsRoute?.route.url).toBe('/ws/notifications');
        });
    });

    describe('default collection', () => {
        /**
         * Tests that default collection is applied on construction.
         * @see example/mocker/__specs__/controller.spec.ts - "should mock selected collection"
         */
        it('should apply default collection on construction', () => {
            const controller = new MocksController(COLLECTIONS_PATH, ROUTES_PATH, 'base');

            expect(controller.currentCollection).toBe('base');
            expect(controller.getActiveRoutes()).toHaveLength(2);
        });

        /**
         * Tests switching from default collection to another.
         * @see example/mocker/controller.ts - useCollection can be called after constructor
         */
        it('should allow switching from default collection', () => {
            const controller = new MocksController(COLLECTIONS_PATH, ROUTES_PATH, 'base');

            controller.useCollection('extended');

            expect(controller.currentCollection).toBe('extended');
        });
    });
});
