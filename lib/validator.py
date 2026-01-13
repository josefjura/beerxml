import xml.etree.ElementTree as ET
import sys

def validate_value(value, field_def):
    vtype = field_def['type']
    
    if vtype == 'integer':
        try:
            int(value)
            return True, None
        except ValueError:
            return False, f"Expected integer, got '{value}'"
            
    elif vtype == 'float':
        try:
            float(value)
            return True, None
        except ValueError:
            return False, f"Expected float, got '{value}'"
            
    elif vtype == 'boolean':
        if value.lower() in ['true', 'false', '1', '0', 'TRUE', 'FALSE']:
            return True, None
        return False, f"Expected boolean (true/false/1/0), got '{value}'"
        
    elif vtype == 'date' or field_def['original_type'] == 'xs:date':
        import re
        if re.match(r'^\d{4}-\d{2}-\d{2}$', value):
            return True, None
        return False, f"Expected ISO date (YYYY-MM-DD), got '{value}'"

    elif vtype == 'enum':
        if value in field_def['enum_values']:
            return True, None
        return False, f"Value '{value}' not in allowed list: {field_def['enum_values']}"
        
    return True, None


def validate_file(xml_path, schema, xsd_path=None):
    # If we have a real XSD and want to use a real validator, we could...
    # but we don't have lxml or xmlschema.
    # So we stick to our custom validator but make it namespace-aware.
    try:
        tree = ET.parse(xml_path)
        root = tree.getroot()
        
        # Strip namespace for simple matching in our custom validator
        tag = root.tag
        if '}' in tag:
            tag = tag.split('}', 1)[1]
            
        return validate_element(root, tag, schema)
    except ET.ParseError as e:
        return [f"XML Parse Error: {e}"]

def validate_element(element, type_name, schema, path=""):
    errors = []
    
    # Simple namespace stripping for children
    def get_clean_tag(e):
        return e.tag.split('}', 1)[1] if '}' in e.tag else e.tag

    definition = schema.get(type_name)
    if not definition:
        # Match tag name to type name (case-insensitive, ignoring underscores)
        clean_tag = get_clean_tag(element)
        candidate = clean_tag.upper().replace('_', '')
        for key in schema:
            if key.upper().replace('_', '') == candidate:
                definition = schema[key]
                break
                
        if not definition:
            for child in element:
                child_errors = validate_element(child, get_clean_tag(child), schema, f"{path}/{clean_tag}")
                errors.extend(child_errors)
            return errors

    if definition:
        clean_tag = get_clean_tag(element)
        for field_name, field_def in definition.items():
            # Find child ignoring namespace
            child = None
            for c in element:
                if get_clean_tag(c) == field_name:
                    child = c
                    break
            
            if field_def['required'] and child is None:
                errors.append(f"Missing required field: {path}/{clean_tag}/{field_name}")
        
        for child in element:
            c_tag = get_clean_tag(child)
            field_def = definition.get(c_tag)
            if field_def:
                if child.text:
                    valid, msg = validate_value(child.text.strip(), field_def)
                    if not valid:
                        errors.append(f"Invalid value at {path}/{clean_tag}/{c_tag}: {msg}")
                
                original = field_def['original_type']
                if original.endswith('Type'):
                    subtype = original[:-4]
                    child_errors = validate_element(child, subtype, schema, f"{path}/{clean_tag}")
                    errors.extend(child_errors)
    return errors


