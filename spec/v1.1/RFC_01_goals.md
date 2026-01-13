# BeerXML v1.1 - The "Clean XML" Standard (Draft)

## Philosophy
BeerXML 1.1 aims to be a **strict, modern, and unambiguous** revision of BeerXML 1.0. It maintains the XML structure to ensure ease of adoption for developers with existing BeerXML support, but eliminates the parsing ambiguities that plagued the original specification.

## Core Changes from v1.0

### 1. Data Types & Strictness
*   **Problem in v1.0:** Fields defined as "Floating Point" in the spec often contained string units in reality (e.g., `<IBU>34.2 IBU</IBU>`). This forced developers to write custom string parsers for numeric fields.
*   **Solution in v1.1:** All numeric fields MUST contain only raw numbers.
    *   **Valid:** `<IBU>34.2</IBU>`
    *   **Invalid:** `<IBU>34.2 IBU</IBU>`

### 2. Units of Measurement
*   **Problem in v1.0:** Units were implicit (kg, Liters) but Appendix A introduced "Display Fields" (like `DISPLAY_AMOUNT`) which encouraged redundant data and drift. Worse, developers often parsed these string fields instead of the canonical numeric fields, or wrote units directly into numeric fields.
*   **Solution in v1.1:**
    *   **Strict Canonical Units:** All weights are Kilograms (kg). All volumes are Liters (l). All temperatures are Celsius (C).
    *   **No Display Hints:** The Data Layer is not the Presentation Layer. It is the responsibility of the consuming application to convert `0.02835 kg` to `1 oz` if the user prefers Imperial units. We explicitly remove all "Display" tags to prevent data drift and ambiguity.

### 3. Date Format
*   **Problem in v1.0:** Dates were free-text (e.g., `3 Jan 04`, `Jan 3, 2004`).
*   **Solution in v1.1:** All dates MUST use **ISO 8601** format (`YYYY-MM-DD`).
    *   Example: `<DATE>2004-01-03</DATE>`

### 4. Character Encoding
*   **Problem in v1.0:** Defaulted to `ISO-8859-1`.
*   **Solution in v1.1:** File encoding MUST be **UTF-8**.

### 5. Namespaces & Versioning
*   **Problem in v1.0:** No namespace protection.
*   **Solution in v1.1:** 
    *   Root element must specify version: `<RECIPES version="1.1">`
    *   Consider XML Namespaces: `<RECIPES xmlns="http://beerxml.com/v1">`

## Example Comparison

**Legacy v1.0 (Ambiguous):**
```xml
<RECIPE>
  <NAME>My Stout</NAME>
  <DATE>3/1/2024</DATE>
  <IBU>35.5 IBU</IBU> 
</RECIPE>
```

**Proposed v1.1 (Clean):**
```xml
<RECIPE version="1.1">
  <NAME>My Stout</NAME>
  <DATE>2024-03-01</DATE>
  <IBU>35.5</IBU>
</RECIPE>
```
