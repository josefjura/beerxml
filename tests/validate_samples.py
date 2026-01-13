import os
import sys
from lib.schema_parser import parse_xsd
from lib.validator import validate_file

def run_tests():
    xsd_path = "spec/xsd/beerxml.xsd"
    samples_dir = "samples"
    
    if not os.path.exists(xsd_path):
        print("XSD not found.")
        return

    print("Parsing Schema...")
    schema = parse_xsd(xsd_path)
    
    print(f"Scanning {samples_dir}...")
    
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
        errors = validate_file(file_path, schema)
        
        if not errors:
            print("OK")
            passed += 1
        else:
            print("FAIL")
            for e in errors:
                print(f"  - {e}")
            failed += 1
            
    print("-" * 30)
    print(f"Total: {total}, Passed: {passed}, Failed: {failed}")
    
    if failed > 0:
        sys.exit(1)

if __name__ == "__main__":
    run_tests()
