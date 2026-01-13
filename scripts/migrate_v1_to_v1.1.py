import argparse
import xml.etree.ElementTree as ET
import re
import sys
import os
from datetime import datetime

# Namespace for v1.1
NS_URL = "http://beerxml.com/v1.1"
ET.register_namespace('', NS_URL)

def parse_date(date_str):
    """
    Attempt to parse a date string into YYYY-MM-DD.
    Returns the original string if parsing fails (to avoid data loss),
    but prints a warning.
    """
    if not date_str:
        return None
        
    date_str = date_str.strip()
    
    # Common formats found in BeerXML 1.0 files
    formats = [
        "%d %b %y",      # 3 Jan 04
        "%d %b %Y",      # 3 Jan 2004
        "%m/%d/%Y",      # 1/3/2004 (US)
        "%m/%d/%y",      # 1/3/04
        "%Y-%m-%d",      # 2004-01-03 (ISO)
        "%d.%m.%Y",      # 03.01.2004 (EU)
    ]
    
    for fmt in formats:
        try:
            dt = datetime.strptime(date_str, fmt)
            return dt.strftime("%Y-%m-%d")
        except ValueError:
            continue
            
    # Fallback: Try detailed regex for "3 Jan 04" if %b is locale-dependent
    # Not implementing complex locale logic here to keep it dependency-free.
    
    print(f"Warning: Could not parse date '{date_str}'. Keeping original.")
    return date_str

def clean_number(value):
    """
    Strip units and non-numeric characters from a string to extract a float/decimal.
    Handles '34.5 IBU', '5.5 %', '-'.
    """
    if not value:
        return "0.0"
    
    val = value.strip()
    
    if val == '-':
        return "0.0"
        
    # Extract the first float-like number found at the start
    # Match optionally negative, digits, optional dot, optional digits
    match = re.match(r'^-?\d+(\.\d+)?', val)
    if match:
        return match.group(0)
    
    # If no match at start, try searching anywhere?
    # Spec generally puts number first.
    
    return val

def clean_boolean(value):
    """
    Normalize booleans to 'true' or 'false'.
    """
    if not value:
        return "false"
        
    v = value.strip().upper()
    if v in ['TRUE', 'YES', '1']:
        return "true"
    if v in ['FALSE', 'NO', '0']:
        return "false"
        
    return "false"

def migrate_tree(tree):
    root = tree.getroot()
    
    # 1. Update/Add Namespace
    # In ElementTree, changing the default namespace of an existing tree is hard.
    # It's often easier to recreate the tree or strip namespaces and re-add.
    # Here we will just rely on the register_namespace global and setting the tag correctly.
    
    # Helper to fix tag name (add namespace)
    def fix_tag(elem):
        # Strip existing namespace if any
        if '}' in elem.tag:
            local_name = elem.tag.split('}', 1)[1]
        else:
            local_name = elem.tag
        
        elem.tag = f"{{{NS_URL}}}{local_name}"

    # Recursively update tags and process text content
    for elem in root.iter():
        fix_tag(elem)
        
        # Determine the field type context based on tag name (simple heuristic)
        # In a robust migration, we would check the parent type, but tag names are fairly unique in BeerXML.
        
        tag_local = elem.tag.split('}', 1)[1]
        
        # Version Tag -> 1.1
        if tag_local == 'VERSION':
            elem.text = "1.1"
            
        elif elem.text:
            # Numeric cleaning
            # List of known numeric tags from Appendix A or loose v1.0
            numeric_tags = [
                'IBU', 'IBUS', 'EST_ABV', 'ABV', 'ACTUAL_EFFICIENCY', 
                'CALORIES', 'AMOUNT', 'BATCH_SIZE', 'BOIL_SIZE', 'OG', 'FG',
                'COLOR', 'YIELD', 'ALPHA', 'BETA', 'HSI', 'HUMULENE', 
                'CARYOPHYLLENE', 'COHUMULONE', 'MYRCENE', 
                'PH', 'CARBONATION', 'ATTENUATION',
                'MIN_TEMPERATURE', 'MAX_TEMPERATURE', 'STEP_TEMP', 'END_TEMP',
                'TUN_TEMP', 'SPARGE_TEMP', 'GRAIN_TEMP', 'PRIMARY_TEMP', 
                'SECONDARY_TEMP', 'TERTIARY_TEMP', 'AGE_TEMP', 'CARBONATION_TEMP',
                'TIME', 'BOIL_TIME', 'STEP_TIME', 'RAMP_TIME',
                'COARSE_FINE_DIFF', 'MOISTURE', 'DIASTATIC_POWER', 'PROTEIN',
                'IBU_GAL_PER_LB', 'POTENTIAL'
            ]
            
            if tag_local in numeric_tags:
                elem.text = clean_number(elem.text)
                
            # Date cleaning
            elif tag_local in ['DATE', 'CULTURE_DATE']:
                elem.text = parse_date(elem.text)
                
            # Boolean cleaning
            elif tag_local in ['AMOUNT_IS_WEIGHT', 'ADD_AFTER_BOIL', 'RECOMMEND_MASH', 
                               'ADD_TO_SECONDARY', 'EQUIP_ADJUST', 'CALC_BOIL_VOLUME', 'FORCED_CARBONATION']:
                elem.text = clean_boolean(elem.text)

    # Post-process: Remove DISPLAY_* and other removed fields
    # We iterate again or do it safely. Since we are iterating and modifying text in the first pass,
    # removing elements needs to be done carefully (iterating over a copy or using findall/remove).
    
    # List of tags to REMOVE in v1.1
    tags_to_remove = [
        'DISPLAY_AMOUNT', 'DISPLAY_TIME', 'DISPLAY_BOIL_SIZE', 'DISPLAY_BATCH_SIZE',
        'DISPLAY_TUN_VOLUME', 'DISPLAY_TUN_WEIGHT', 'DISPLAY_TOP_UP_WATER',
        'DISPLAY_TRUB_CHILLER_LOSS', 'DISPLAY_LAUTER_DEADSPACE', 'DISPLAY_TOP_UP_KETTLE',
        'DISPLAY_OG_MIN', 'DISPLAY_OG_MAX', 'DISPLAY_FG_MIN', 'DISPLAY_FG_MAX',
        'DISPLAY_COLOR_MIN', 'DISPLAY_COLOR_MAX', 'OG_RANGE', 'FG_RANGE', 'IBU_RANGE',
        'CARB_RANGE', 'COLOR_RANGE', 'ABV_RANGE', 'DISPLAY_GRAIN_TEMP', 'DISPLAY_TUN_TEMP',
        'DISPLAY_SPARGE_TEMP', 'DISPLAY_STEP_TEMP', 'DISPLAY_INFUSE_AMT',
        'DISPLAY_OG', 'DISPLAY_FG', 'DISPLAY_PRIMARY_TEMP', 'DISPLAY_SECONDARY_TEMP',
        'DISPLAY_TERTIARY_TEMP', 'DISPLAY_AGE_TEMP', 'DISPLAY_CARB_TEMP',
        'EST_OG', 'EST_FG', 'EST_COLOR' # These are often strings with units in v1.0, removing them enforces calculation or clean re-add later
    ]
    
    # Function to remove children recursively
    def prune_elements(element):
        to_remove = []
        for child in element:
            local_tag = child.tag.split('}', 1)[1] if '}' in child.tag else child.tag
            if local_tag in tags_to_remove:
                to_remove.append(child)
            else:
                prune_elements(child)
        
        for child in to_remove:
            element.remove(child)

    prune_elements(root)

    return tree

def migrate_file(input_path, output_path):
    try:
        tree = ET.parse(input_path)
        new_tree = migrate_tree(tree)
        
        # Ensure output directory exists
        os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
        
        new_tree.write(output_path, encoding='UTF-8', xml_declaration=True)
        print(f"Migrated: {input_path} -> {output_path}")
        return True
    except Exception as e:
        print(f"Failed to migrate {input_path}: {e}")
        return False

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Migrate BeerXML v1.0 files to v1.1")
    parser.add_argument("input", help="Input XML file or directory")
    parser.add_argument("output", help="Output XML file or directory")
    
    args = parser.parse_args()
    
    if os.path.isdir(args.input):
        # Batch mode
        if not os.path.exists(args.output):
            os.makedirs(args.output)
            
        for root, dirs, files in os.walk(args.input):
            for file in files:
                if file.lower().endswith('.xml'):
                    in_file = os.path.join(root, file)
                    # Rel path for structure preservation
                    rel_path = os.path.relpath(in_file, args.input)
                    out_file = os.path.join(args.output, rel_path)
                    
                    migrate_file(in_file, out_file)
    else:
        # Single file mode
        migrate_file(args.input, args.output)
