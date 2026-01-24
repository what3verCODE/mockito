# Feature Comparison: example/mocker (JS) vs @mockito/binding (Rust)

This document tracks feature parity between the original JavaScript implementation and the new Rust implementation.

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and tested |
| 🚧 | Partially implemented |
| ❌ | Not implemented yet |
| ➖ | Not applicable / Internal |

---

## Controller Tests

Source: `example/mocker/__specs__/controller.spec.ts`

| # | Original Test | Status | E2E Test | Notes |
|---|---------------|--------|----------|-------|
| 1 | should mock selected collection | ✅ | `mocks-controller.spec.ts` - "should return active routes after selecting collection" | |
| 2 | should throw if no collection found | ✅ | `mocks-controller.spec.ts` - "should throw error for non-existent collection" | |
| 3 | should throw if collection have duplicated routes | ❌ | TODO: "should throw error for duplicated routes in collection" | |
| 4 | should mock selected collection with nesting | ✅ | `mocks-controller.spec.ts` - "should return routes with inheritance resolved" | |
| 5 | should throw if collection with nesting not found | ❌ | TODO: "should throw error for non-existent parent collection" | |
| 6 | should throw if route in collection not found | ❌ | TODO: "should throw error if route in collection not found" | |
| 7 | should throw if preset in collection not found | ❌ | TODO: "should throw error if preset in collection not found" | |
| 8 | should throw if variant in collection not found | ❌ | TODO: "should throw error if variant in collection not found" | |
| 9 | should mock selected route for default collection | ❌ | TODO: "should allow switching individual routes (useRoutes)" | Requires `useRoutes` method |
| 10 | should throw if route not found | ❌ | TODO | Requires `useRoutes` method |
| 11 | should throw if route is a websocket route | ❌ | TODO | Requires `useRoutes` method |
| 12 | should throw if preset for route not found | ❌ | TODO | Requires `useRoutes` method |
| 13 | should throw if variant for route not found | ❌ | TODO | Requires `useRoutes` method |
| 14 | should restore mock to selected collection | ❌ | TODO: "should support resetRoutes" | Requires `resetRoutes` method |
| 15 | should restore mock to nothing if no collection selected | ❌ | TODO | Requires `resetRoutes` method |
| 16 | should throw if websocket route not found | ❌ | TODO | Requires `useSocket` method |
| 17 | should throw if route is not a websocket route | ❌ | TODO | Requires `useSocket` method |
| 18 | should throw if preset for websocket route not found | ❌ | TODO | Requires `useSocket` method |
| 19 | should throw if variant for websocket route not found | ❌ | TODO | Requires `useSocket` method |
| 20 | should pass parser url for websocket route | 🚧 | `mocks-controller.spec.ts` - "should return WebSocket routes" | Parser field not tested |

---

## Mocks Tests

Source: `example/mocker/__specs__/mocks.spec.ts`

| # | Original Test | Status | E2E Test | Notes |
|---|---------------|--------|----------|-------|
| 1 | should load definitions | ✅ | `MocksManager` constructor loads files | Constructor behavior |
| 2 | should throw if more than 1 collection file | ➖ | N/A | Rust uses glob, may differ |
| 3 | should throw if no collection found | ✅ | `edge-cases.spec.ts` - "should throw error for non-existent collections file" | |

---

## Loader Tests

Source: `example/mocker/__specs__/loader.spec.ts`

| # | Original Test | Status | E2E Test | Notes |
|---|---------------|--------|----------|-------|
| 1 | should load file based on data from ctor | ✅ | `mocks-manager.spec.ts` - body content tests | YAML parsing works |
| 2 | should read only yaml files | ➖ | N/A | Rust may accept JSON too |

---

## Validation Tests

Source: `example/mocker/__specs__/validation.spec.ts`

| # | Original Test | Status | E2E Test | Notes |
|---|---------------|--------|----------|-------|
| 1 | should successfully validate collection | ✅ | Collections load successfully | |
| 2 | should successfully validate routes | ✅ | Routes load successfully | |
| 3 | should not accept collection without id | 🚧 | Not explicitly tested | Rust validation |
| 4 | should not accept collection without routes | 🚧 | Not explicitly tested | Rust validation |
| 5 | should not accept empty collection | 🚧 | Not explicitly tested | Rust validation |
| 6 | should not accept route without id | 🚧 | Not explicitly tested | Rust validation |
| 7 | should not accept route without url | 🚧 | Not explicitly tested | Rust validation |
| 8 | should not accept route with empty presets | 🚧 | Not explicitly tested | Rust validation |
| 9 | should not accept route with empty variants | 🚧 | Not explicitly tested | Rust validation |
| 10 | should not accept route with incorrect http method | 🚧 | Not explicitly tested | Rust validation |
| 11 | should not accept route with empty headers | 🚧 | Not explicitly tested | Rust validation |
| 12 | should not accept route with empty params | 🚧 | Not explicitly tested | Rust validation |
| 13 | should not accept route with empty query | 🚧 | Not explicitly tested | Rust validation |
| 14 | should not accept route with empty payload | 🚧 | Not explicitly tested | Rust validation |
| 15 | should not accept route with empty response headers | 🚧 | Not explicitly tested | Rust validation |
| 16 | should successfully validate websocket route | ✅ | `mocks-controller.spec.ts` - WebSocket tests | |
| 17 | should successfully validate parser field (ts) | ➖ | N/A | Parser is JS-specific |
| 18 | should successfully validate parser field (js) | ➖ | N/A | Parser is JS-specific |
| 19 | should fail to validate parser field (without encode) | ➖ | N/A | Parser is JS-specific |
| 20 | should fail to validate parser field (without decode) | ➖ | N/A | Parser is JS-specific |
| 21 | should concat path from context and file path from config | ➖ | N/A | Parser is JS-specific |

---

## Library Utility Tests

### route-meta.ts

Source: `example/mocker/lib/__specs__/route-meta.spec.ts`

| # | Original Test | Status | E2E Test | Notes |
|---|---------------|--------|----------|-------|
| 1 | should split route into meta object | ➖ | Internal | Route string parsing: `route:preset:variant` |
| 2 | should throw if route incorrect | ➖ | Internal | Error handling for malformed routes |
| 3 | should create route identifier based on meta | ➖ | Internal | Creates `route:preset` identifier |

### object-intersects.ts

Source: `example/mocker/lib/__specs__/object-intersects.spec.ts`

| # | Original Test | Status | E2E Test | Notes |
|---|---------------|--------|----------|-------|
| 1 | should return false if no keys found | ❌ | TODO | Request matching |
| 2 | should return false if intersected keys have different values | ❌ | TODO | Request matching |
| 3 | should return true if intersected keys have same values | ❌ | TODO | Request matching |

### url-matches.ts

Source: `example/mocker/lib/__specs__/url-matches.spec.ts`

| # | Original Test | Status | E2E Test | Notes |
|---|---------------|--------|----------|-------|
| 1 | should detect if url string is template | ❌ | TODO | URL path parameters |
| 2 | should fill url template with parameters | ❌ | TODO | URL path parameters |

### get-single-expression.ts

Source: `example/mocker/lib/__specs__/get-single-expression.spec.ts`

| # | Original Test | Status | E2E Test | Notes |
|---|---------------|--------|----------|-------|
| 1 | should return expression for simple template | ❌ | TODO | Expression parsing `${foo}` |
| 2 | should return undefined for template with surrounding text | ❌ | TODO | Expression parsing |
| 3 | should return undefined for multiple expressions | ❌ | TODO | Expression parsing |
| 4 | should return undefined for escaped expression | ❌ | TODO | Expression parsing |
| 5 | should return undefined for plain text | ❌ | TODO | Expression parsing |

### compile-template.ts

Source: `example/mocker/lib/__specs__/compile-template.spec.ts`

| # | Original Test | Status | E2E Test | Notes |
|---|---------------|--------|----------|-------|
| 1 | jmespath: should preserve string/number/object/array/boolean/null types | ❌ | TODO | JMESPath support |
| 2 | jmespath: nested value, array element access | ❌ | TODO | JMESPath support |
| 3 | string interpolation: interpolate string values | ❌ | TODO | Template interpolation |
| 4 | string interpolation: handle nested object templates | ❌ | TODO | Template interpolation |
| 5 | array templates: should compile templates inside arrays | ❌ | TODO | Template interpolation |
| 6 | object templates: should handle multiple expressions | ❌ | TODO | Template interpolation |
| 7 | function calls: sum, to_string, length | ❌ | TODO | JMESPath functions |
| 8 | function calls: complex filter/project/sort/join | ❌ | TODO | JMESPath functions |
| 9 | escaped expressions: should not evaluate escaped expressions | ❌ | TODO | `\${name}` handling |
| 10 | context validation: should throw error when context is not JSON | ❌ | TODO | Error handling |
| 11 | null result: should return empty string for null | ❌ | TODO | Null handling |

---

## Summary

### E2E Test Statistics

```
Test Files: 9 passed
Tests:      147 passed | 14 todo (161 total)
Duration:   ~200ms
```

### Test Files

| File | Tests | Description |
|------|-------|-------------|
| `mocks-controller.spec.ts` | 34 | Controller API, collection switching |
| `mocks-manager.spec.ts` | 19 | Manager API, collection resolution |
| `http-methods.spec.ts` | 22 | GET/POST/PUT/PATCH/DELETE, status codes |
| `url-patterns.spec.ts` | 18 | URL paths, parameters |
| `response-body.spec.ts` | 17 | Body structures, data types |
| `edge-cases.spec.ts` | 17 | Error handling, edge cases |
| `collection-inheritance.spec.ts` | 14 | Inheritance chains, overrides |
| `websocket.spec.ts` | 11 | WebSocket transport |
| `version.spec.ts` | 9 | API exports, enums |

### By Original Category

| Category | Total | ✅ Done | 🚧 Partial | ❌ TODO | ➖ N/A |
|----------|-------|---------|------------|--------|--------|
| Controller | 20 | 4 | 1 | 15 | 0 |
| Mocks | 3 | 2 | 0 | 0 | 1 |
| Loader | 2 | 1 | 0 | 0 | 1 |
| Validation | 21 | 2 | 13 | 0 | 6 |
| Lib: route-meta | 3 | 0 | 0 | 0 | 3 |
| Lib: object-intersects | 3 | 0 | 0 | 3 | 0 |
| Lib: url-matches | 2 | 0 | 0 | 2 | 0 |
| Lib: get-single-expression | 5 | 0 | 0 | 5 | 0 |
| Lib: compile-template | 11 | 0 | 0 | 11 | 0 |
| **Total** | **70** | **9** | **14** | **36** | **11** |

### Core Features Status

| Feature | Status | Notes |
|---------|--------|-------|
| Load collections from YAML | ✅ | Works with glob patterns |
| Load routes from YAML | ✅ | Works with glob patterns |
| Collection inheritance (`from`) | ✅ | Multi-level supported |
| Route override in child collection | ✅ | Child wins over parent |
| HTTP route handling | ✅ | GET, POST, etc. |
| WebSocket route handling | ✅ | Transport type detection |
| `useCollection` | ✅ | Switch active collection |
| `getActiveRoutes` | ✅ | Return resolved routes |
| `currentCollection` | ✅ | Getter for current ID |
| `useRoutes` | ❌ | Dynamic route switching |
| `useSocket` | ❌ | WebSocket route switching |
| `resetRoutes` | ❌ | Reset to collection state |
| `findRoute` | ❌ | Request matching |
| URL path parameters | ❌ | `/users/:id` support |
| Query matching | ❌ | `${query.page == '1'}` |
| Header matching | 🚧 | Parsed, not tested for matching |
| Payload matching | 🚧 | Parsed, not tested for matching |
| JMESPath expressions | ❌ | For request matching |
| Template interpolation | ❌ | `${request.id}` in responses |

---

## API Compatibility

### MocksController (Rust) vs Controller (JS)

```typescript
// JS: example/mocker/controller.ts
class Controller {
  useCollection(id: string): Promise<Controller>
  useRoutes(routes: string[]): Promise<Controller>
  useSocket(routes: string[]): Promise<Controller>
  resetRoutes(): Promise<Controller>
  getWebSocket(route: string): Promise<WebSocket>
  get state(): FlatRoute[]
}

// Rust: @mockito/binding
class MocksController {
  constructor(collectionsPath: string, routesPath: string, defaultCollection?: string)
  useCollection(collectionId: string): void              // ✅
  get currentCollection(): string | null                  // ✅
  getActiveRoutes(): Array<ActiveRoute>                   // ✅
  // Missing: useRoutes, useSocket, resetRoutes, getWebSocket
}
```

### MocksManager (Rust) - New API

```typescript
// Rust: @mockito/binding (stateless alternative)
class MocksManager {
  constructor(collectionsPath: string, routesPath: string)
  resolveCollection(collectionId: string): Array<ActiveRoute>  // ✅
}
```

---

## Migration Guide

To use `@mockito/binding` as drop-in replacement:

### Currently Supported

```typescript
// ✅ Creating controller with default collection
const controller = new MocksController(collectionsPath, routesPath, 'my-collection');

// ✅ Switching collections
controller.useCollection('another-collection');

// ✅ Getting active routes
const routes = controller.getActiveRoutes();

// ✅ Checking current collection
console.log(controller.currentCollection);
```

### Not Yet Supported

```typescript
// ❌ Dynamic route switching
await controller.useRoutes(['route:preset:variant']);

// ❌ WebSocket route switching
await controller.useSocket(['ws-route:preset:variant']);

// ❌ Reset to collection state
await controller.resetRoutes();

// ❌ Request matching (findRoute equivalent)
const matchedRoute = controller.findRoute(request);
```
