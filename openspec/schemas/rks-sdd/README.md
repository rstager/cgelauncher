# Spec-Driven Schema (Project Override)

This repository includes a project-local OpenSpec schema override at `openspec/schemas/spec-driven`.
It extends the default workflow with explicit data model, API contract, and testing traceability artifacts.

## Workflow

Artifact sequence:

1. `proposal` → `proposal.md`
2. `specs` → `specs/<capability>/spec.md`
3. `design` → `design.md`
4. `datamodels` → `datamodels.yaml`
5. `apis` → `apis.yaml`
6. `testing` → `testing.md`
7. `tasks` → `tasks.md`

Dependency gates enforce completion order (for example, `tasks` depends on `testing`, `datamodels`, and `apis`).

## Artifact Roles

- `proposal.md`: Why this change is needed and what capabilities are affected.
- `specs/.../spec.md`: Normative requirements and scenarios (WHAT).
- `design.md`: Human-readable architecture and implementation rationale (HOW).
- `datamodels.yaml`: Authoritative programmatic data model definitions as JSON Schema (YAML syntax).
- `apis.yaml`: Authoritative API contract in OpenAPI-compatible format.
- `testing.md`: Source of truth for automated test implementation, including explicit test cases and traceability.
- `tasks.md`: Actionable implementation checklist.

## Key Conventions

### Design vs Programmatic Sources

- `design.md` explains intent, purpose, and architecture in human-readable form.
- `datamodels.yaml` and `apis.yaml` hold authoritative machine-readable definitions.

### Data Models

- `datamodels.yaml` uses JSON Schema draft 2020-12 in YAML form.
- Keep schema names aligned with API component schema names when applicable.

### APIs

- `apis.yaml` must be OpenAPI 3.x compatible.
- Preferred: reusable payloads in `components.schemas` referenced via `$ref`.
- Exception: inline schemas only for one-off, non-reused payloads.

### Testing

- The project maintains a full automated test suite.
- Test cases in `testing.md` must include: ID, name, purpose, type, preconditions, steps/input, expected results, and priority.
- Include a bidirectional traceability matrix between spec requirements/scenarios and test case IDs.

## Validate the Schema

From repository root:

```bash
openspec schema validate spec-driven
```

## Verify Template Resolution

```bash
openspec templates --json
```

The `source` for all artifacts should resolve to `project`.
