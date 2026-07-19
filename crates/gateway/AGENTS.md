# 网关架构约束

## 调用链

- 请求必须遵循 `HTTP Request -> Axum Handler -> GatewayService -> ProviderService` 的单向调用链。
- 禁止跨层调用：Axum Handler 不得直接调用 ProviderService，ProviderService 也不得反向依赖 GatewayService 或 HTTP Layer。

## 分层职责

- HTTP Layer 只负责 HTTP 协议相关工作，包括路由、请求提取、基础校验、状态码和响应转换。
- Axum Handler 应保持轻量，只完成 HTTP 输入到 GatewayService 调用的适配，不承载网关编排或供应商实现。
- Gateway Layer 负责网关业务编排；GatewayService 是 Handler 与 ProviderService 之间的唯一业务入口。
- Provider Layer 负责供应商特有的协议、请求和响应适配；ProviderService 不得依赖 Axum 的 Handler 或路由类型。

## 横切能力

- HTTP 通用能力应实现为 HTTP Layer，例如认证、请求追踪、超时和 HTTP 日志。
- 网关通用能力应实现为 Gateway Layer，例如路由选择、重试、熔断和限流。
- 供应商通用能力应实现为 Provider Layer，例如供应商鉴权、协议转换和上游错误映射。
- 横切能力应放在对应层的边界，避免混入 Handler 或核心业务流程。

## 变更要求

- 新增功能时先确定所属层，并保持依赖方向从 HTTP Layer 指向 Gateway Layer，再指向 Provider Layer。
- 如果实现需要绕过既定调用链，应先调整架构设计和本文约束，不得直接引入跨层依赖。

## 错误处理

- 不允许软件崩溃，出错需要用response返回