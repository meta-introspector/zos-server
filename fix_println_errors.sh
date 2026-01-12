#!/bin/bash

# Fix println format errors
sed -i 's/println!("=".repeat(\([0-9]*\)));/println!("{}", "=".repeat(\1));/g' src/lean4_foundation.rs
sed -i 's/println!("=".repeat(\([0-9]*\)));/println!("{}", "=".repeat(\1));/g' src/meta_fixed_point.rs
sed -i 's/println!("=".repeat(\([0-9]*\)));/println!("{}", "=".repeat(\1));/g' src/meta_introspector_capstone.rs
sed -i 's/println!("=".repeat(\([0-9]*\)));/println!("{}", "=".repeat(\1));/g' src/nidex_builder.rs

# Fix float format errors
sed -i 's/{:.6f}/{:.6}/g' src/iree_kleene_backend.rs

echo "Fixed println and format errors"
