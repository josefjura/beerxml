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
        if value.upper() in ['TRUE', 'FALSE']:
            return True, None
        return False, f"Expected TRUE/FALSE, got '{value}'"
        
    elif vtype == 'enum':
        # Case sensitive check as per XML usually, but BeerXML might be loose. 
        # Standard says "TRUE/FALSE" uppercase, so let's assume enums are case sensitive as per XSD.
        # However, checking the XSD, values are capitalized (e.g., "Ale").
        if value in field_def['enum_values']:
            return True, None
        return False, f"Value '{value}' not in allowed list: {field_def['enum_values']}"
        
    return True, None

def validate_element(element, type_name, schema, path=""):
    errors = []
    
    if type_name not in schema:
        # It might be a list wrapper like RECIPES -> RECIPE
        # Check if it has children that match a known type
        # For BeerXML, the root types (RECIPES, HOPS, etc.) are just wrappers.
        # The schema_parser mapped complex types like 'Recipe' (from RecipeType).
        
        # Heuristic: if element tag is plural of a known type or specific wrapper
        # Let's look at the element tag.
        pass

    # In BeerXML, we have structures like:
    # <RECIPES><RECIPE>...</RECIPE></RECIPES>
    # <HOPS><HOP>...</HOP></HOPS>
    
    # We need to find the definition for the current element.
    # The schema parser extracted ComplexTypes.
    # e.g. 'Recipe' for 'RecipeType'.
    
    # If this element corresponds to a ComplexType
    definition = schema.get(type_name)
    if not definition:
        # Try finding a type that matches the tag name
        # e.g. Tag <RECIPE> matches type 'Recipe'
        # e.g. Tag <HOP> matches type 'Hop'
        
        # Simple mapping for BeerXML based on naming convention
        # Capitalized Tag -> Title Case Type
        candidate = element.tag.title() 
        # Handle special cases if any? 
        # MASH_STEP -> MashStep
        candidate = candidate.replace('_', '').replace('Mashstep', 'MashStep') # rough
        
        # Better: Iterate schema keys and match case-insensitive
        for key in schema:
            if key.upper() == element.tag.upper().replace('_', ''):
                definition = schema[key]
                break
                
        if not definition:
            # If still not found, maybe it's a wrapper like 'RECIPES'
            # Wrappers aren't explicitly complexTypes with fields in the simplified parser, 
            # they are usually defined as elements in the XSD pointing to complexTypes.
            # But our parser focused on ComplexTypes.
            # Let's just recurse for children if we can't find a def.
            for child in element:
                child_errors = validate_element(child, child.tag, schema, f"{path}/{element.tag}")
                errors.extend(child_errors)
            return errors

    # If we have a definition, validate fields
    if definition:
        # Check required fields
        for field_name, field_def in definition.items():
            child = element.find(field_name)
            if field_def['required'] and child is None:
                errors.append(f"Missing required field: {path}/{element.tag}/{field_name}")
        
        # Validate present fields
        for child in element:
            field_def = definition.get(child.tag)
            if field_def:
                # Validate value
                if child.text:
                    valid, msg = validate_value(child.text.strip(), field_def)
                    if not valid:
                        errors.append(f"Invalid value at {path}/{element.tag}/{child.tag}: {msg}")
                
                # Recurse if it's a complex object (though in BeerXML, most nested things are inside wrappers)
                # But wait, our parser treats everything inside a ComplexType as a field.
                # If a field actually points to another ComplexType (like <STYLE> inside <RECIPE>),
                # We need to validate that child structure too.
                
                # Check if the field type is another ComplexType in our schema
                # In schema_parser, we resolved 'type' to basic types.
                # But 'original_type' holds the key.
                original = field_def['original_type']
                if original.endswith('Type'):
                    subtype = original[:-4] # e.g. StyleType -> Style
                    # Recurse
                    child_errors = validate_element(child, subtype, schema, f"{path}/{element.tag}")
                    errors.extend(child_errors)
            
            else:
                # Unknown tag? The standard says ignore non-standard tags.
                # So we won't flag error, maybe just warning if we were strict.
                pass
                
    return errors

def validate_file(xml_path, schema):
    try:
        tree = ET.parse(xml_path)
        root = tree.getroot()
        return validate_element(root, root.tag, schema)
    except ET.ParseError as e:
        return [f"XML Parse Error: {e}"]

