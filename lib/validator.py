import re
import xml.etree.ElementTree as ET
from datetime import datetime
from decimal import Decimal, InvalidOperation

# BeerXML 1.1 "Clean XML" (docs/spec/v1.1/RFC_01_goals.md), as the 1.1 XSD
# settles it: the root is in the XSD's targetNamespace, every record's
# VERSION is the fixed decimal 1.1, booleans are xs:boolean, dates are ISO,
# no DISPLAY_* fields and the file is UTF-8.
NAMESPACE_V11 = "http://beerxml.com/v1.1"
XS_BOOLEANS = ['true', 'false', '1', '0']
XS_DECIMAL = re.compile(r'[+-]?([0-9]+(\.[0-9]*)?|\.[0-9]+)')
ENCODING_DECL = re.compile(rb"""^<\?xml\s[^>]*?encoding\s*=\s*["']([^"']*)["']""")


def validate_value(value, field_def, clean_xml=False):
    vtype = field_def['type']

    fixed = field_def.get('fixed')
    if fixed is not None:
        try:
            if XS_DECIMAL.fullmatch(value) and Decimal(value) == Decimal(fixed):
                return True, None
        except InvalidOperation:
            pass
        return False, f"Expected fixed value {fixed}, got '{value}'"

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
            
    elif vtype == 'boolean' and clean_xml:
        if value in XS_BOOLEANS:
            return True, None
        return False, f"Expected boolean (true/false/1/0), got '{value}'"

    elif vtype == 'boolean':
        if value.lower() in XS_BOOLEANS:
            return True, None
        return False, f"Expected boolean (true/false/1/0), got '{value}'"
        
    elif vtype == 'date' or field_def['original_type'] == 'xs:date':
        if re.fullmatch(r'[0-9]{4}-[0-9]{2}-[0-9]{2}', value):
            try:
                datetime.strptime(value, '%Y-%m-%d')
                return True, None
            except ValueError:
                pass
        return False, f"Expected ISO date (YYYY-MM-DD), got '{value}'"

    elif vtype == 'enum':
        if value in field_def['enum_values']:
            return True, None
        return False, f"Value '{value}' not in allowed list: {field_def['enum_values']}"
        
    return True, None


def clean_xml_document_errors(xml_path, root):
    """The document-wide 1.1 rules: UTF-8 and the 1.1 namespace."""
    errors = []
    with open(xml_path, 'rb') as f:
        head = f.read(1024)
    if head.startswith(b'\xef\xbb\xbf'):
        head = head[3:]
    match = ENCODING_DECL.match(head)
    if match and match.group(1).decode('ascii', 'replace').upper() != 'UTF-8':
        errors.append(f"Encoding must be UTF-8, got '{match.group(1).decode('ascii', 'replace')}'")
    namespace = root.tag[1:].split('}', 1)[0] if root.tag.startswith('{') else None
    if namespace != NAMESPACE_V11:
        errors.append(f"Root element must be in namespace {NAMESPACE_V11}, got {namespace!r}")
    return errors


def validate_file(xml_path, schema, xsd_path=None, clean_xml=False):
    """Validate a file against a parsed schema.

    clean_xml applies the BeerXML 1.1 Clean XML rules on top of the schema.
    """
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
            
        errors = clean_xml_document_errors(xml_path, root) if clean_xml else []
        return errors + validate_element(root, tag, schema, clean_xml=clean_xml)
    except ET.ParseError as e:
        return [f"XML Parse Error: {e}"]

def validate_element(element, type_name, schema, path="", clean_xml=False):
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
                child_errors = validate_element(child, get_clean_tag(child), schema, f"{path}/{clean_tag}", clean_xml)
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
            if clean_xml and field_def is None and c_tag.startswith('DISPLAY_'):
                errors.append(f"Display field not allowed in 1.1: {path}/{clean_tag}/{c_tag}")
            if field_def:
                if child.text:
                    valid, msg = validate_value(child.text.strip(), field_def, clean_xml)
                    if not valid:
                        errors.append(f"Invalid value at {path}/{clean_tag}/{c_tag}: {msg}")
                
                original = field_def['original_type']
                if original.endswith('Type'):
                    subtype = original[:-4]
                    child_errors = validate_element(child, subtype, schema, f"{path}/{clean_tag}", clean_xml)
                    errors.extend(child_errors)
    return errors


