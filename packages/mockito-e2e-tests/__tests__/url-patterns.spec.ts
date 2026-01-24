/**
 * URL Patterns Tests
 *
 * Tests for different URL patterns and path parameters.
 *
 * @see example/mocker/validation.ts - url field
 * @see example/mocker/lib/url-matches.ts - URL matching utilities
 */
import path from 'node:path';
import {describe, expect, it, beforeAll} from '@rstest/core';
import {MocksManager} from '@mockito/binding';

const FIXTURES_PATH = path.join(import.meta.dirname, '..', '__fixtures__');
const COLLECTIONS_PATH = path.join(FIXTURES_PATH, 'collections.yaml');
const ROUTES_PATH = path.join(FIXTURES_PATH, 'routes', '*.yaml');

describe('URL Patterns', () => {
    let manager: MocksManager;

    beforeAll(() => {
        manager = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);
    });

    describe('simple paths', () => {
        /**
         * Tests simple API path.
         * @see example/mocker/validation.ts - url string field
         */
        it('should parse /api/users path', () => {
            const routes = manager.resolveCollection('base');
            const route = routes.find(r => r.route.id === 'users-api');

            expect(route?.route.url).toBe('/api/users');
        });

        /**
         * Tests another simple path.
         * @see example/mocker/validation.ts - url string field
         */
        it('should parse /api/products path', () => {
            const routes = manager.resolveCollection('base');
            const route = routes.find(r => r.route.id === 'products-api');

            expect(route?.route.url).toBe('/api/products');
        });

        /**
         * Tests root-level path.
         * @see example/mocker/validation.ts - url string field
         */
        it('should parse /health path', () => {
            const routes = manager.resolveCollection('health');
            const route = routes.find(r => r.route.id === 'health-check');

            expect(route?.route.url).toBe('/health');
        });
    });

    describe('paths with parameters', () => {
        /**
         * Tests path with :id parameter.
         * @see example/mocker/lib/url-matches.ts - isTemplate, fillUrlWithParameters
         */
        it('should parse /api/users/:id path', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-crud');

            expect(route?.route.url).toBe('/api/users/:id');
        });

        /**
         * Tests PUT route with :id parameter.
         * @see example/mocker/lib/url-matches.ts - URL parameters
         */
        it('should parse PUT /api/users/:id path', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-put');

            expect(route?.route.url).toBe('/api/users/:id');
        });

        /**
         * Tests PATCH route with :id parameter.
         * @see example/mocker/lib/url-matches.ts - URL parameters
         */
        it('should parse PATCH /api/users/:id path', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-patch');

            expect(route?.route.url).toBe('/api/users/:id');
        });

        /**
         * Tests DELETE route with :id parameter.
         * @see example/mocker/lib/url-matches.ts - URL parameters
         */
        it('should parse DELETE /api/users/:id path', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-delete');

            expect(route?.route.url).toBe('/api/users/:id');
        });
    });

    describe('nested paths', () => {
        /**
         * Tests nested path for auth.
         * @see example/mocker/validation.ts - url string field
         */
        it('should parse /api/auth/login path', () => {
            const routes = manager.resolveCollection('auth-success');
            const route = routes.find(r => r.route.id === 'auth-api');

            expect(route?.route.url).toBe('/api/auth/login');
        });

        /**
         * Tests nested path for search.
         * @see example/mocker/validation.ts - url string field
         */
        it('should parse /api/search path', () => {
            const routes = manager.resolveCollection('search');
            const route = routes.find(r => r.route.id === 'search-api');

            expect(route?.route.url).toBe('/api/search');
        });

        /**
         * Tests nested path for files.
         * @see example/mocker/validation.ts - url string field
         */
        it('should parse /api/files path', () => {
            const routes = manager.resolveCollection('file-upload');
            const route = routes.find(r => r.route.id === 'file-upload');

            expect(route?.route.url).toBe('/api/files');
        });

        /**
         * Tests nested path for payments.
         * @see example/mocker/validation.ts - url string field
         */
        it('should parse /api/payments path', () => {
            const routes = manager.resolveCollection('deeply-nested');
            const route = routes.find(r => r.route.id === 'payments-api');

            expect(route?.route.url).toBe('/api/payments');
        });
    });

    describe('WebSocket paths', () => {
        /**
         * Tests WebSocket notifications path.
         * @see example/mocker/validation.ts - url for WS routes
         */
        it('should parse /ws/notifications path', () => {
            const routes = manager.resolveCollection('with-websocket');
            const route = routes.find(r => r.route.id === 'ws-notifications');

            expect(route?.route.url).toBe('/ws/notifications');
        });

        /**
         * Tests WebSocket chat path.
         * @see example/mocker/validation.ts - url for WS routes
         */
        it('should parse /ws/chat path', () => {
            const routes = manager.resolveCollection('ws-chat');
            const route = routes.find(r => r.route.id === 'ws-chat');

            expect(route?.route.url).toBe('/ws/chat');
        });
    });

    describe('URL uniqueness across routes', () => {
        /**
         * Tests that different routes can have the same URL with different methods.
         * @see example/mocker/validation.ts - route definition
         */
        it('should allow same URL with different methods', () => {
            const routes = manager.resolveCollection('crud-operations');

            const usersCrud = routes.find(r => r.route.id === 'users-crud');
            const usersDelete = routes.find(r => r.route.id === 'users-delete');

            // Both use /api/users/:id but different methods
            expect(usersCrud?.route.url).toBe('/api/users/:id');
            expect(usersDelete?.route.url).toBe('/api/users/:id');
        });

        /**
         * Tests different URLs for POST routes.
         * @see example/mocker/validation.ts - route definition
         */
        it('should have different URLs for different POST resources', () => {
            const routes = manager.resolveCollection('crud-operations');

            const usersPost = routes.find(r => r.route.id === 'users-post');

            expect(usersPost?.route.url).toBe('/api/users');
        });
    });

    describe('preset params matching URL params', () => {
        /**
         * Tests that preset params correspond to URL params.
         * @see example/mocker/validation.ts - params in preset
         */
        it('should have preset params matching URL params', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-crud');

            // URL has :id, preset has params.id
            expect(route?.route.url).toContain(':id');
            expect(route?.preset.params?.id).toBe('123');
        });

        /**
         * Tests PUT route preset params.
         * @see example/mocker/validation.ts - params in preset
         */
        it('should have PUT preset params', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-put');

            expect(route?.preset.params?.id).toBe('123');
        });

        /**
         * Tests PATCH route preset params.
         * @see example/mocker/validation.ts - params in preset
         */
        it('should have PATCH preset params', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-patch');

            expect(route?.preset.params?.id).toBe('123');
        });
    });
});
