# BeerXML Standard

This project maintains the BeerXML standard for brewing recipes and data. It includes the legacy v1.0 specification, a draft v1.1 specification, and Python-based tools for validation and migration.

## Project Structure

*   `docs/`: Documentation and specifications.
    *   `spec/v1.0/`: BeerXML 1.0 (Legacy) XSD and Markdown spec.
    *   `spec/v1.1/`: BeerXML 1.1 (Draft) XSD, Markdown spec, and RFCs.
*   `lib/`: Python library for schema parsing and XML validation.
*   `scripts/`: Utility scripts for data migration and cleanup.
*   `samples/`: Sample XML files for testing (original v1.0, corrected, and v1.1 examples).
*   `tests/`: Test runners (currently `validate_samples.py`).
*   `mkdocs.yml`: Configuration for the documentation site.

## Development

### Prerequisites

*   Python 3
*   `mkdocs` (for documentation, optional)

### Running Tests

To validate all XML samples against the schemas:

```bash
export PYTHONPATH=$PYTHONPATH:.
python3 tests/validate_samples.py
```

This script automatically detects whether a file is v1.0 or v1.1 and validates it against the appropriate schema.

### Data Migration

To migrate BeerXML v1.0 files to the v1.1 format:

```bash
python3 scripts/migrate_v1_to_v1.1.py <input_file_or_dir> <output_file_or_dir>
```

This script:
*   Adds the v1.1 namespace (`http://beerxml.com/v1.1`).
*   Cleans numeric fields (strips units).
*   Standardizes date formats to ISO 8601 (`YYYY-MM-DD`).
*   Normalizes booleans (`true`/`false`).
*   Removes deprecated `DISPLAY_*` fields.

### Documentation

The documentation is built with MkDocs.

```bash
# Serve locally
mkdocs serve

# Build static site
mkdocs build
```

## Conventions

*   **Language:** Python 3 for tooling.
*   **XML Parsing:** Uses `xml.etree.ElementTree`.
*   **Namespaces:** v1.1 uses `http://beerxml.com/v1.1`.
*   **Validation:** Custom validator in `lib/validator.py` that checks types (integer, float, boolean, date, enum) against the XSD definitions parsed by `lib/schema_parser.py`.
