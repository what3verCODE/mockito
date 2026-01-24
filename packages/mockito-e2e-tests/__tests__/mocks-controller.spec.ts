/**
 * E2E Tests for MocksController (Rust implementation)
 *
 * These tests document expected behavior based on the original JS implementation.
 * Some tests may fail if features are not yet implemented in Rust - this is expected
 * and helps track implementation progress.
 *
 * MocksController manages stateful collection switching and provides active routes.
 *
 * @see example/mocker/__specs__/controller.spec.ts for original tests
 * @see example/mocker/controller.ts for original implementation
 */
import path from 'node:path';
import {describe, expect, it, beforeEach} from '@rstest/core';
import {MocksController, Transport, HttpMethod} from '@mockito/binding';

const FIXTURES_PATH = path.join(import.meta.dirname, '..', '__fixtures__');
const COLLECTIONS_PATH = path.join(FIXTURES_PATH, 'collections.yaml');
const ROUTES_PATH = path.join(FIXTURES_PATH, 'routes', '*.yaml');

describe('MocksController', () => {
    let controller: MocksController;

    beforeEach(() => {
        controller = new MocksController(COLLECTIONS_PATH, ROUTES_PATH);
    });

    describe('constructor', () => {
        /**
         * Tests controller initialization without default collection.
         * In JS version, Controller is created with pre-loaded collections/routes.
         * @see example/mocker/__specs__/controller.spec.ts - createController()
         */
        it('should create controller without default collection', () => {
            const ctrl = new MocksController(COLLECTIONS_PATH, ROUTES_PATH);
            expect(ctrl.currentCollection).toBeNull();
        });

        /**
         * Tests controller initialization with default collection applied.
         * @see example/mocker/__specs__/controller.spec.ts - "should mock selected collection"
         */
        it('should create controller with default collection', () => {
            const ctrl = new MocksController(COLLECTIONS_PATH, ROUTES_PATH, 'base');
            expect(ctrl.currentCollection).toBe('base');
        });

        /**
         * Tests error when non-existent default collection is provided.
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if no collection found"
         */
        it('should throw error for non-existent default collection', () => {
            expect(
                () => new MocksController(COLLECTIONS_PATH, ROUTES_PATH, 'non-existent')
            ).toThrow();
        });
    });

    describe('useCollection', () => {
        /**
         * Tests switching to a specific collection.
         * @see example/mocker/__specs__/controller.spec.ts - "should mock selected collection"
         */
        it('should switch to specified collection', () => {
            controller.useCollection('base');

            expect(controller.currentCollection).toBe('base');
        });

        /**
         * Tests switching between different collections.
         * @see example/mocker/__specs__/controller.spec.ts - multiple useCollection calls
         */
        it('should switch between collections', () => {
            controller.useCollection('base');
            expect(controller.currentCollection).toBe('base');

            controller.useCollection('extended');
            expect(controller.currentCollection).toBe('extended');
        });

        /**
         * Tests error handling for non-existent collection.
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if no collection found"
         */
        it('should throw error for non-existent collection', () => {
            expect(() => controller.useCollection('non-existent')).toThrow();
        });
    });

    describe('getActiveRoutes', () => {
        /**
         * Tests that no routes returned when no collection is selected.
         * @see example/mocker/__specs__/controller.spec.ts - "should restore mock to nothing if no collection selected"
         */
        it('should return empty array when no collection selected', () => {
            const routes = controller.getActiveRoutes();
            expect(routes).toHaveLength(0);
        });

        /**
         * Tests that active routes are returned after selecting collection.
         * @see example/mocker/__specs__/controller.spec.ts - "should mock selected collection"
         */
        it('should return active routes after selecting collection', () => {
            controller.useCollection('base');

            const routes = controller.getActiveRoutes();
            expect(routes).toHaveLength(2);
        });

        /**
         * Tests collection inheritance (from field) resolution.
         * @see example/mocker/__specs__/controller.spec.ts - "should mock selected collection with nesting"
         */
        it('should return routes with inheritance resolved', () => {
            controller.useCollection('extended');

            const routes = controller.getActiveRoutes();
            expect(routes.length).toBeGreaterThanOrEqual(3);

            const routeIds = routes.map(r => r.route.id);
            expect(routeIds).toContain('users-api');
            expect(routeIds).toContain('products-api');
            expect(routeIds).toContain('orders-api');
        });

        /**
         * Tests WebSocket route handling.
         * @see example/mocker/__specs__/controller.spec.ts - "should pass parser url for websocket route"
         */
        it('should return WebSocket routes', () => {
            controller.useCollection('with-websocket');

            const routes = controller.getActiveRoutes();
            expect(routes).toHaveLength(1);

            expect(routes[0]?.route.transport).toBe(Transport.WebSocket);
        });
    });

    describe('currentCollection', () => {
        /**
         * Tests initial state before any collection is selected.
         * @see example/mocker/controller.ts - selectedCollection initial value
         */
        it('should be null initially', () => {
            expect(controller.currentCollection).toBeNull();
        });

        /**
         * Tests currentCollection getter after useCollection.
         * @see example/mocker/controller.ts - useCollection method
         */
        it('should return current collection id after useCollection', () => {
            controller.useCollection('standalone');

            expect(controller.currentCollection).toBe('standalone');
        });

        /**
         * Tests that currentCollection updates when switching.
         * @see example/mocker/controller.ts - useCollection updates selectedCollection
         */
        it('should update when switching collections', () => {
            controller.useCollection('base');
            controller.useCollection('minimal');

            expect(controller.currentCollection).toBe('minimal');
        });
    });

    describe('route structure validation', () => {
        /**
         * Tests Route structure from Rust binding matches expected shape.
         * @see example/mocker/validation.ts - Route type definition
         */
        it('should have route with all required fields', () => {
            controller.useCollection('base');
            const routes = controller.getActiveRoutes();
            const route = routes[0];

            expect(route).toBeDefined();
            expect(route?.route.id).toBeDefined();
            expect(route?.route.url).toBeDefined();
            expect(route?.route.transport).toBeDefined();
            expect(route?.route.presets).toBeDefined();
        });

        /**
         * Tests Preset structure from Rust binding.
         * @see example/mocker/validation.ts - Preset type (PRESET_SCHEMA)
         */
        it('should have preset with variants', () => {
            controller.useCollection('base');
            const routes = controller.getActiveRoutes();
            const route = routes[0];

            expect(route?.preset.id).toBeDefined();
            expect(route?.preset.variants).toBeDefined();
            expect(Array.isArray(route?.preset.variants)).toBe(true);
        });

        /**
         * Tests Variant structure from Rust binding.
         * @see example/mocker/validation.ts - Variant type (VARIANT_SCHEMA)
         */
        it('should have variant with response data', () => {
            controller.useCollection('base');
            const routes = controller.getActiveRoutes();
            const usersRoute = routes.find(r => r.route.id === 'users-api');

            expect(usersRoute?.variant.id).toBe('default');
            expect(usersRoute?.variant.status).toBe(200);
            expect(usersRoute?.variant.headers).toBeDefined();
            expect(usersRoute?.variant.body).toBeDefined();
        });
    });

    describe('HTTP method handling', () => {
        /**
         * Tests GET method parsing.
         * @see example/mocker/consts.ts - HTTP_METHODS constant
         * @see example/mocker/validation.ts - method field validation
         */
        it('should correctly identify GET routes', () => {
            controller.useCollection('base');
            const routes = controller.getActiveRoutes();
            const usersRoute = routes.find(r => r.route.id === 'users-api');

            expect(usersRoute?.route.method).toBe(HttpMethod.Get);
        });

        /**
         * Tests POST method parsing.
         * @see example/mocker/consts.ts - HTTP_METHODS constant
         */
        it('should correctly identify POST routes', () => {
            controller.useCollection('extended');
            const routes = controller.getActiveRoutes();
            const ordersRoute = routes.find(r => r.route.id === 'orders-api');

            expect(ordersRoute?.route.method).toBe(HttpMethod.Post);
        });
    });

    describe('collection inheritance behavior', () => {
        /**
         * Tests that child collection includes parent routes (from field).
         * @see example/mocker/__specs__/controller.spec.ts - "should mock selected collection with nesting"
         * @see example/mocker/controller.ts - fillNestedRoutes method
         */
        it('should include parent routes in child collection', () => {
            controller.useCollection('extended');
            const routes = controller.getActiveRoutes();

            // extended inherits from base which has products-api
            const productsRoute = routes.find(r => r.route.id === 'products-api');
            expect(productsRoute).toBeDefined();
        });

        /**
         * Tests that child route overrides parent route with same id.
         * @see example/mocker/controller.ts - fillNestedRoutes skips if identifier exists
         */
        it('should override parent route with same route id', () => {
            controller.useCollection('override-parent');
            const routes = controller.getActiveRoutes();

            const usersRoute = routes.find(r => r.route.id === 'users-api');
            // Child's variant should override parent's
            expect(usersRoute?.variant.id).toBe('empty-list');
        });

        /**
         * Tests multi-level inheritance (grandparent → parent → child).
         * @see example/mocker/controller.ts - recursive fillNestedRoutes
         */
        it('should support multi-level inheritance', () => {
            controller.useCollection('deeply-nested');
            const routes = controller.getActiveRoutes();

            // Should have routes from: base -> extended -> deeply-nested
            const routeIds = routes.map(r => r.route.id);
            expect(routeIds).toContain('products-api'); // from base
            expect(routeIds).toContain('payments-api'); // from deeply-nested
        });
    });

    describe('transport types', () => {
        /**
         * Tests HTTP transport identification.
         * @see example/mocker/consts.ts - HTTP_METHODS vs WEBSOCKET_METHOD
         */
        it('should identify HTTP transport', () => {
            controller.useCollection('base');
            const routes = controller.getActiveRoutes();

            routes.forEach(route => {
                expect(route.route.transport).toBe(Transport.Http);
            });
        });

        /**
         * Tests WebSocket transport identification.
         * @see example/mocker/consts.ts - WEBSOCKET_METHOD = 'WS'
         */
        it('should identify WebSocket transport', () => {
            controller.useCollection('with-websocket');
            const routes = controller.getActiveRoutes();

            expect(routes[0]?.route.transport).toBe(Transport.WebSocket);
        });
    });

    describe('error variant handling', () => {
        /**
         * Tests error response variants with status codes and body.
         * @see example/mocker/validation.ts - Variant structure with status/body
         */
        it('should correctly parse error response variants', () => {
            controller.useCollection('extended');
            const routes = controller.getActiveRoutes();

            const usersRoute = routes.find(r => r.route.id === 'users-api');
            expect(usersRoute?.variant.status).toBe(404);

            const body = usersRoute?.variant.body as {error: string; code: string};
            expect(body.error).toBe('Users not found');
            expect(body.code).toBe('NOT_FOUND');
        });
    });

    /**
     * Features from JS implementation that may not be implemented yet.
     * These tests are marked as .todo and will be enabled when features are ready.
     *
     * @see example/mocker/__specs__/controller.spec.ts for original test cases
     */
    describe('TODO: features to implement', () => {
        /**
         * @see example/mocker/__specs__/controller.spec.ts - "should mock selected route for default collection"
         */
        it.todo('should allow switching individual routes without changing collection (useRoutes)');

        /**
         * @see example/mocker/__specs__/controller.spec.ts - "should mock selected route for default collection"
         */
        it.todo('should merge new routes with existing collection routes');

        /**
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if route is a websocket route"
         */
        it.todo('should support useSocket for WebSocket-only route switching');

        /**
         * @see example/mocker/controller.ts - getWebSocket method
         */
        it.todo('should provide getWebSocket method for WebSocket instance access');

        /**
         * @see example/mocker/__specs__/controller.spec.ts - "should restore mock to selected collection"
         */
        it.todo('should support resetRoutes to restore collection state');

        /**
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if collection have duplicated routes"
         */
        it.todo('should throw error for duplicated routes in collection');

        /**
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if collection with nesting not found"
         */
        it.todo('should throw error for non-existent parent collection (from field)');

        /**
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if route in collection not found"
         */
        it.todo('should throw error if route in collection not found');

        /**
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if preset in collection not found"
         */
        it.todo('should throw error if preset in collection not found');

        /**
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if variant in collection not found"
         */
        it.todo('should throw error if variant in collection not found');
    });
});
