import xml.etree.ElementTree as ET
import re

def clean_xml(file_path):
    tree = ET.parse(file_path)
    root = tree.getroot()
    
    # 1. Fix numeric fields having units or '-'
    for elem in root.iter():
        if elem.text:
            original = elem.text.strip()
            # Replace '-' with '0.0' for numeric fields (heuristic: if tag suggests number)
            # Actually, let's just look at the specific errors we saw.
            
            if original == '-':
                # List of tags that failed with '-'
                if elem.tag in ['COARSE_FINE_DIFF', 'MOISTURE', 'DIASTATIC_POWER', 'PROTEIN']:
                    elem.text = '0.0'
            
            # Remove units like " IBU", " %"
            # Regex for "number unit"
            # Matches: 32.4 IBU, 5.3 %
            match = re.match(r'^([\d\.]+)\s*(IBU|%|SG|SRM|cal/pint)$', original)
            if match:
                elem.text = match.group(1)
                
    # 2. Fix empty WATERS
    # We need to find RECIPEs that have WATERS with no WATER children
    # Iterating and modifying structure is tricky.
    
    for recipe in root.findall('RECIPE'):
        waters = recipe.find('WATERS')
        if waters is not None:
            # Check if it has any WATER child
            if len(waters.findall('WATER')) == 0:
                # Remove the WATERS tag completely
                recipe.remove(waters)

    tree.write(file_path, encoding='UTF-8', xml_declaration=True)

if __name__ == "__main__":
    clean_xml("samples/corrected/recipes.xml")
    print("Cleaned samples/corrected/recipes.xml")
