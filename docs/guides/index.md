# Guides

Task- and surface-oriented documentation for cow-rs.

* [Alloy Doctrine](alloy-doctrine.md) - Pre-1.0; binding for the v1.0 cut and forward.
* [Alloy Major-Release Absorption Runbook](alloy-major-release-runbook.md) - This runbook supplements ADR 0026 with the operational procedure for absorbing a major-version Alloy release into the cow-rs workspace.
* [Architecture](architecture.md) - cow-rs is a small family of focused crates.
* [cow-rs and the TypeScript SDK](comparison-with-typescript-sdk.md) - The canonical SDK for CoW Protocol is @cowprotocol/cow-sdk, a TypeScript monorepo maintained by the protocol team.
* [Deployments And The Registry](deployments.md) - This page explains how cow-rs resolves deployed contract addresses.
* [Examples](examples.md) - The examples are organized by user goal rather than by crate internals.
* [Getting Started](getting-started.md) - cow-rs is a trading-first Rust SDK for CoW Protocol.
* [Integrations](integrations.md) - This guide explains how native runtime adapters plug into the public cow-rs surface.
* [MSRV Policy](msrv-policy.md) - This workspace declares Rust 1.94.0 as its minimum supported Rust version for the published cow-sdk crate family.
* [Observability](observability.md) - The cow-rs SDK family ships an opt-in tracing feature so host applications can route structured spans and events from the SDK into their own subscriber without paying any dependency or runtime cost when the feature is disabled.
* [Parity And Provenance](parity.md) - This document defines the parity authorities for cow-rs, the committed source-lock contract that pins them, the surface-to-evidence map, and the in-scope and out-of-scope boundaries for the release.
* [Performance Posture](performance.md) - This document maps the performance-sensitive surfaces of the cow-rs SDK family and records the benchmark coverage that protects them against regressions.
* [Publication Handoff](publication-handoff.md) - This document describes how publication ownership for the cow-rs crates on crates.io is managed: how invitations are issued, how maintainers are rotated on and off the owner list, and how a broken release is retracted.
* [Release Checklist](release-checklist.md) - Use this checklist before tagging or publishing a release that changes the public cow-rs surface.
* [Transport](transport.md) - This page explains how cow-rs dispatches HTTP requests to the CoW Protocol orderbook and the subgraph, how to choose a transport on native and browser targets, and how to plug in a custom transport implementation for tests, bridging, or bes...
* [Verification](verification.md) - Use this guide to understand how cow-rs justifies its public behavior and where the current executable evidence lives.
