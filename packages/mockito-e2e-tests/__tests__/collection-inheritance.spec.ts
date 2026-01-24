/**
 * Collection Inheritance Tests
 *
 * Tests for collection inheritance via 'from' field.
 *
 * @see example/mocker/__specs__/controller.spec.ts - "should mock selected collection with nesting"
 * @see example/mocker/controller.ts - fillNestedRoutes method
 */
import path from 'node:path';
import {describe, expect, it, beforeAll} from '@rstest/core';
import {MocksManager, MocksController} from '@mockito/binding';

const FIXTURES_PATH = path.join(import.meta.dirname, '..', '__fixtures__');
const COLLECTIONS_PATH = path.join(FIXTURES_PATH, 'collections.yaml');
const ROUTES_PATH = path.join(FIXTURES_PATH, 'routes', '*.yaml');

describe('Collection Inheritance', () => {
    let manager: MocksManager;

    beforeAll(() => {
        manager = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);
    });

    describe('single level inheritance', () => {
        /**
         * Tests basic inheritance with 'from' field.
         * @see example/mocker/__specs__/controller.spec.ts - "should mock selected collection with nesting"
         */
        it('should inherit routes from parent collection', () => {
            const parentRoutes = manager.resolveCollection('base');
            const childRoutes = manager.resolveCollection('extended');

            // Parent has 2 routes
            expect(parentRoutes).toHaveLength(2);

            // Child inherits from parent and adds more
            expect(childRoutes.length).toBeGreaterThan(parentRoutes.length);

            // Parent routes should be present in child
            const parentIds = parentRoutes.map(r => r.route.id);
            const childIds = childRoutes.map(r => r.route.id);

            parentIds.forEach(id => {
                expect(childIds).toContain(id);
            });
        });

        /**
         * Tests that child routes override parent routes.
         * @see example/mocker/controller.ts - fillNestedRoutes: child wins
         */
        it('should override parent route when child defines same route id', () => {
            const parentRoutes = manager.resolveCollection('base');
            const childRoutes = manager.resolveCollection('override-parent');

            const parentUsersRoute = parentRoutes.find(r => r.route.id === 'users-api');
            const childUsersRoute = childRoutes.find(r => r.route.id === 'users-api');

            // Parent has default variant
            expect(parentUsersRoute?.variant.id).toBe('default');

            // Child overrides with empty-list variant
            expect(childUsersRoute?.variant.id).toBe('empty-list');
        });
    });

    describe('multi-level inheritance', () => {
        /**
         * Tests two-level inheritance chain.
         * @see example/mocker/controller.ts - recursive fillNestedRoutes
         */
        it('should support two-level inheritance', () => {
            const level1Routes = manager.resolveCollection('level-1');
            const level2Routes = manager.resolveCollection('level-2');

            expect(level1Routes).toHaveLength(1);
            expect(level2Routes).toHaveLength(2);

            const level2Ids = level2Routes.map(r => r.route.id);
            expect(level2Ids).toContain('users-api');
            expect(level2Ids).toContain('products-api');
        });

        /**
         * Tests three-level inheritance chain.
         * @see example/mocker/controller.ts - recursive fillNestedRoutes
         */
        it('should support three-level inheritance', () => {
            const level3Routes = manager.resolveCollection('level-3');

            expect(level3Routes).toHaveLength(3);

            const ids = level3Routes.map(r => r.route.id);
            expect(ids).toContain('users-api');
            expect(ids).toContain('products-api');
            expect(ids).toContain('orders-api');
        });

        /**
         * Tests four-level inheritance chain.
         * @see example/mocker/controller.ts - recursive fillNestedRoutes
         */
        it('should support four-level inheritance', () => {
            const level4Routes = manager.resolveCollection('level-4');

            expect(level4Routes).toHaveLength(4);

            const ids = level4Routes.map(r => r.route.id);
            expect(ids).toContain('users-api');
            expect(ids).toContain('products-api');
            expect(ids).toContain('orders-api');
            expect(ids).toContain('payments-api');
        });

        /**
         * Tests deeply nested collection (base → extended → deeply-nested).
         * @see example/mocker/controller.ts - recursive fillNestedRoutes
         */
        it('should resolve deeply-nested inheritance chain', () => {
            const routes = manager.resolveCollection('deeply-nested');

            // Should have all routes from the chain
            const ids = routes.map(r => r.route.id);
            expect(ids).toContain('users-api');     // from base
            expect(ids).toContain('products-api');  // from base
            expect(ids).toContain('orders-api');    // from extended
            expect(ids).toContain('payments-api');  // from deeply-nested
        });
    });

    describe('override behavior in inheritance chain', () => {
        /**
         * Tests override at second level.
         * @see example/mocker/controller.ts - fillNestedRoutes identifier check
         */
        it('should use child preset when overriding', () => {
            const routes = manager.resolveCollection('extended');
            const usersRoute = routes.find(r => r.route.id === 'users-api');

            // extended defines users-api:error:not-found which overrides base's users-api:success:default
            expect(usersRoute?.preset.id).toBe('error');
            expect(usersRoute?.variant.id).toBe('not-found');
        });

        /**
         * Tests that non-overridden routes keep parent values.
         * @see example/mocker/controller.ts - fillNestedRoutes preserves parent routes
         */
        it('should preserve parent routes that are not overridden', () => {
            const routes = manager.resolveCollection('extended');
            const productsRoute = routes.find(r => r.route.id === 'products-api');

            // products-api is not overridden, should have parent's preset
            expect(productsRoute?.preset.id).toBe('success');
            expect(productsRoute?.variant.id).toBe('default');
        });
    });

    describe('collections without inheritance', () => {
        /**
         * Tests collection without 'from' field.
         * @see example/mocker/controller.ts - fillNestedRoutes returns early if !collection.from
         */
        it('should work without inheritance', () => {
            const routes = manager.resolveCollection('standalone');

            // standalone has its own routes only
            expect(routes.length).toBeGreaterThanOrEqual(1);
        });

        /**
         * Tests minimal collection.
         * @see example/mocker/controller.ts - basic collection handling
         */
        it('should resolve minimal collection', () => {
            const routes = manager.resolveCollection('minimal');

            expect(routes).toHaveLength(1);
            expect(routes[0]?.route.id).toBe('users-api');
        });
    });

    describe('MocksController inheritance', () => {
        /**
         * Tests inheritance via MocksController.
         * @see example/mocker/__specs__/controller.spec.ts - "should mock selected collection with nesting"
         */
        it('should resolve inheritance when using controller', () => {
            const controller = new MocksController(COLLECTIONS_PATH, ROUTES_PATH);

            controller.useCollection('deeply-nested');
            const routes = controller.getActiveRoutes();

            const ids = routes.map(r => r.route.id);
            expect(ids).toContain('users-api');
            expect(ids).toContain('products-api');
            expect(ids).toContain('orders-api');
            expect(ids).toContain('payments-api');
        });

        /**
         * Tests switching between collections with different inheritance.
         * @see example/mocker/controller.ts - useCollection resets state
         */
        it('should correctly switch between collections with different inheritance depth', () => {
            const controller = new MocksController(COLLECTIONS_PATH, ROUTES_PATH);

            controller.useCollection('base');
            expect(controller.getActiveRoutes()).toHaveLength(2);

            controller.useCollection('level-4');
            expect(controller.getActiveRoutes()).toHaveLength(4);

            controller.useCollection('minimal');
            expect(controller.getActiveRoutes()).toHaveLength(1);
        });
    });

    describe('inheritance with overrides at each level', () => {
        /**
         * Tests override propagation through inheritance chain.
         * @see example/mocker/controller.ts - fillNestedRoutes order matters
         */
        it('should propagate overrides correctly through chain', () => {
            // extended overrides users-api from base
            const extendedRoutes = manager.resolveCollection('extended');
            const extendedUsers = extendedRoutes.find(r => r.route.id === 'users-api');
            expect(extendedUsers?.preset.id).toBe('error');

            // deeply-nested inherits from extended, keeps the override
            const deepRoutes = manager.resolveCollection('deeply-nested');
            const deepUsers = deepRoutes.find(r => r.route.id === 'users-api');
            expect(deepUsers?.preset.id).toBe('error');
        });
    });

    describe('inheritance with full-api collection', () => {
        /**
         * Tests collection with many routes but no inheritance.
         * @see example/mocker/controller.ts - routes array handling
         */
        it('should handle collection with many routes', () => {
            const routes = manager.resolveCollection('full-api');

            expect(routes.length).toBe(6);

            const ids = routes.map(r => r.route.id);
            expect(ids).toContain('users-api');
            expect(ids).toContain('products-api');
            expect(ids).toContain('orders-api');
            expect(ids).toContain('health-check');
            expect(ids).toContain('search-api');
            expect(ids).toContain('auth-api');
        });
    });
});
