/**
 * WebSocket Routes Tests
 *
 * Tests for WebSocket transport support.
 *
 * @see example/mocker/consts.ts - WEBSOCKET_METHOD = 'WS'
 * @see example/mocker/__specs__/controller.spec.ts - WebSocket tests
 */
import path from 'node:path';
import {describe, expect, it, beforeAll} from '@rstest/core';
import {MocksManager, MocksController, Transport} from '@mockito/binding';

const FIXTURES_PATH = path.join(import.meta.dirname, '..', '__fixtures__');
const COLLECTIONS_PATH = path.join(FIXTURES_PATH, 'collections.yaml');
const ROUTES_PATH = path.join(FIXTURES_PATH, 'routes', '*.yaml');

describe('WebSocket Routes', () => {
    let manager: MocksManager;

    beforeAll(() => {
        manager = new MocksManager(COLLECTIONS_PATH, ROUTES_PATH);
    });

    describe('transport type', () => {
        /**
         * Tests WebSocket transport detection.
         * @see example/mocker/consts.ts - WEBSOCKET_METHOD = 'WS'
         */
        it('should identify WebSocket transport', () => {
            const routes = manager.resolveCollection('with-websocket');
            const wsRoute = routes[0];

            expect(wsRoute?.route.transport).toBe(Transport.WebSocket);
        });

        /**
         * Tests that HTTP routes have HTTP transport.
         * @see example/mocker/consts.ts - HTTP vs WS transport
         */
        it('should distinguish WebSocket from HTTP transport', () => {
            const wsRoutes = manager.resolveCollection('with-websocket');
            const httpRoutes = manager.resolveCollection('base');

            expect(wsRoutes[0]?.route.transport).toBe(Transport.WebSocket);
            expect(httpRoutes[0]?.route.transport).toBe(Transport.Http);
        });
    });

    describe('WebSocket URL', () => {
        /**
         * Tests WebSocket URL parsing.
         * @see example/mocker/validation.ts - url field
         */
        it('should parse WebSocket URL path', () => {
            const routes = manager.resolveCollection('with-websocket');
            const wsRoute = routes[0];

            expect(wsRoute?.route.url).toBe('/ws/notifications');
        });

        /**
         * Tests chat WebSocket URL.
         * @see example/mocker/validation.ts - url field
         */
        it('should parse chat WebSocket URL', () => {
            const routes = manager.resolveCollection('ws-chat');
            const wsRoute = routes[0];

            expect(wsRoute?.route.url).toBe('/ws/chat');
        });
    });

    describe('WebSocket body', () => {
        /**
         * Tests WebSocket message body.
         * @see example/mocker/validation.ts - body in Variant
         */
        it('should parse notification message body', () => {
            const routes = manager.resolveCollection('with-websocket');
            const wsRoute = routes[0];

            const body = wsRoute?.variant.body as {
                type: string;
                payload: { title: string; content: string };
            };

            expect(body.type).toBe('notification');
            expect(body.payload.title).toBe('New message');
            expect(body.payload.content).toBe('Hello, world!');
        });

        /**
         * Tests chat connected message.
         * @see example/mocker/validation.ts - body in Variant
         */
        it('should parse chat connected message', () => {
            const routes = manager.resolveCollection('ws-chat');
            const wsRoute = routes[0];

            const body = wsRoute?.variant.body as {
                type: string;
                userId: string;
                room: string;
            };

            expect(body.type).toBe('connected');
            expect(body.userId).toBe('user-123');
            expect(body.room).toBe('general');
        });

        /**
         * Tests chat message.
         * @see example/mocker/validation.ts - body in Variant
         */
        it('should parse chat message', () => {
            const routes = manager.resolveCollection('ws-chat-message');
            const wsRoute = routes[0];

            const body = wsRoute?.variant.body as {
                type: string;
                from: string;
                text: string;
                timestamp: string;
            };

            expect(body.type).toBe('message');
            expect(body.from).toBe('user-456');
            expect(body.text).toBe('Hello, World!');
        });
    });

    describe('WebSocket with MocksController', () => {
        /**
         * Tests WebSocket routes via MocksController.
         * @see example/mocker/__specs__/controller.spec.ts - "should pass parser url for websocket route"
         */
        it('should get WebSocket routes via controller', () => {
            const controller = new MocksController(COLLECTIONS_PATH, ROUTES_PATH);
            controller.useCollection('with-websocket');

            const routes = controller.getActiveRoutes();

            expect(routes).toHaveLength(1);
            expect(routes[0]?.route.transport).toBe(Transport.WebSocket);
        });

        /**
         * Tests switching to WebSocket collection.
         * @see example/mocker/controller.ts - useCollection
         */
        it('should switch to WebSocket collection', () => {
            const controller = new MocksController(COLLECTIONS_PATH, ROUTES_PATH);

            controller.useCollection('base');
            expect(controller.getActiveRoutes()[0]?.route.transport).toBe(Transport.Http);

            controller.useCollection('with-websocket');
            expect(controller.getActiveRoutes()[0]?.route.transport).toBe(Transport.WebSocket);
        });
    });

    describe('WebSocket method field', () => {
        /**
         * Tests that WebSocket routes don't have HTTP method.
         * @see example/mocker/consts.ts - WS routes don't use HTTP method
         */
        it('should not have HTTP method for WebSocket route', () => {
            const routes = manager.resolveCollection('with-websocket');
            const wsRoute = routes[0];

            // WebSocket routes typically don't have an HTTP method
            // or it's undefined
            expect(wsRoute?.route.method).toBeUndefined();
        });
    });

    describe('multiple WebSocket variants', () => {
        /**
         * Tests switching between WebSocket variants via collections.
         * @see example/mocker/controller.ts - variant selection
         */
        it('should support different WebSocket variants via collections', () => {
            const connectedRoutes = manager.resolveCollection('ws-chat');
            const messageRoutes = manager.resolveCollection('ws-chat-message');

            const connectedBody = connectedRoutes[0]?.variant.body as { type: string };
            const messageBody = messageRoutes[0]?.variant.body as { type: string };

            expect(connectedBody.type).toBe('connected');
            expect(messageBody.type).toBe('message');
        });
    });
});
