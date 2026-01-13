# Contributing to BeerXML

First off, thank you for considering contributing to the BeerXML standard! It is community involvement that keeps open standards alive and relevant.

## Our Mission
To maintain a stable, unambiguous, and modern data exchange format for the brewing community, ensuring compatibility across software and hardware.

## How to Contribute

### 1. Reporting Issues
If you find a bug in the specification (e.g., an inconsistency in the XSD) or a bug in the provided tooling (Python library/scripts), please open an Issue on GitHub.

### 2. BeerXML Improvement Proposals (BIP)
For significant changes to the standard (like adding new tags or major version bumps), we follow a process similar to Python's PEPs or Rust's RFCs.

1.  **Draft**: Create a new Markdown file in `proposals/BIP-XXXX-description.md`.
2.  **Discuss**: Open a Pull Request to discuss the proposal with the community.
3.  **Prototype**: If applicable, provide a sample XML file and an updated XSD.
4.  **Acceptance**: Once a consensus is reached, the proposal is merged and scheduled for the next minor or major version release.

### 3. Improving Documentation
The Markdown specifications in `spec/` are auto-generated from the XSDs. To improve the descriptions:
1.  Modify the `<xs:documentation>` tags (once added) or the comments in the `.xsd` files.
2.  Run `python3 scripts/xsd_to_md.py` to regenerate the documentation.

### 4. Code Contributions
We welcome improvements to:
*   `lib/`: The reference implementation.
*   `scripts/`: Migration and documentation tools.
*   `tests/`: Adding more real-world brewing samples to our test suite.

## Technical Standards
*   **Version 1.0**: Strictly preserved for legacy compatibility. No changes allowed except for documented clarifications.
*   **Version 1.1+**: Must follow "Clean XML" principles (strict typing, ISO dates, UTF-8).
*   **Testing**: All changes must pass the validation suite:
    ```bash
    export PYTHONPATH=$PYTHONPATH:.
    python3 tests/validate_samples.py
    ```

## Code of Conduct
Please be respectful and constructive. We are all here because we love brewing and software!
