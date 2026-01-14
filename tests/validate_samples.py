import os
import sys
import xml.etree.ElementTree as ET

from lib.schema_parser import parse_xsd
from lib.validator import validate_file

def run_tests():
    xsd_v10 = "docs/spec/v1.0/beerxml-1.0.xsd"
    xsd_v11 = "docs/spec/v1.1/beerxml-1.1.xsd"
    samples_dir = "samples"
    
    print("Parsing Schemas...")
    schema_v10 = parse_xsd(xsd_v10)
    schema_v11 = parse_xsd(xsd_v11)
    
    files_to_test = []
    for root, dirs, files in os.walk(samples_dir):
        for file in files:
            if file.endswith(".xml"):
                files_to_test.append(os.path.join(root, file))
    
    total = 0
    passed = 0
    failed = 0
    
    for file_path in files_to_test:
        total += 1
        print(f"Validating {file_path}...", end=" ")
        
        # Detect version
        try:
            tree = ET.parse(file_path)
            root = tree.getroot()
            
            # Heuristic for version detection
            is_v11 = 'beerxml.com/v1.1' in root.tag or 'xmlns' in root.attrib and 'v1.1' in root.attrib['xmlns']
            # Also check <VERSION> tag if present
            version_elem = root.find('.//VERSION') # very rough
            if version_elem is not None and version_elem.text == '1.1':
                is_v11 = True
                
            schema = schema_v11 if is_v11 else schema_v10
            version_str = "v1.1" if is_v11 else "v1.0"
            
            errors = validate_file(file_path, schema)
            
            if not errors:
                print(f"OK ({version_str})")
                passed += 1
            else:
                print(f"FAIL ({version_str})")
                for e in errors:
                    print(f"  - {e}")
                failed += 1
        except Exception as e:
            print(f"ERROR: {e}")
            failed += 1
            
    print("-" * 30)
    print(f"Total: {total}, Passed: {passed}, Failed: {failed}")
    
    if failed > 0:
        sys.exit(1)


if __name__ == "__main__":
    run_tests()
