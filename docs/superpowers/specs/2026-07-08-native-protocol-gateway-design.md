# Native Protocol Gateway Design

## Purpose

Build the first compileable Rust gateway skeleton from `docs/核心功能.md`.

The first version proves the architecture boundary, not real upstream LLM calls. It exposes one non-streaming primary route for each native protocol, routes by model through a small core layer, and returns mock responses in the same protocol shape used by the client.

## Scope

Implemented in the first slice:

- `GET /healthz`
- `POST /v1/chat/completions`
- `POST /v1/messages`
- `POST /v1beta/models/{model}:generateContent`
- static model routing
- mock provider facade
- protocol-shaped mock responses
- structured gateway errors
- workspace tests and compile checks

Out of scope for the first slice:

- real upstream OpenAI, Anthropic, or Gemini HTTP calls
- embeddings
- OpenAI Responses API
- token counting
- Anthropic batches
- streaming responses
- cross-protocol provider conversion
- API key management, tenant policy, rate limiting, retry, circuit breaking, metrics, billing, guardrails, RAG, and Wasm harness work
- official Go SDK compatibility harness

## Architecture

Use a Rust workspace with protocol crates split by native API family:

```text
ai-gateway/
├── Cargo.toml
└── crates/
    ├── gateway/
    ├── gateway-core/
    └── protocol/
        ├── openai/
        ├── anthropic/
        └── gemini/
```

Crate responsibilities:

- `crates/gateway`: Axum binary, route registration, shared app state, TCP listener, tracing setup, and `/healthz`.
- `crates/gateway-core`: common routing and gateway concepts only. It owns `Protocol`, `RequestContext`, `RouteRequest`, `RouteDecision`, `GatewayError`, a static router, and a mock provider facade.
- `crates/protocol/openai`: OpenAI Chat Completions request and response DTOs plus the OpenAI handler.
- `crates/protocol/anthropic`: Anthropic Messages request and response DTOs plus the Anthropic handler.
- `crates/protocol/gemini`: Gemini `generateContent` request and response DTOs plus the Gemini handler.

Dependency direction:

```text
gateway ─────────────► protocol/openai
   │                  ► protocol/anthropic
   │                  ► protocol/gemini
   │
   └──────────────────► gateway-core

protocol/* ──────────► gateway-core

gateway-core does not depend on protocol crates.
```

This intentionally avoids a `gateway-protocol` crate and avoids a global Canonical Request IR. Protocol-specific DTOs stay in their own crates.

## Request Flow

Each protocol route follows the same control flow while preserving protocol-specific types:

1. Axum receives a JSON request.
2. The relevant protocol crate deserializes the request into its native DTO.
3. The handler extracts minimal routing metadata: `Protocol`, requested model, stream flag, and request id.
4. The handler calls `gateway-core` routing with `RouteRequest`.
5. The static router returns `RouteDecision`, including provider id, upstream model, and timeout.
6. The mock provider facade returns `MockCompletion`.
7. The protocol crate maps `MockCompletion` back into its own native response DTO.
8. Axum serializes the protocol response.

`gateway-core` must not inspect messages, content blocks, tools, multimodal fields, or protocol-specific options. Those stay owned by protocol crates.

## Core Types

`gateway-core` should provide small stable types:

```rust
pub enum Protocol {
    OpenAi,
    Anthropic,
    Gemini,
}

pub struct RequestContext {
    pub request_id: String,
}

pub struct RouteRequest<'a> {
    pub protocol: Protocol,
    pub requested_model: &'a str,
    pub streaming: bool,
}

pub struct RouteDecision {
    pub provider: ProviderId,
    pub upstream_model: String,
    pub timeout: Duration,
}
```

The first static router should accept a small deterministic set of model names for tests and local use, such as:

- `gpt-4o-mini` routed to mock OpenAI
- `claude-3-5-haiku-latest` routed to mock Anthropic
- `gemini-1.5-flash` routed to mock Gemini

The exact list can be small, but unknown models must produce a routing error.

## Protocol DTOs

DTOs should be minimal but shaped like the native APIs enough for SDK-facing growth:

- OpenAI Chat Completions: `model`, `messages`, optional `stream`.
- Anthropic Messages: `model`, `messages`, optional `stream`, optional `max_tokens`.
- Gemini Generate Content: model from path, `contents` in body.

Use `serde_json::Value` for nested content that is not needed by the first slice. This keeps the skeleton compileable without prematurely modeling every vendor field.

## Error Handling

Errors should be structured in core and mapped at the HTTP boundary:

- JSON extraction errors: `400 Bad Request`.
- missing required route metadata, such as model: `400 Bad Request`.
- unsupported streaming request: `501 Not Implemented`.
- unknown model: `404 Not Found`.
- mock provider failure: `502 Bad Gateway`.
- unexpected gateway failure: `500 Internal Server Error`.

Protocol handlers should return error bodies that are stable and close to the protocol family, but the first slice does not need full vendor-identical error envelopes.

Only the four approved routes should be mounted in the first slice. Other documented APIs should remain unmounted rather than pretending to be compatible.

## Testing

Required verification commands:

```sh
cargo fmt --all
cargo check --workspace --locked
cargo test --workspace --locked
```

Test coverage:

- `gateway-core` unit tests for static routing, unknown model errors, protocol enum behavior, and streaming rejection.
- protocol crate unit tests for minimal request parsing and mapping `MockCompletion` into the protocol response DTO.
- `gateway` integration tests using the Axum router directly without binding a TCP port.

Acceptance criteria:

- the workspace compiles
- all tests pass
- `/healthz` returns a healthy response
- each of the three primary protocol routes accepts a minimal JSON request and returns a mock response
- `stream: true` returns `501`
- unknown model returns `404`

## Future Work

After this skeleton is in place, later designs can add:

- real provider adapters under `crates/provider/*`
- explicit cross-protocol mapping modules
- streaming support per protocol
- the OpenAI Responses API
- embeddings and token counting
- model registry configuration
- tenant-aware routing and policy
- official Go SDK compatibility tests
- Wasm-based harness work described in `docs/需求.md`
