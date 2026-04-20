# Python Package Publishing

This repository publishes a Python package named `cityjson-index` that imports
the `cityjson_index` module.

The package depends on `cityjson-lib` at runtime. Treat that as a normal PyPI
dependency, not as a local checkout path. `cityjson-lib` is already published,
so `cityjson-index` can be published independently as long as the declared
version range stays compatible.

The package long description comes from [python/README.md](../python/README.md).
Keep that file short and distribution-focused.

## Dependency Policy

Use this dependency line in [python/pyproject.toml](../python/pyproject.toml):

```toml
dependencies = ["cityjson-lib~=0.5.0"]
```

That means:

- `cityjson-index` resolves `cityjson-lib` from PyPI during installation
- the local `[tool.uv.sources]` entry remains for development only

## Publish Flow

1. Build and publish the `cityjson-index` Python package.
2. Verify `pip install cityjson-index` in a clean virtualenv.
3. Run the Python smoke tests against the installed wheel.
4. If the dependency range ever changes, confirm the new `cityjson-lib`
   version is already on PyPI before publishing `cityjson-index` against it.

## Release Checklist

- Build an sdist and a wheel for this package.
- Confirm the wheel metadata lists `cityjson-lib` as an install requirement.
- Install both packages into a fresh environment with no local path overrides.
- Run [python/tests/test_api.py](../python/tests/test_api.py) after installation.
- Keep the runtime dependency range compatible with the released `cityjson-lib`
  line, and widen it only when the Python API is confirmed to remain stable.

## Notes

- Do not add a reverse dependency from `cityjson-lib` back to `cityjson-index`.
- Keep the development path override in `[tool.uv.sources]` local to this repo.
- If a future release requires a breaking dependency bump, publish the needed
  `cityjson-lib` version first, then cut `cityjson-index` against the new range.
