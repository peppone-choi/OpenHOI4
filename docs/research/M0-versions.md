# M0 버전 재확인

확인일: 2026-10-06 (Asia/Seoul). 공식 레지스트리 응답의 stable/latest와 license를 확인했다. 미결정 D-10을 확정하지 않으며 허용 목록은 02 §14.3을 유지한다.

| 생태계 | 패키지 | 버전 | 라이선스 | 출처 |
|---|---|---|---|---|
| cargo | `tokio` | 1.53.2 | MIT | [공식 API](https://crates.io/api/v1/crates/tokio) |
| cargo | `axum` | 0.8.9 | MIT | [공식 API](https://crates.io/api/v1/crates/axum) |
| cargo | `rmp-serde` | 1.3.1 | MIT | [공식 API](https://crates.io/api/v1/crates/rmp-serde) |
| cargo | `ts-rs` | 12.0.1 | MIT | [공식 API](https://crates.io/api/v1/crates/ts-rs) |
| cargo | `fixed` | 1.31.0 | MIT/Apache-2.0 | [공식 API](https://crates.io/api/v1/crates/fixed) |
| cargo | `rand_chacha` | 0.10.0 | MIT OR Apache-2.0 | [공식 API](https://crates.io/api/v1/crates/rand_chacha) |
| cargo | `serde` | 1.0.229 | MIT OR Apache-2.0 | [공식 API](https://crates.io/api/v1/crates/serde) |
| cargo | `postcard` | 1.1.3 | MIT OR Apache-2.0 | [공식 API](https://crates.io/api/v1/crates/postcard) |
| cargo | `toml` | 1.1.6+spec-1.1.0 | MIT OR Apache-2.0 | [공식 API](https://crates.io/api/v1/crates/toml) |
| cargo | `schemars` | 1.2.2 | MIT | [공식 API](https://crates.io/api/v1/crates/schemars) |
| cargo | `zstd` | 0.14.0 | BSD-3-Clause | [공식 API](https://crates.io/api/v1/crates/zstd) |
| cargo | `proptest` | 1.11.0 | MIT OR Apache-2.0 | [공식 API](https://crates.io/api/v1/crates/proptest) |
| cargo | `cargo-deny` | 0.20.2 | MIT OR Apache-2.0 | [공식 API](https://crates.io/api/v1/crates/cargo-deny) |
| npm | `typescript` | 7.0.2 | Apache-2.0 | [공식 API](https://registry.npmjs.org/typescript/latest) |
| npm | `vite` | 8.3.3 | MIT | [공식 API](https://registry.npmjs.org/vite/latest) |
| npm | `react` | 19.3.0 | MIT | [공식 API](https://registry.npmjs.org/react/latest) |
| npm | `three` | 0.186.1 | MIT | [공식 API](https://registry.npmjs.org/three/latest) |
| npm | `@msgpack/msgpack` | 3.1.3 | ISC | [공식 API](https://registry.npmjs.org/%40msgpack%2Fmsgpack/latest) |
| npm | `@fluent/bundle` | 0.19.1 | Apache-2.0 | [공식 API](https://registry.npmjs.org/%40fluent%2Fbundle/latest) |
| npm | `vitest` | 5.0.3 | MIT | [공식 API](https://registry.npmjs.org/vitest/latest) |
| npm | `@playwright/test` | 1.63.0 | Apache-2.0 | [공식 API](https://registry.npmjs.org/%40playwright%2Ftest/latest) |

Rust stable: 1.99.0, 2026-10-01. [공식 발표](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/).

추가 구현 도구·직접 의존성을 채택하면 구현 세션이 공식 레지스트리에서 버전·라이선스를 다시 확인하고 ADR과 lockfile에 기록한다. 최신 조회 자체가 허용 목록 검사를 대체하지 않는다.
