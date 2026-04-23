from __future__ import annotations

import sys

from cityjson_lib import WriteOptions
from cityjson_lib._fake_complete import build_fake_complete_model


def main() -> int:
    model = build_fake_complete_model()
    try:
        sys.stdout.write(
            model.serialize_document(
                WriteOptions(pretty=True, validate_default_themes=False)
            )
        )
    finally:
        model.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
