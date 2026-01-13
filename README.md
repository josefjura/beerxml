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
    *   `xsd/`: The XML Schema Definition for v1.0.
    *   `legacy/`: Original documentation files from beerxml.com.
    *   `v1.1/`: **BeerXML 1.1 (Draft)** - Modern "Clean XML" evolution.
        *   `RFC_01_goals.md`: Design goals for v1.1.
        *   `beerxml-1.1.xsd`: Strict schema for v1.1.
*   `lib/`: Python reference library for parsing and validation.
*   `tests/`: Validation scripts and test runners.
*   `samples/`:
    *   `v1.1_sample.xml`: Example of the new modern XML format.
    *   `original/`: The original samples from beerxml.com.
    *   `corrected/`: Fixed versions of the original samples.
    *   `brewtlery_unetice_realworld_sample.xml`: A real-world example from modern software.
*   `scripts/`: Utility scripts.

## Usage

### Prerequisites
*   Python 3

### Running Tests
To validate all samples (Detects v1.0 vs v1.1 automatically):

```bash
export PYTHONPATH=$PYTHONPATH:.
python3 tests/validate_samples.py
```

## Specification

*   [BeerXML 1.0 Spec](spec/v1.0.md)

*   [BeerXML 1.1 Spec (Draft)](spec/v1.1/v1.1.md)

*   **BeerXML 1.1 Goals**: See [RFC 01](spec/v1.1/RFC_01_goals.md)



## Contributing

We welcome community contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to propose changes (BIPs) or improve the tooling.



## License

Original BeerXML standard is copyright its respective authors.

This repository structure and tooling are provided under MIT License.


