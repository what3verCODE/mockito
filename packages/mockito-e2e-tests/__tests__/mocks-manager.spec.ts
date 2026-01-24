/**
 * E2E Tests for MocksManager (Rust implementation)
 *
 * These tests document expected behavior based on the original JS implementation.
 * Some tests may fail if features are not yet implemented in Rust - this is expected
 * and helps track implementation progress.
 *
 * MocksManager is stateless and resolves collections on-demand.
 *
 * @see example/mocker/__specs__/mocks.spec.ts for Mocks loader tests
 * @see example/mocker/mocks.ts for Mocks class
 * @see example/mocker/controller.ts for collection resolution logic
 */
import path from 'node:path';
import {describe, expect, it, beforeAll} from '@rstest/core';
import {MocksManager, Transport, HttpMethod} from '@mockito/binding';

const FIXTURES_PATH = path.join(import.meta.dirname, '..', '__fixtures__');
const COLLECTIONS_PATH = path.join(FIXTURES_PATH, 'collections.yaml');
const ROUTES_PATH = path.join(FIXTURES_PATH, 'routes', '*.yaml');

describe('MocksManager', () => {
    let manager: MocksManager;

    beforeAll(() => {
        manager = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);
    });

    describe('resolveCollection', () => {
        /**
         * Tests basic collection resolution without inheritance.
         * @see example/mocker/__specs__/controller.spec.ts - "should mock selected collection"
         * @see example/mocker/controller.ts - useCollection method
         */
        it('should resolve base collection without inheritance', () => {
            const routes = manager.resolveCollection('base');

            expect(routes).toHaveLength(2);

            const routeIds = routes.map(r => r.route.id);
            expect(routeIds).toContain('users-api');
            expect(routeIds).toContain('products-api');
        });

        /**
         * Tests collection inheritance via 'from' field.
         * @see example/mocker/__specs__/controller.spec.ts - "should mock selected collection with nesting"
         * @see example/mocker/controller.ts - fillNestedRoutes method
         */
        it('should resolve collection with inheritance (from)', () => {
            const routes = manager.resolveCollection('extended');

            // extended inherits from base (2 routes) + adds 2 more
            // users-api:error:not-found overrides users-api:success:default from base
            expect(routes.length).toBeGreaterThanOrEqual(3);

            const usersRoute = routes.find(r => r.route.id === 'users-api');
            expect(usersRoute).toBeDefined();
            expect(usersRoute?.preset.id).toBe('error');
            expect(usersRoute?.variant.id).toBe('not-found');

            const ordersRoute = routes.find(r => r.route.id === 'orders-api');
            expect(ordersRoute).toBeDefined();
            expect(ordersRoute?.preset.id).toBe('success');
        });

        /**
         * Tests that child route overrides parent route with same route id.
         * @see example/mocker/controller.ts - fillNestedRoutes: "if (map[identifier] === undefined)"
         */
        it('should override parent route with child route (same route id)', () => {
            const routes = manager.resolveCollection('override-parent');

            const usersRoute = routes.find(r => r.route.id === 'users-api');
            expect(usersRoute).toBeDefined();
            // Child overrides parent: success:empty-list instead of success:default
            expect(usersRoute?.preset.id).toBe('success');
            expect(usersRoute?.variant.id).toBe('empty-list');
        });

        /**
         * Tests multi-level inheritance chain (grandparent → parent → child).
         * @see example/mocker/controller.ts - recursive fillNestedRoutes call
         */
        it('should resolve deeply nested collections', () => {
            const routes = manager.resolveCollection('deeply-nested');

            // deeply-nested inherits from extended, which inherits from base
            expect(routes.length).toBeGreaterThanOrEqual(4);

            const paymentsRoute = routes.find(r => r.route.id === 'payments-api');
            expect(paymentsRoute).toBeDefined();
        });

        /**
         * Tests collection without 'from' field (no inheritance).
         * @see example/mocker/controller.ts - fillNestedRoutes returns early if !collection.from
         */
        it('should resolve standalone collection without inheritance', () => {
            const routes = manager.resolveCollection('standalone');

            // Standalone has two routes with same route id but different presets
            // Rust implementation deduplicates by route id + preset id combination
            // Last one wins when same route id + preset id is used
            expect(routes.length).toBeGreaterThanOrEqual(1);

            const usersRoute = routes.find(r => r.route.id === 'users-api');
            expect(usersRoute).toBeDefined();
        });

        /**
         * Tests WebSocket route in collection.
         * @see example/mocker/__specs__/controller.spec.ts - "should pass parser url for websocket route"
         * @see example/mocker/consts.ts - WEBSOCKET_METHOD = 'WS'
         */
        it('should resolve collection with WebSocket routes', () => {
            const routes = manager.resolveCollection('with-websocket');

            expect(routes).toHaveLength(1);

            const wsRoute = routes[0];
            expect(wsRoute?.route.id).toBe('ws-notifications');
            expect(wsRoute?.route.transport).toBe(Transport.WebSocket);
        });

        /**
         * Tests error for non-existent collection.
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if no collection found"
         */
        it('should throw error for non-existent collection', () => {
            expect(() => manager.resolveCollection('non-existent')).toThrow();
        });
    });

    describe('ActiveRoute structure', () => {
        /**
         * Tests Route structure matches expected shape.
         * @see example/mocker/validation.ts - Route type (ROUTE_SCHEMA)
         */
        it('should have correct route structure', () => {
            const routes = manager.resolveCollection('base');
            const usersRoute = routes.find(r => r.route.id === 'users-api');

            expect(usersRoute).toBeDefined();
            expect(usersRoute?.route).toMatchObject({
                id: 'users-api',
                url: '/api/users',
                transport: Transport.Http,
                method: HttpMethod.Get,
            });
        });

        /**
         * Tests Preset structure matches expected shape.
         * @see example/mocker/validation.ts - Preset type (PRESET_SCHEMA)
         */
        it('should have correct preset structure', () => {
            const routes = manager.resolveCollection('base');
            const usersRoute = routes.find(r => r.route.id === 'users-api');

            expect(usersRoute?.preset).toBeDefined();
            expect(usersRoute?.preset.id).toBe('success');
            expect(usersRoute?.preset.variants).toBeDefined();
        });

        /**
         * Tests Variant structure with all fields.
         * @see example/mocker/validation.ts - Variant type (VARIANT_SCHEMA)
         */
        it('should have correct variant structure', () => {
            const routes = manager.resolveCollection('base');
            const usersRoute = routes.find(r => r.route.id === 'users-api');

            expect(usersRoute?.variant).toBeDefined();
            expect(usersRoute?.variant.id).toBe('default');
            expect(usersRoute?.variant.status).toBe(200);
            expect(usersRoute?.variant.headers).toBeDefined();
            expect(usersRoute?.variant.body).toBeDefined();
        });

        /**
         * Tests body content preservation through parsing.
         * @see example/mocker/__specs__/loader.spec.ts - YAML parsing tests
         */
        it('should have correct variant body content', () => {
            const routes = manager.resolveCollection('base');
            const usersRoute = routes.find(r => r.route.id === 'users-api');

            const body = usersRoute?.variant.body as {data: Array<{id: number; name: string}>; total: number};
            expect(body.data).toHaveLength(2);
            expect(body.data[0]).toMatchObject({id: 1, name: 'John Doe'});
            expect(body.total).toBe(2);
        });
    });

    describe('HTTP methods', () => {
        /**
         * Tests GET method parsing from YAML.
         * @see example/mocker/consts.ts - HTTP_METHODS = ['GET', 'POST', ...]
         * @see example/mocker/validation.ts - method field oneOf validation
         */
        it('should correctly parse GET method', () => {
            const routes = manager.resolveCollection('base');
            const usersRoute = routes.find(r => r.route.id === 'users-api');

            expect(usersRoute?.route.method).toBe(HttpMethod.Get);
        });

        /**
         * Tests POST method parsing from YAML.
         * @see example/mocker/consts.ts - HTTP_METHODS constant
         */
        it('should correctly parse POST method', () => {
            const routes = manager.resolveCollection('extended');
            const ordersRoute = routes.find(r => r.route.id === 'orders-api');

            expect(ordersRoute?.route.method).toBe(HttpMethod.Post);
        });
    });

    describe('preset matching conditions', () => {
        /**
         * Tests preset headers condition parsing.
         * @see example/mocker/validation.ts - PRESET_SCHEMA headers field
         * @see example/mocker/router/playwright-router.ts - header matching logic
         */
        it('should parse preset with headers condition', () => {
            const routes = manager.resolveCollection('deeply-nested');
            const paymentsRoute = routes.find(r => r.route.id === 'payments-api');

            expect(paymentsRoute?.preset.headers).toBeDefined();
        });

        /**
         * Tests preset payload condition parsing.
         * @see example/mocker/validation.ts - PRESET_SCHEMA payload field
         * @see example/mocker/router/playwright-router.ts - payload matching logic
         */
        it('should parse preset with payload condition', () => {
            const routes = manager.resolveCollection('extended');
            const ordersRoute = routes.find(r => r.route.id === 'orders-api');

            expect(ordersRoute?.preset.payload).toBeDefined();
        });
    });

    /**
     * Features that may not be implemented yet.
     * These tests are marked as .todo and will be enabled when features are ready.
     *
     * @see example/mocker/__specs__/validation.spec.ts for validation tests
     * @see example/mocker/lib/ for utility functions
     */
    describe('TODO: features to implement', () => {
        /**
         * @see example/mocker/validation.ts - STRING_OR_NON_EMPTY_STRING_RECORD for query
         * @see example/mocker/router/playwright-router.ts - query matching with expressions
         */
        it.todo('should support query parameter matching expressions');

        /**
         * @see example/mocker/lib/object-intersects.ts - JMESPath support
         * @see example/mocker/router/playwright-router.ts - payload matching
         */
        it.todo('should support JMESPath expressions in payload matching');

        /**
         * @see example/mocker/lib/url-matches.ts - fillUrlWithParameters
         * @see example/mocker/validation.ts - params field in preset
         */
        it.todo('should support URL path parameters (e.g., /users/:id)');

        /**
         * @see example/mocker/consts.ts - ALL_HTTP_METHODS = '*'
         * @see example/mocker/validation.ts - method oneOf includes '*'
         */
        it.todo('should support wildcard HTTP method (*)');
    });
});
