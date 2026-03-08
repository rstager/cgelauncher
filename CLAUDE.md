# Agent Operating Manual

This file defines how agents should execute work in this repository.

## Primary Model

- A primary controller agent owns planning, sequencing, validation, and final integration.
- The primary agent should delegate to subagents whenever work can be parallelized.
- Subagents should be scoped to one clear responsibility and return concise artifacts/results.

## Work Modes


## Mandatory Development Flow (Strict)

1. OpenSpec first: define or update spec artifacts before implementation.
2. Human checkpoint: do not start coding until spec intent is approved.
3. TDD first implementation path:
   - Write failing tests first.
   - Implement minimal code to pass.
   - Refactor while keeping suite green.
5. Structure-first scaffolding:
   - Start with function names, variable names, and docstrings/signatures.
   - Run explore/research subagents to validate naming and conventions.
6. Code generation after naming passes review.
7. Final validation:
   - full tests and quality gates green.


## Naming and Comments

Names must describe domain behavior, not implementation history.

### Naming rules

- Never use implementation details in names (for example: `ZodValidator`, `MCPWrapper`, `JSONParser`).
- Never use temporal names (for example: `NewAPI`, `LegacyHandler`, `UnifiedTool`).
- Avoid design-pattern words unless they add real clarity.

### Preferred style

- `Tool` over `AbstractToolInterface`
- `RemoteTool` over `MCPToolWrapper`
- `Registry` over `ToolRegistryManager`
- `execute()` over `executeToolWithValidation()`

### Comments

- Keep comments intent-focused.
- Prefer comments for non-obvious constraints and tradeoffs.
- Do not add comments that restate obvious code.

### Code style
- use assert/exception type handling for programming misuse of apis

## Testing Policy

- Every change should include comprehensive tests for real logic.
- Target near-100% coverage when practical for touched areas.
- No-exceptions default: unit + integration + end-to-end test coverage expectations apply.
- Do not mock the behavior under test.
- End-to-end tests should use real integrations/data paths where feasible.
- Treat test failures and suspicious logs as first-class defects to resolve.
- Maintain a full automated test suite for the product

## Tooling Quick Reference

### OpenSpec

- Purpose: spec-first planning and change control.
- Typical flow: `/opsx:new` -> `/opsx:ff` or `/opsx:continue` -> `/opsx:apply` -> `/opsx:archive`.
- use rks-sdd schema
- follow a test driven development methodology. Write the tests and the code in parallel agents and then iterate untill all tests pass.
- the tests should be driven by the spec
- the tests should not be modified to match the implementation



## Workspace Hygiene

- Use `scratchpad/` for temporary docs, one-off scripts, experiments, and throwaway artifacts.
- If needed for research/examples, add temporary git submodules only under `scratchpad/`.
- Web research is allowed via available MCP tools; capture useful findings in `scratchpad/` before implementation.
- Use `docs/` for durable human documentation after implementation/testing.
- Base docs on OpenSpec artifacts plus real code behavior.

