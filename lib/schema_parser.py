import xml.etree.ElementTree as ET

def parse_xsd(xsd_path):
    tree = ET.parse(xsd_path)
    root = tree.getroot()
    ns = {'xs': 'http://www.w3.org/2001/XMLSchema'}
    
    # Map XSD types to readable types/internal types
    # We will use this for validation logic
    type_mapping = {
        'xs:string': 'text',
        'xs:int': 'integer',
        'xs:decimal': 'float',
        'xs:date': 'date',
        'xs:boolean': 'boolean',
        'booleanType': 'boolean',
        'decimalType': 'float',
        'temperatureType': 'float',
        'timeType': 'float',
        'percentageType': 'float'
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
                type_mapping[name] = 'text'
            elif base == 'xs:decimal':
                type_mapping[name] = 'float'

    complex_types = {}
    for complex_type in root.findall('xs:complexType', ns):
        name = complex_type.get('name')
        # Remove "Type" suffix for cleaner names if present
        if name.endswith('Type'):
            clean_name = name[:-4]
        else:
            clean_name = name
            
        fields = {}
        # Handle xs:all and xs:sequence
        container = complex_type.find('xs:all', ns)
        if container is None:
            container = complex_type.find('xs:sequence', ns)
            
        if container is not None:
            for element in container.findall('xs:element', ns):
                field_name = element.get('name')
                field_type = element.get('type')
                min_occurs = element.get('minOccurs', '1') # Default is 1
                
                required = True if min_occurs != '0' else False
                
                # Check if type is an enum
                # If field_type is in enums, it's an enum type
                is_enum = field_type in enums
                
                # Resolve basic type
                basic_type = type_mapping.get(field_type, field_type)
                if is_enum:
                    basic_type = 'enum'

                fields[field_name] = {
                    'type': basic_type,
                    'original_type': field_type,
                    'required': required,
                    'enum_values': enums.get(field_type)
                }
        
        complex_types[clean_name] = fields

    return complex_types
