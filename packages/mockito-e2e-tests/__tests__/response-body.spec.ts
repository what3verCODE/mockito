/**
 * Response Body Tests
 *
 * Tests for different response body types and structures.
 *
 * @see example/mocker/validation.ts - body field in Variant
 * @see example/mocker/__specs__/loader.spec.ts - YAML content preservation
 */
import path from 'node:path';
import {describe, expect, it, beforeAll} from '@rstest/core';
import {MocksManager} from '@mockito/binding';

const FIXTURES_PATH = path.join(import.meta.dirname, '..', '__fixtures__');
const COLLECTIONS_PATH = path.join(FIXTURES_PATH, 'collections.yaml');
const ROUTES_PATH = path.join(FIXTURES_PATH, 'routes', '*.yaml');

describe('Response Body', () => {
    let manager: MocksManager;

    beforeAll(() => {
        manager = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);
    });

    describe('object body', () => {
        /**
         * Tests simple object body.
         * @see example/mocker/__specs__/loader.spec.ts - object parsing
         */
        it('should preserve object body structure', () => {
            const routes = manager.resolveCollection('base');
            const usersRoute = routes.find(r => r.route.id === 'users-api');

            const body = usersRoute?.variant.body as Record<string, unknown>;

            expect(typeof body).toBe('object');
            expect(body).not.toBeNull();
        });

        /**
         * Tests nested object body.
         * @see example/mocker/__specs__/loader.spec.ts - nested object parsing
         */
        it('should preserve nested object structure', () => {
            const routes = manager.resolveCollection('base');
            const usersRoute = routes.find(r => r.route.id === 'users-api');

            const body = usersRoute?.variant.body as {
                data: Array<{ id: number; name: string; email: string }>;
                total: number;
            };

            expect(body.data).toBeDefined();
            expect(body.data[0]).toHaveProperty('id');
            expect(body.data[0]).toHaveProperty('name');
            expect(body.data[0]).toHaveProperty('email');
        });
    });

    describe('array body', () => {
        /**
         * Tests array in body.
         * @see example/mocker/__specs__/loader.spec.ts - array parsing
         */
        it('should preserve array in body', () => {
            const routes = manager.resolveCollection('base');
            const usersRoute = routes.find(r => r.route.id === 'users-api');

            const body = usersRoute?.variant.body as { data: unknown[] };

            expect(Array.isArray(body.data)).toBe(true);
            expect(body.data).toHaveLength(2);
        });

        /**
         * Tests nested arrays.
         * @see example/mocker/__specs__/loader.spec.ts - nested array parsing
         */
        it('should preserve nested array of objects', () => {
            const routes = manager.resolveCollection('base');
            const productsRoute = routes.find(r => r.route.id === 'products-api');

            const body = productsRoute?.variant.body as {
                products: Array<{ id: number; name: string; price: number }>;
            };

            expect(body.products).toHaveLength(2);
            expect(body.products[0]).toMatchObject({
                id: 1,
                name: 'Product A',
                price: 99.99,
            });
        });
    });

    describe('primitive values', () => {
        /**
         * Tests number values in body.
         * @see example/mocker/__specs__/loader.spec.ts - number parsing
         */
        it('should preserve number values', () => {
            const routes = manager.resolveCollection('base');
            const usersRoute = routes.find(r => r.route.id === 'users-api');

            const body = usersRoute?.variant.body as { total: number };

            expect(typeof body.total).toBe('number');
            expect(body.total).toBe(2);
        });

        /**
         * Tests string values in body.
         * @see example/mocker/__specs__/loader.spec.ts - string parsing
         */
        it('should preserve string values', () => {
            const routes = manager.resolveCollection('base');
            const usersRoute = routes.find(r => r.route.id === 'users-api');

            const body = usersRoute?.variant.body as {
                data: Array<{ name: string }>;
            };

            expect(typeof body.data[0].name).toBe('string');
            expect(body.data[0].name).toBe('John Doe');
        });

        /**
         * Tests boolean values in body.
         * @see example/mocker/__specs__/loader.spec.ts - boolean parsing
         */
        it('should preserve boolean values', () => {
            const routes = manager.resolveCollection('crud-operations');
            const postRoute = routes.find(r => r.route.id === 'users-post');

            const body = postRoute?.variant.body as { created: boolean };

            expect(typeof body.created).toBe('boolean');
            expect(body.created).toBe(true);
        });

        /**
         * Tests float number values.
         * @see example/mocker/__specs__/loader.spec.ts - float parsing
         */
        it('should preserve float values', () => {
            const routes = manager.resolveCollection('base');
            const productsRoute = routes.find(r => r.route.id === 'products-api');

            const body = productsRoute?.variant.body as {
                products: Array<{ price: number }>;
            };

            expect(body.products[0].price).toBe(99.99);
            expect(body.products[1].price).toBe(149.99);
        });
    });

    describe('error response body', () => {
        /**
         * Tests error object structure.
         * @see example/mocker/validation.ts - error response body
         */
        it('should preserve error object', () => {
            const routes = manager.resolveCollection('error-responses');
            const route = routes.find(r => r.route.id === 'users-crud');

            const body = route?.variant.body as {
                error: string;
                code: string;
            };

            expect(body.error).toBe('User not found');
            expect(body.code).toBe('USER_NOT_FOUND');
        });

        /**
         * Tests validation error with fields array.
         * @see example/mocker/validation.ts - validation error body
         */
        it('should preserve validation error fields', () => {
            const routes = manager.resolveCollection('error-responses');
            const route = routes.find(r => r.route.id === 'users-post');

            const body = route?.variant.body as {
                error: string;
                fields: Array<{ name: string } | string>;
            };

            expect(body.error).toBe('Validation failed');
            expect(body.fields).toBeDefined();
            expect(body.fields.length).toBeGreaterThan(0);
        });
    });

    describe('special body structures', () => {
        /**
         * Tests auth response with token.
         * @see example/mocker/validation.ts - auth response body
         */
        it('should preserve auth token response', () => {
            const routes = manager.resolveCollection('auth-success');
            const route = routes.find(r => r.route.id === 'auth-api');

            const body = route?.variant.body as {
                token: string;
                expiresIn: number;
            };

            expect(body.token).toBe('jwt-token-here');
            expect(body.expiresIn).toBe(3600);
        });

        /**
         * Tests file upload response.
         * @see example/mocker/validation.ts - file response body
         */
        it('should preserve file upload response', () => {
            const routes = manager.resolveCollection('file-upload');
            const route = routes.find(r => r.route.id === 'file-upload');

            const body = route?.variant.body as {
                fileId: string;
                filename: string;
                size: number;
                url: string;
            };

            expect(body.fileId).toBe('file-123');
            expect(body.filename).toBe('uploaded.pdf');
            expect(body.size).toBe(1024);
            expect(body.url).toBe('/files/file-123');
        });

        /**
         * Tests search results with pagination.
         * @see example/mocker/validation.ts - paginated response body
         */
        it('should preserve search results with pagination', () => {
            const routes = manager.resolveCollection('search');
            const route = routes.find(r => r.route.id === 'search-api');

            const body = route?.variant.body as {
                query: string;
                page: number;
                limit: number;
                results: Array<{ id: number; title: string }>;
                total: number;
            };

            expect(body.query).toBe('test');
            expect(body.page).toBe(1);
            expect(body.limit).toBe(10);
            expect(body.results).toHaveLength(2);
            expect(body.total).toBe(100);
        });

        /**
         * Tests health check response.
         * @see example/mocker/validation.ts - health response body
         */
        it('should preserve health check response', () => {
            const routes = manager.resolveCollection('health');
            const route = routes.find(r => r.route.id === 'health-check');

            const body = route?.variant.body as {
                status: string;
                timestamp: string;
            };

            expect(body.status).toBe('ok');
            expect(body.timestamp).toBe('2024-01-01T00:00:00Z');
        });
    });

    describe('empty body', () => {
        /**
         * Tests response without body (204 No Content).
         * @see example/mocker/validation.ts - optional body field
         */
        it('should handle response without body', () => {
            const routes = manager.resolveCollection('crud-operations');
            const route = routes.find(r => r.route.id === 'users-delete');

            // 204 responses typically don't have body
            expect(route?.variant.status).toBe(204);
        });
    });

    describe('empty array/object body', () => {
        /**
         * Tests empty results array.
         * @see example/mocker/validation.ts - empty array body
         */
        it('should preserve empty array', () => {
            const routes = manager.resolveCollection('search-empty');
            const route = routes.find(r => r.route.id === 'search-api');

            const body = route?.variant.body as {
                results: unknown[];
                total: number;
            };

            expect(body.results).toEqual([]);
            expect(body.total).toBe(0);
        });

        /**
         * Tests empty list response.
         * @see example/mocker/validation.ts - empty data body
         */
        it('should preserve empty data list', () => {
            const routes = manager.resolveCollection('override-parent');
            const route = routes.find(r => r.route.id === 'users-api');

            const body = route?.variant.body as {
                data: unknown[];
                total: number;
            };

            expect(body.data).toEqual([]);
            expect(body.total).toBe(0);
        });
    });
});
