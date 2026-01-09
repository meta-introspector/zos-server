#!/bin/bash

# Fix all error conversion issues by replacing ? with .map_err(|e| e.to_string())?
find src/plugins -name "*.rs" -exec sed -i 's/Library::new(\([^)]*\))?/Library::new(\1).map_err(|e| e.to_string())?/g' {} \;
find src/plugins -name "*.rs" -exec sed -i 's/\.get(\([^)]*\))?/.get(\1).map_err(|e| e.to_string())?/g' {} \;
find src/plugins -name "*.rs" -exec sed -i 's/CString::new(\([^)]*\))?/CString::new(\1).map_err(|e| e.to_string())?/g' {} \;

echo "Fixed error conversions in plugin files"
