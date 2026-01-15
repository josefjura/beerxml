import xml.etree.ElementTree as ET
import sys
import os
import argparse

def parse_xsd(xsd_path):
    tree = ET.parse(xsd_path)
    root = tree.getroot()
    ns = {'xs': 'http://www.w3.org/2001/XMLSchema'}
    
    # Map XSD types to readable types
    type_mapping = {
        'xs:string': 'Text',
        'xs:int': 'Integer',
        'xs:decimal': 'Decimal',
        'xs:boolean': 'Boolean',
        'xs:date': 'Date (YYYY-MM-DD)',
        'booleanType': 'Boolean',
        'decimalType': 'Decimal',
        'temperatureType': 'Temperature (C)',
        'timeType': 'Time (min)',
        'percentageType': 'Percentage'
    }

    # Find all simpleTypes (enumerations)
    enums = {}
    for simple_type in root.findall('xs:simpleType', ns):
        name = simple_type.get('name')
        restriction = simple_type.find('xs:restriction', ns)
        if restriction is not None:
            base = restriction.get('base')
            enumeration_values = [e.get('value') for e in restriction.findall('xs:enumeration', ns)]
            if enumeration_values:
                enums[name] = enumeration_values
            
            if base == 'xs:string' and not enumeration_values:
                type_mapping[name] = 'Text'
            elif base == 'xs:decimal':
                type_mapping[name] = 'Decimal'

    complex_types = {}
    for complex_type in root.findall('xs:complexType', ns):
        name = complex_type.get('name')
        if not name:
            continue
            
        # Remove "Type" suffix for cleaner names if present
        if name.endswith('Type'):
            clean_name = name[:-4]
        else:
            clean_name = name
            
        fields = []
        # Handle xs:all and xs:sequence
        container = complex_type.find('xs:all', ns)
        if container is None:
            container = complex_type.find('xs:sequence', ns)
            
        if container is not None:
            for element in container.findall('xs:element', ns):
                field_name = element.get('name')
                field_type = element.get('type')
                
                # Handle minOccurs
                min_occurs = element.get('minOccurs', '1') # Default is 1
                required = "Yes" if min_occurs != '0' else "No"
                
                # Handle fixed values (like VERSION)
                fixed_val = element.get('fixed')
                
                # Check if type is an enum
                readable_type = type_mapping.get(field_type, field_type)
                
                # Clean up prefixes if still present
                if readable_type and readable_type.startswith('xs:'):
                     readable_type = type_mapping.get(readable_type, readable_type[3:].title())

                enum_values = enums.get(field_type)
                
                fields.append({
                    'name': field_name,
                    'type': readable_type,
                    'required': required,
                    'enum': enum_values,
                    'fixed': fixed_val
                })
        
        complex_types[clean_name] = fields

    return complex_types

def generate_markdown(complex_types, title, version):
    md = []
    md.append(f"# {title}")
    md.append(f"\n*Auto-generated from BeerXML {version} XSD*")
    
    if version == "1.1":
        md.append("\n**Note:** All numeric values must be raw numbers (no units). All Dates must be ISO-8601.")
        md.append("\n## Changes from v1.0")
        md.append("\nThe following changes have been made to modernize and clarify the standard:")
        md.append("\n1.  **Namespace:** All v1.1 files must use the namespace `http://beerxml.com/v1.1`.")
        md.append("2.  **Strict Numeric Types:** Fields that represent numbers (e.g., `AMOUNT`, `TEMPERATURE`) must now be pure XML decimals or integers. They **cannot** contain units (e.g., use `5.5` not `5.5 kg`).")
        md.append("    *   *Rationale:* This simplifies parsing and removes ambiguity. Units are strictly defined by the standard (kg, Liters, Celsius, etc.).")
        md.append("3.  **Strict Date Format:** All dates must be in ISO-8601 format (`YYYY-MM-DD`).")
        md.append("    *   *Rationale:* v1.0 allowed loose text dates which caused interoperability issues.")
        md.append("4.  **Booleans:** Boolean fields must be `true` or `false` (lowercase recommended, though XML Schema allows `1`/`0`).")
        md.append("5.  **Removal of Display Tags:** All `DISPLAY_*` fields (e.g., `DISPLAY_AMOUNT`, `DISPLAY_TIME`) have been removed.")
        md.append("    *   *Rationale:* Formatting for display is the responsibility of the application, not the data interchange format.")
        md.append("6.  **Removal of Ranges:** `_RANGE` fields (e.g., `OG_RANGE`) in Styles have been removed where `_MIN` and `_MAX` fields already exist.")
    
    # Order of presentation (logical order)
    order = ['Recipe', 'Style', 'Hop', 'Fermentable', 'Yeast', 'Misc', 'Water', 'Equipment', 'Mash', 'MashStep']
    
    for name in order:
        if name in complex_types:
            md.append(f"\n## {name} Record")
            md.append("| Tag | Type | Required | Description |")
            md.append("| :--- | :--- | :--- | :--- |")
            
            for field in complex_types[name]:
                desc = []
                if field['fixed']:
                    desc.append(f"Fixed Value: **{field['fixed']}**")
                if field['enum']:
                    desc.append(f"Values: {', '.join(field['enum'])}")
                
                desc_str = "<br>".join(desc)
                
                md.append(f"| {field['name']} | {field['type']} | {field['required']} | {desc_str} |")
                
    return "\n".join(md)

if __name__ == "__main__":
    # V1.0 Generation - DISABLED (We now have a manually improved version)
    # v1_xsd = "docs/spec/v1.0/beerxml-1.0.xsd"
    # if os.path.exists(v1_xsd):
    #     data = parse_xsd(v1_xsd)
    #     md_content = generate_markdown(data, "BeerXML 1.0 Specification", "1.0")
    #     with open("docs/spec/v1.0/v1.0.md", "w") as f:
    #         f.write(md_content)
    #     print("Generated docs/spec/v1.0/v1.0.md")

    # V1.1 Generation
    v11_xsd = "docs/spec/v1.1/beerxml-1.1.xsd"
    if os.path.exists(v11_xsd):
        data = parse_xsd(v11_xsd)
        md_content = generate_markdown(data, "BeerXML 1.1 Specification", "1.1")
        with open("docs/spec/v1.1/v1.1.md", "w") as f:
            f.write(md_content)
        print("Generated docs/spec/v1.1/v1.1.md")
