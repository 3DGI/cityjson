# Template / Instance Import Note

## What The Technical Question Really Means

The unresolved question is:

When `serde_cityjson` has parsed CityJSON JSON that contains:

- `geometry-templates.vertices-templates`
- `geometry-templates.templates`
- `GeometryInstance.template`
- `GeometryInstance.transformationMatrix`

how should it turn that into a valid `cityjson-rs` model without reimplementing `cityjson-rs` internal storage and validation rules?

## Why This Is Tricky

`cityjson-rs` owns model invariants such as:

- template geometries must reference template vertices, not regular vertices
- regular geometries must reference regular vertices
- `GeometryInstance` must reference a valid template handle
- materials, textures, and semantics must match the geometry topology
- inserted geometry must pass the crate’s validation rules

Those are model concerns, not JSON concerns.

But `serde_cityjson` is the crate that sees the JSON first.
So after parsing, it must still decide:

- this parsed geometry is a regular geometry
- this parsed geometry is a template geometry
- this parsed geometry is a geometry instance

and then it must insert those correctly into `cityjson-rs`.

## What We Want To Avoid

We do not want `serde_cityjson` or `cjlib` to duplicate model invariant logic by:

- rebuilding internal geometry structures by hand
- reimplementing root-vertex vs template-vertex rules
- reimplementing geometry-instance validity rules
- reimplementing topology validation logic

That would spread correctness-critical behavior across multiple crates.

## What “Small Import-Oriented APIs” Means

This does **not** mean adding a second large public indexed-geometry API for users.

It means giving `serde_cityjson` a narrow, validated way to say:

- insert this parsed geometry as regular geometry
- insert this parsed geometry as template geometry
- insert this parsed geometry instance referencing template X

while letting `cityjson-rs` remain the place that enforces the actual model invariants.

## Best-Case Outcome

Best case:

- the current public `cityjson-rs` APIs are already sufficient
- `serde_cityjson` can use them directly
- no new API is needed

## Second-Best Outcome

If the current APIs are not quite enough, then `cityjson-rs` should add only a very small amount
of import-oriented support so `serde_cityjson` can reach the correct insertion path for:

- template geometries
- geometry instances

without reconstructing internals manually.

## What The Question Is Really Asking

In plain language:

How do we let `serde_cityjson` tell `cityjson-rs` “this is a template geometry” or “this is a
geometry instance” in a way that stays validated and does not duplicate model logic?

## Desired End State

- `serde_cityjson` owns JSON parsing and JSON-boundary interpretation
- `cityjson-rs` owns model insertion rules and validation
- `cjlib` stays out of template / instance storage details

That keeps the architecture clean:

`json <--> serde_cityjson <--> cityjson-rs <--> cjlib`
