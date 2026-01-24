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

    describe('useRoutes', () => {
        /**
         * Tests switching individual route variant without changing collection.
         * @see example/mocker/__specs__/controller.spec.ts - "should mock selected route for default collection"
         */
        it('should switch individual route variant for selected collection', () => {
            controller.useCollection('base');

            // Initial state
            const initialRoutes = controller.getActiveRoutes();
            const initialUsersRoute = initialRoutes.find(r => r.route.id === 'users-api');
            expect(initialUsersRoute?.variant.id).toBe('default');

            // Switch users-api to error:not-found variant
            controller.useRoutes(['users-api:error:not-found']);

            const updatedRoutes = controller.getActiveRoutes();
            const updatedUsersRoute = updatedRoutes.find(r => r.route.id === 'users-api');

            expect(updatedUsersRoute?.variant.id).toBe('not-found');
            expect(updatedUsersRoute?.preset.id).toBe('error');
        });

        /**
         * Tests that existing routes are preserved when adding new routes.
         * @see example/mocker/__specs__/controller.spec.ts - "should mock selected route for default collection"
         */
        it('should merge new routes with existing collection routes', () => {
            controller.useCollection('base');

            const initialRoutes = controller.getActiveRoutes();
            expect(initialRoutes).toHaveLength(2); // users-api and products-api

            // Add orders-api without removing existing routes
            controller.useRoutes(['orders-api:success:default']);

            const updatedRoutes = controller.getActiveRoutes();
            expect(updatedRoutes).toHaveLength(3);

            const routeIds = updatedRoutes.map(r => r.route.id);
            expect(routeIds).toContain('users-api');
            expect(routeIds).toContain('products-api');
            expect(routeIds).toContain('orders-api');
        });

        /**
         * Tests that routes with same ID are overridden.
         * @see example/mocker/controller.ts - useRoutes method
         */
        it('should override existing route when same route:preset combination is used', () => {
            controller.useCollection('base');

            // Initial: users-api:success:default
            const initialRoutes = controller.getActiveRoutes();
            const initialUsersRoute = initialRoutes.find(r => r.route.id === 'users-api');
            expect(initialUsersRoute?.variant.id).toBe('default');

            // Override with different preset/variant
            controller.useRoutes(['users-api:success:empty-list']);

            const updatedRoutes = controller.getActiveRoutes();
            // Should still have 2 routes, not 3
            expect(updatedRoutes).toHaveLength(2);

            const updatedUsersRoute = updatedRoutes.find(r => r.route.id === 'users-api');
            expect(updatedUsersRoute?.variant.id).toBe('empty-list');
        });

        /**
         * Tests error handling when route not found.
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if route not found"
         */
        it('should throw error if route not found', () => {
            controller.useCollection('base');

            expect(() => controller.useRoutes(['nonexistent-route:preset:variant'])).toThrow();
        });

        /**
         * Tests error handling when preset not found.
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if preset for route not found"
         */
        it('should throw error if preset not found', () => {
            controller.useCollection('base');

            expect(() => controller.useRoutes(['users-api:nonexistent-preset:default'])).toThrow();
        });

        /**
         * Tests error handling when variant not found.
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if variant for route not found"
         */
        it('should throw error if variant not found', () => {
            controller.useCollection('base');

            expect(() => controller.useRoutes(['users-api:success:nonexistent-variant'])).toThrow();
        });

        /**
         * Tests error handling for invalid route reference format.
         */
        it('should throw error for invalid route reference format', () => {
            controller.useCollection('base');

            expect(() => controller.useRoutes(['invalid-format'])).toThrow();
        });

        /**
         * Tests useRoutes without selecting a collection first.
         * @see example/mocker/__specs__/controller.spec.ts - "should restore mock to nothing if no collection selected"
         */
        it('should work without collection selected', () => {
            // No collection selected
            expect(controller.getActiveRoutes()).toHaveLength(0);

            // Add route directly
            controller.useRoutes(['users-api:success:default']);

            expect(controller.getActiveRoutes()).toHaveLength(1);
            expect(controller.getActiveRoutes()[0]?.route.id).toBe('users-api');
        });

        /**
         * Tests switching multiple routes at once.
         */
        it('should handle multiple routes in single call', () => {
            controller.useCollection('base');

            // Override users-api and add orders-api
            controller.useRoutes([
                'users-api:error:not-found',
                'orders-api:success:default'
            ]);

            const routes = controller.getActiveRoutes();
            expect(routes).toHaveLength(3);

            const usersRoute = routes.find(r => r.route.id === 'users-api');
            const ordersRoute = routes.find(r => r.route.id === 'orders-api');

            expect(usersRoute?.variant.id).toBe('not-found');
            expect(ordersRoute?.variant.id).toBe('default');
        });

        /**
         * Tests that useRoutes does not change currentCollection.
         */
        it('should not change currentCollection', () => {
            controller.useCollection('base');
            expect(controller.currentCollection).toBe('base');

            controller.useRoutes(['users-api:error:not-found']);

            expect(controller.currentCollection).toBe('base');
        });

        /**
         * Tests fail-fast behavior - if one route fails, no changes are applied.
         */
        it('should fail fast and not apply partial changes', () => {
            controller.useCollection('base');

            const initialRoutes = controller.getActiveRoutes();
            const initialUsersVariant = initialRoutes.find(r => r.route.id === 'users-api')?.variant.id;

            // First route is valid, second is invalid
            expect(() => controller.useRoutes([
                'users-api:success:empty-list',
                'nonexistent:preset:variant'
            ])).toThrow();

            // Routes should remain unchanged
            const currentRoutes = controller.getActiveRoutes();
            const currentUsersVariant = currentRoutes.find(r => r.route.id === 'users-api')?.variant.id;

            expect(currentUsersVariant).toBe(initialUsersVariant);
        });

        /**
         * Tests that useRoutes throws error for WebSocket routes.
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if route is a websocket route"
         */
        it('should throw error for WebSocket route', () => {
            expect(() => controller.useRoutes(['ws-notifications:default:message'])).toThrow(
                /Use 'useSocket' instead/
            );
        });
    });

    describe('useSocket', () => {
        /**
         * Tests basic WebSocket route switching.
         * @see example/mocker/__specs__/controller.spec.ts - "should pass parser url for websocket route"
         */
        it('should switch WebSocket route', () => {
            // Use socket route directly
            controller.useSocket(['ws-notifications:default:message']);

            const routes = controller.getActiveRoutes();
            expect(routes).toHaveLength(1);
            expect(routes[0]?.route.id).toBe('ws-notifications');
            expect(routes[0]?.route.transport).toBe(Transport.WebSocket);
        });

        /**
         * Tests switching WebSocket variant.
         */
        it('should switch WebSocket route variant', () => {
            controller.useCollection('with-websocket');

            const initialRoutes = controller.getActiveRoutes();
            expect(initialRoutes[0]?.variant.id).toBe('message');

            // Switch to ws-chat which has different variants
            controller.useSocket(['ws-chat:default:connected']);

            const routes = controller.getActiveRoutes();
            const wsChat = routes.find(r => r.route.id === 'ws-chat');
            expect(wsChat?.variant.id).toBe('connected');
        });

        /**
         * Tests merging WebSocket routes with existing routes.
         */
        it('should merge WebSocket routes with existing routes', () => {
            controller.useCollection('with-websocket');

            // Initial: 1 WebSocket route
            expect(controller.getActiveRoutes()).toHaveLength(1);

            // Add another WebSocket route
            controller.useSocket(['ws-chat:default:message']);

            expect(controller.getActiveRoutes()).toHaveLength(2);
            const routeIds = controller.getActiveRoutes().map(r => r.route.id);
            expect(routeIds).toContain('ws-notifications');
            expect(routeIds).toContain('ws-chat');
        });

        /**
         * Tests error when trying to use HTTP route with useSocket.
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if route is not a websocket route"
         */
        it('should throw error for HTTP route', () => {
            expect(() => controller.useSocket(['users-api:success:default'])).toThrow(
                /Use 'useRoutes' instead/
            );
        });

        /**
         * Tests error when WebSocket route not found.
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if websocket route not found"
         */
        it('should throw error if route not found', () => {
            expect(() => controller.useSocket(['nonexistent:preset:variant'])).toThrow();
        });

        /**
         * Tests error when preset not found.
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if preset for websocket route not found"
         */
        it('should throw error if preset not found', () => {
            expect(() => controller.useSocket(['ws-notifications:nonexistent-preset:message'])).toThrow();
        });

        /**
         * Tests error when variant not found.
         * @see example/mocker/__specs__/controller.spec.ts - "should throw if variant for websocket route not found"
         */
        it('should throw error if variant not found', () => {
            expect(() => controller.useSocket(['ws-notifications:default:nonexistent-variant'])).toThrow();
        });

        /**
         * Tests fail-fast behavior for useSocket.
         */
        it('should fail fast and not apply partial changes', () => {
            controller.useSocket(['ws-notifications:default:message']);

            const initialRoutes = controller.getActiveRoutes();
            expect(initialRoutes).toHaveLength(1);

            // First route is valid, second is invalid
            expect(() => controller.useSocket([
                'ws-chat:default:message',
                'nonexistent:preset:variant'
            ])).toThrow();

            // Routes should remain unchanged
            expect(controller.getActiveRoutes()).toHaveLength(1);
            expect(controller.getActiveRoutes()[0]?.route.id).toBe('ws-notifications');
        });

        /**
         * Tests useSocket without selecting a collection first.
         */
        it('should work without collection selected', () => {
            // No collection selected
            expect(controller.getActiveRoutes()).toHaveLength(0);

            // Add WebSocket route directly
            controller.useSocket(['ws-notifications:default:message']);

            expect(controller.getActiveRoutes()).toHaveLength(1);
            expect(controller.getActiveRoutes()[0]?.route.transport).toBe(Transport.WebSocket);
        });

        /**
         * Tests handling multiple WebSocket routes in single call.
         */
        it('should handle multiple routes in single call', () => {
            controller.useSocket([
                'ws-notifications:default:message',
                'ws-chat:default:connected'
            ]);

            const routes = controller.getActiveRoutes();
            expect(routes).toHaveLength(2);

            const routeIds = routes.map(r => r.route.id);
            expect(routeIds).toContain('ws-notifications');
            expect(routeIds).toContain('ws-chat');
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
