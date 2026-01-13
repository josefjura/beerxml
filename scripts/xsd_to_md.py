import xml.etree.ElementTree as ET
import sys
import os

def parse_xsd(xsd_path):
    tree = ET.parse(xsd_path)
    root = tree.getroot()
    ns = {'xs': 'http://www.w3.org/2001/XMLSchema'}
    
    # Map XSD types to readable types
    type_mapping = {
        'xs:string': 'Text',
        'xs:int': 'Integer',
        'xs:decimal': 'Floating Point',
        'booleanType': 'Boolean',
        'decimalType': 'Floating Point',
        'temperatureType': 'Temperature',
        'timeType': 'Time',
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
                type_mapping[name] = 'Floating Point'

    complex_types = {}
    for complex_type in root.findall('xs:complexType', ns):
        name = complex_type.get('name')
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
                min_occurs = element.get('minOccurs', '1') # Default is 1
                
                required = "Yes" if min_occurs != '0' else "No"
                
                # Check if type is an enum
                readable_type = type_mapping.get(field_type, field_type)
                enum_values = enums.get(field_type)
                
                fields.append({
                    'name': field_name,
                    'type': readable_type,
                    'required': required,
                    'enum': enum_values
                })
        
        complex_types[clean_name] = fields

    return complex_types

def generate_markdown(complex_types):
    md = []
    md.append("# BeerXML 1.0 Specification")
    md.append("\n*Generated from beerxml.xsd*")
    
    # Order of presentation (logical order)
    order = ['Recipe', 'Style', 'Hop', 'Fermentable', 'Yeast', 'Misc', 'Water', 'Equipment', 'Mash', 'MashStep']
    
    for name in order:
        if name in complex_types:
            md.append(f"\n## {name} Record")
            md.append("| Tag | Type | Required | Description |")
            md.append("| :--- | :--- | :--- | :--- |")
            
            for field in complex_types[name]:
                desc = ""
                if field['enum']:
                    desc = f"Values: {', '.join(field['enum'])}"
                
                md.append(f"| {field['name']} | {field['type']} | {field['required']} | {desc} |")
                
    return "\n".join(md)

if __name__ == "__main__":
    xsd_path = "spec/xsd/beerxml.xsd"
    if not os.path.exists(xsd_path):
        print(f"Error: {xsd_path} not found")
        sys.exit(1)
        
    data = parse_xsd(xsd_path)
    md_content = generate_markdown(data)
    
    with open("spec/v1.0.md", "w") as f:
        f.write(md_content)
    print("Generated spec/v1.0.md")
