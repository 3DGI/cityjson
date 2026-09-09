"""Validate the composed CityJSON specification site before publication."""

from html.parser import HTMLParser
from pathlib import Path
import sys
from urllib.parse import unquote, urlsplit


CANONICAL_HOST = "specs.citymodel.3dgi.nl"
REQUIRED_PAGES = (
    "index.html",
    "citation/index.html",
    "license/index.html",
    "arrow/index.html",
    "arrow/cityjson-arrow-ipc-spec/index.html",
    "arrow/package-schema/index.html",
    "arrow/package-spec/index.html",
    "parquet/index.html",
    "parquet/cityjson-parquet-spec/index.html",
    "parquet/package-schema/index.html",
    "parquet/native-parquet-dataset/index.html",
)
NORMATIVE_PAGES = (
    "arrow/cityjson-arrow-ipc-spec/index.html",
    "arrow/package-schema/index.html",
    "arrow/package-spec/index.html",
    "parquet/cityjson-parquet-spec/index.html",
    "parquet/package-schema/index.html",
    "parquet/native-parquet-dataset/index.html",
)
REQUIRED_METADATA = (
    "Experimental",
    "cityjson-arrow.package.v3alpha3",
    "Balázs Dukai",
    "3DGI",
    "CC BY 4.0",
    "3DGI/cityjson-rs",
)


class LinkCollector(HTMLParser):
    """Collect links from one rendered HTML document."""

    def __init__(self) -> None:
        super().__init__()
        self.links: list[str] = []

    def handle_starttag(
        self, tag: str, attrs: list[tuple[str, str | None]]
    ) -> None:
        if tag not in {"a", "link", "script", "img"}:
            return

        target_attribute = "href" if tag in {"a", "link"} else "src"
        for name, value in attrs:
            if name == target_attribute and value:
                self.links.append(value)


def rendered_target(site_root: Path, source: Path, link: str) -> Path | None:
    """Return the local file targeted by an internal rendered-site link."""
    parsed = urlsplit(link)
    if parsed.scheme and parsed.scheme not in {"http", "https"}:
        return None
    if parsed.netloc and parsed.netloc != CANONICAL_HOST:
        return None
    if not parsed.path:
        return None

    decoded_path = unquote(parsed.path)
    if decoded_path.startswith("/"):
        candidate = site_root / decoded_path.removeprefix("/")
    else:
        candidate = source.parent / decoded_path

    if candidate.is_dir() or decoded_path.endswith("/"):
        return candidate / "index.html"
    return candidate


def validate_required_pages(site_root: Path) -> list[str]:
    """Validate required output pages and normative metadata."""
    errors: list[str] = []
    for relative_path in REQUIRED_PAGES:
        page = site_root / relative_path
        if not page.is_file():
            errors.append(f"missing required page: {relative_path}")
            continue

        canonical_path = relative_path.removesuffix("index.html")
        canonical_url = f"https://{CANONICAL_HOST}/{canonical_path}"
        content = page.read_text(encoding="utf-8")
        if canonical_url not in content:
            errors.append(
                f"{relative_path}: missing canonical URL {canonical_url!r}"
            )

    for relative_path in NORMATIVE_PAGES:
        page = site_root / relative_path
        if not page.is_file():
            continue
        content = page.read_text(encoding="utf-8")
        for marker in REQUIRED_METADATA:
            if marker not in content:
                errors.append(f"{relative_path}: missing metadata {marker!r}")
    return errors


def validate_internal_links(site_root: Path) -> list[str]:
    """Validate local links in all generated HTML pages."""
    errors: list[str] = []
    for page in site_root.rglob("*.html"):
        collector = LinkCollector()
        collector.feed(page.read_text(encoding="utf-8"))
        for link in collector.links:
            target = rendered_target(site_root, page, link)
            if target is not None and not target.exists():
                errors.append(
                    f"{page.relative_to(site_root)}: broken link {link!r}"
                )
    return errors


def main() -> None:
    """Run all publication validations."""
    if len(sys.argv) != 2:
        sys.stderr.write("usage: check-spec-site.py SITE_ROOT\n")
        raise SystemExit(2)

    site_root = Path(sys.argv[1])
    if not site_root.is_dir():
        sys.stderr.write(f"site root does not exist: {site_root}\n")
        raise SystemExit(2)

    errors = validate_required_pages(site_root)
    errors.extend(validate_internal_links(site_root))
    if errors:
        sys.stderr.write("Specification site validation failed:\n")
        for error in errors:
            sys.stderr.write(f"- {error}\n")
        raise SystemExit(1)

    sys.stdout.write("Specification site validation passed.\n")


if __name__ == "__main__":
    main()
