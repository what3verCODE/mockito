/**
 * HTTP Methods Tests
 *
 * Tests for all HTTP methods support (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS).
 *
 * @see example/mocker/consts.ts - HTTP_METHODS constant
 * @see example/mocker/validation.ts - method field validation
 */
import path from 'node:path';
import {describe, expect, it, beforeAll} from '@rstest/core';
import {MocksManager, HttpMethod, Transport} from '@mockito/binding';

const FIXTURES_PATH = path.join(import.meta.dirname, '..', '__fixtures__');
const COLLECTIONS_PATH = path.join(FIXTURES_PATH, 'collections.yaml');
const ROUTES_PATH = path.join(FIXTURES_PATH, 'routes', '*.yaml');

describe('HTTP Methods', () => {
    let manager: MocksManager;

    beforeAll(() => {
        manager = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);
    });

    describe('CRUD operations', () => {
        /**
         * Tests GET method for reading resources.
         * @see example/mocker/consts.ts - HTTP_METHODS includes 'GET'
         */
        it('should support GET method', () => {
            const routes = manager.resolveCollection('crud-operations');
            const getRoute = routes.find(r => r.route.id === 'users-crud');

            expect(getRoute).toBeDefined();
            expect(getRoute?.route.method).toBe(HttpMethod.Get);
            expect(getRoute?.route.url).toBe('/api/users/:id');
        });

        /**
         * Tests POST method for creating resources.
         * @see example/mocker/consts.ts - HTTP_METHODS includes 'POST'
         */
        it('should support POST method', () => {
            const routes = manager.resolveCollection('crud-operations');
            const postRoute = routes.find(r => r.route.id === 'users-post');

            expect(postRoute).toBeDefined();
            expect(postRoute?.route.method).toBe(HttpMethod.Post);
            expect(postRoute?.variant.status).toBe(201);
        });

        /**
         * Tests PUT method for replacing resources.
         * @see example/mocker/consts.ts - HTTP_METHODS includes 'PUT'
         */
        it('should support PUT method', () => {
            const routes = manager.resolveCollection('crud-operations');
            const putRoute = routes.find(r => r.route.id === 'users-put');

            expect(putRoute).toBeDefined();
            expect(putRoute?.route.method).toBe(HttpMethod.Put);
        });

        /**
         * Tests PATCH method for partial updates.
         * @see example/mocker/consts.ts - HTTP_METHODS includes 'PATCH'
         */
        it('should support PATCH method', () => {
            const routes = manager.resolveCollection('crud-operations');
            const patchRoute = routes.find(r => r.route.id === 'users-patch');

            expect(patchRoute).toBeDefined();
            expect(patchRoute?.route.method).toBe(HttpMethod.Patch);
        });

        /**
         * Tests DELETE method for removing resources.
         * @see example/mocker/consts.ts - HTTP_METHODS includes 'DELETE'
         */
        it('should support DELETE method', () => {
            const routes = manager.resolveCollection('crud-operations');
            const deleteRoute = routes.find(r => r.route.id === 'users-delete');

            expect(deleteRoute).toBeDefined();
            expect(deleteRoute?.route.method).toBe(HttpMethod.Delete);
            expect(deleteRoute?.variant.status).toBe(204);
        });
    });

    describe('response status codes', () => {
        /**
         * Tests 200 OK status.
         * @see example/mocker/validation.ts - Variant status field
         */
        it('should return 200 for successful GET', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-crud');

            expect(route?.variant.status).toBe(200);
        });

        /**
         * Tests 201 Created status.
         * @see example/mocker/validation.ts - Variant status field
         */
        it('should return 201 for successful POST', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-post');

            expect(route?.variant.status).toBe(201);
        });

        /**
         * Tests 204 No Content status.
         * @see example/mocker/validation.ts - Variant status field
         */
        it('should return 204 for successful DELETE', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-delete');

            expect(route?.variant.status).toBe(204);
        });

        /**
         * Tests 400 Bad Request status.
         * @see example/mocker/validation.ts - Variant status field
         */
        it('should return 400 for validation error', () => {
            const routes = manager.resolveCollection('error-responses');
            const route = routes.find(r => r.route.id === 'users-post');

            expect(route?.variant.status).toBe(400);
        });

        /**
         * Tests 401 Unauthorized status.
         * @see example/mocker/validation.ts - Variant status field
         */
        it('should return 401 for authentication error', () => {
            const routes = manager.resolveCollection('auth-failed');
            const route = routes.find(r => r.route.id === 'auth-api');

            expect(route?.variant.status).toBe(401);
        });

        /**
         * Tests 404 Not Found status.
         * @see example/mocker/validation.ts - Variant status field
         */
        it('should return 404 for not found', () => {
            const routes = manager.resolveCollection('error-responses');
            const route = routes.find(r => r.route.id === 'users-crud');

            expect(route?.variant.status).toBe(404);
        });

        /**
         * Tests 429 Too Many Requests status.
         * @see example/mocker/validation.ts - Variant status field
         */
        it('should return 429 for rate limiting', () => {
            const routes = manager.resolveCollection('auth-rate-limited');
            const route = routes.find(r => r.route.id === 'auth-api');

            expect(route?.variant.status).toBe(429);
        });

        /**
         * Tests 503 Service Unavailable status.
         * @see example/mocker/validation.ts - Variant status field
         */
        it('should return 503 for service unavailable', () => {
            const routes = manager.resolveCollection('health-unhealthy');
            const route = routes.find(r => r.route.id === 'health-check');

            expect(route?.variant.status).toBe(503);
        });
    });

    describe('response headers', () => {
        /**
         * Tests Content-Type header preservation.
         * @see example/mocker/validation.ts - headers in Variant
         */
        it('should include Content-Type header', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-crud');

            expect(route?.variant.headers?.['Content-Type']).toBe('application/json');
        });

        /**
         * Tests Location header for created resources.
         * @see example/mocker/validation.ts - headers in Variant
         */
        it('should include Location header for 201 response', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-post');

            expect(route?.variant.headers?.['Location']).toBe('/api/users/999');
        });

        /**
         * Tests Set-Cookie header.
         * @see example/mocker/validation.ts - headers in Variant
         */
        it('should include Set-Cookie header for auth', () => {
            const routes = manager.resolveCollection('auth-success');
            const route = routes.find(r => r.route.id === 'auth-api');

            expect(route?.variant.headers?.['Set-Cookie']).toBe('session=abc123; HttpOnly');
        });

        /**
         * Tests Retry-After header.
         * @see example/mocker/validation.ts - headers in Variant
         */
        it('should include Retry-After header for rate limiting', () => {
            const routes = manager.resolveCollection('auth-rate-limited');
            const route = routes.find(r => r.route.id === 'auth-api');

            expect(route?.variant.headers?.['Retry-After']).toBe('60');
        });
    });

    describe('preset conditions', () => {
        /**
         * Tests params in preset.
         * @see example/mocker/validation.ts - params in Preset
         */
        it('should parse URL params in preset', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-crud');

            expect(route?.preset.params).toBeDefined();
            expect(route?.preset.params?.id).toBe('123');
        });

        /**
         * Tests headers condition in preset.
         * @see example/mocker/validation.ts - headers in Preset
         */
        it('should parse headers condition in preset', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-post');

            expect(route?.preset.headers).toBeDefined();
        });

        /**
         * Tests payload condition in preset.
         * @see example/mocker/validation.ts - payload in Preset
         */
        it('should parse payload condition in preset', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-post');

            expect(route?.preset.payload).toBeDefined();
        });

        /**
         * Tests query condition in preset.
         * @see example/mocker/validation.ts - query in Preset
         */
        it('should parse query condition in preset', () => {
            const routes = manager.resolveCollection('search');
            const route = routes.find(r => r.route.id === 'search-api');

            expect(route?.preset.query).toBeDefined();
            expect(route?.preset.query?.q).toBe('test');
            expect(route?.preset.query?.page).toBe('1');
        });
    });

    describe('all routes have HTTP transport', () => {
        /**
         * Tests that all CRUD routes have HTTP transport.
         * @see example/mocker/consts.ts - Transport types
         */
        it('should have HTTP transport for all CRUD routes', () => {
            const routes = manager.resolveCollection('crud-operations');

            routes.forEach(route => {
                expect(route.route.transport).toBe(Transport.Http);
            });
        });
    });
});
