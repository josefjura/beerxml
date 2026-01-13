# BeerXML Standard

Reviving and maintaining the BeerXML standard for brewing recipes and data.

## Project Goal
This repository aims to:
1.  **Preserve** the original BeerXML 1.0 standard.
2.  **Clarify** the specification with readable documentation and reference implementations.
3.  **Validate** existing and new BeerXML files against the strict standard.
4.  **Evolve** the standard into a modern, semantic version (BeerXML v2.0) in the future.

## Directory Structure

*   `spec/`: The specification documents.
    *   `v1.0.md`: Markdown version of the BeerXML 1.0 spec (Auto-generated).
    *   `xsd/`: The XML Schema Definition.
    *   `legacy/`: Original documentation files.
*   `lib/`: Python reference library for parsing and validation.
*   `tests/`: Validation scripts and test runners.
*   `samples/`:
    *   `original/`: The original samples from beerxml.com (Note: Some contain strict validation errors).
    *   `corrected/`: Fixed versions of the original samples that pass strict validation.
    *   `brewtlery_unetice_realworld_sample.xml`: A real-world example from modern software.
*   `scripts/`: Utility scripts.

## Usage

### Prerequisites
*   Python 3

### Running Tests
To validate all samples against the XSD:

```bash
export PYTHONPATH=$PYTHONPATH:.
python3 tests/validate_samples.py
```

*Note: Expect `samples/original/recipes.xml` to fail validation due to historical non-compliance (units in numeric fields, etc.). usage of `samples/corrected/recipes.xml` is recommended for reference.*

## Specification
See [BeerXML 1.0 Specification](spec/v1.0.md) for the detailed tag reference.

## License
Original BeerXML standard is copyright its respective authors.
This repository structure and tooling are provided under MIT License (to be confirmed).
