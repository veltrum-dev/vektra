# Versioning and release policy

Vektra uses a GitHub Project, Issues, and Milestones for the long-term roadmap, executable capabilities, and release commitments respectively. This policy describes what version numbers promise and how work enters a release.

## What `0.1.0` means

`0.1.0` is Vektra's first public release whose included functionality is complete, documented, and verified. Every component in scope must provide a complete and composable public API, with applicable states, keyboard and focus behavior, accessibility semantics, themes, documentation, examples, and deterministic tests.

The release must have no unresolved P0/P1 gaps in correctness, keyboard behavior, focus, accessibility, themes, or documentation. `0.1.0` does not require every future component on the roadmap; capabilities that are not release commitments remain tracked on the Roadmap.

## `0.x` and `1.0.0`

`0.x` means that public APIs can still change incompatibly as GPUI evolves. It does not mean low quality, incomplete components, or missing verification. Every released component must still meet the quality standard stated for its release.

`1.0.0` marks a commitment to long-term public API stability. It is not the first version where baseline quality is expected, nor does it mean breaking changes can never happen; later incompatible changes will follow the corresponding major-version rules.

## GPUI compatibility

GPUI is not yet stable, so Vektra pins a specific revision in the workspace root `Cargo.toml`. That revision is the source of truth for the current GPUI API and compatibility. Upgrading it can require coordinated changes to Vektra's implementation or public API, so consumers should use the same pinned revision for `gpui` and `gpui_platform`.

Every user-facing breaking change must be recorded explicitly in the changelog or release notes, including its migration impact and the available replacement.

## Responsibilities of planning objects

- **Vektra Roadmap Project** organizes long-term direction, priorities, target releases, and execution state across versions; it does not replace Issues.
- **Issues** record individual executable and verifiable capabilities or gaps, together with discussion, dependencies, and relationships.
- **Milestones** represent work committed to a specific release. They have no due date without a confirmed schedule, and a future idea does not enter a Milestone merely by appearing on the Roadmap.

Release-scope changes must remain traceable through Issues and Milestones. Unscheduled work stays in the Roadmap's Future/Unscheduled view rather than existing only in source TODOs or chat history.
