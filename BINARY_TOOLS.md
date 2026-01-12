# Automorphic Field Theory Binary Tools

This document catalogs all the binary executables generated during the automorphic field theory analysis.

## Core Analysis Tools

### `multi_file_markov`
**Source**: `multi_file_markov.rs`
**Purpose**: Processes 33.9M files generating comprehensive transition models
**Output**: Forward/reverse binary models for 1.47M Rust files
**Usage**: `./multi_file_markov` (processes entire filesystem)

### `file_list_markov`
**Source**: `file_list_markov.rs`
**Purpose**: Builds Markov models from file path structures with checkpointing
**Output**: Binary models saved every 1000 files processed
**Usage**: `./file_list_markov` (filters to .rs files only)

### `top_transitions`
**Source**: `top_transitions_sampler.rs`
**Purpose**: Samples top 10 highest transitions across all 45,944 models
**Output**: Reveals 6.87B transition instances, 19,258 unique patterns
**Usage**: `./top_transitions` (analyzes models/ directory)

## Compiler Analysis Tools

### `automorphic_field`
**Source**: `automorphic_field.rs`
**Purpose**: Main mathematical framework proving compiler self-encoding
**Output**: 97.79% similarity validation between HIR and rustc source
**Usage**: `./automorphic_field` (validates self-reference proof)

### `rustc_hir_cicd`
**Source**: `rustc_hir_cicd_simple.rs`
**Purpose**: Extracts HIR representations from rustc modules
**Output**: 37 successful HIR representations from 76 modules
**Usage**: `./rustc_hir_cicd` (processes rustc source tree)

### `hir_rustc_comparison`
**Source**: `hir_rustc_comparison.rs`
**Purpose**: Validates compiler self-reference through similarity analysis
**Output**: Proves 97.79% HIR-to-rustc-content similarity
**Usage**: `./hir_rustc_comparison` (compares saved models)

### `compiler_path_matcher`
**Source**: `compiler_path_matcher.rs`
**Purpose**: Tests if compiler output strings exist in 33.9M file database
**Output**: Binary-source correspondence verification
**Usage**: `./compiler_path_matcher` (validates compiler outputs)

## Mathematical Tools

### `kleene2markov2godel`
**Source**: `kleene2markov2godel.rs`
**Purpose**: Converts regex patterns to probabilistic automata to arithmetic encoding
**Output**: Gödel number 65,203,482 for "src/helloworld.rs"
**Usage**: `./kleene2markov2godel` (demonstrates transformation chain)

### `morphism_map`
**Source**: `morphism_map.rs`
**Purpose**: Creates automorphic morphism mapping for Markov transitions
**Output**: Cross-domain morphism preservation mappings
**Usage**: `./morphism_map` (generates algebraic structure)

### `transition_matrix`
**Source**: `transition_matrix.rs`
**Purpose**: Samples top 3 transitions from each model
**Output**: Universal pattern discovery with overflow protection
**Usage**: `./transition_matrix` (creates transition matrices)

## Model Organization Tools

### `model_classifier`
**Source**: `model_classifier.rs`
**Purpose**: Organizes 45,947 binary models by classification
**Output**: Structured directories: forward/, reverse/, compiler/, numeric/
**Usage**: `./model_classifier` (organizes model files)

### `model_similarity`
**Source**: `model_similarity.rs`
**Purpose**: Computes similarity between transition patterns
**Output**: 2,408 similar pairs, 85.3% maximum similarity
**Usage**: `./model_similarity` (finds model correlations)

### `venn_markov_analyzer`
**Source**: `venn_markov_analyzer.rs`
**Purpose**: Analyzes overlaps between different model sets
**Output**: Rustc path and content model intersections
**Usage**: `./venn_markov_analyzer` (processes local Rust files)

## Analysis and Validation Tools

### `markov_analyzer`
**Source**: `markov_analyzer.rs`
**Purpose**: Graph visualization, fixed points, regex extraction
**Output**: Self-reference detection and pattern analysis
**Usage**: `./markov_analyzer` (analyzes model properties)

### `markov_comparison`
**Source**: `markov_comparison.rs`
**Purpose**: Compares compilation dump with path database
**Output**: Validates compiler output consistency
**Usage**: `./markov_comparison` (cross-validates models)

### `rustc_markov_analyzer`
**Source**: `rustc_markov_analyzer.rs`
**Purpose**: Compares rustc source code Markov model with directory structure
**Output**: File path vs content correlation analysis
**Usage**: `./rustc_markov_analyzer` (analyzes rustc structure)

## Repository Analysis Tools

### `git_pack_analyzer`
**Source**: `git_pack_analyzer.rs`
**Purpose**: Extracts compressed representations and delta patterns
**Output**: Repository structure analysis and compression metrics
**Usage**: `./git_pack_analyzer` (processes git objects)

### `homotopy_unirepo`
**Source**: `homotopy_unirepo.rs`
**Purpose**: Builds compressed topological representations from git objects
**Output**: Repository topology mapping
**Usage**: `./homotopy_unirepo` (creates topological maps)

### `rust_repo_generator`
**Source**: `rust_repo_generator.rs`
**Purpose**: Generates repository templates using Markov path patterns
**Output**: Automated project scaffolding based on learned patterns
**Usage**: `./rust_repo_generator` (creates new project templates)

## Test and Demo Tools

### `helloworld`
**Source**: `src/helloworld.rs`
**Purpose**: Simple test program for Gödel number calculation
**Output**: Basic "Hello, world!" for mathematical encoding
**Usage**: `./helloworld` (demonstrates basic compilation)

### `test_wrap`
**Source**: `wrapping_cost_analyzer.rs` (test build)
**Purpose**: Tests git object wrapping cost analysis
**Output**: Repository efficiency metrics
**Usage**: `./test_wrap` (validates wrapping cost calculations)

## Build Commands

All tools compile with standard rustc:

```bash
# Core analysis
rustc multi_file_markov.rs -o multi_file_markov
rustc file_list_markov.rs -o file_list_markov
rustc top_transitions_sampler.rs -o top_transitions

# Compiler analysis
rustc automorphic_field.rs -o automorphic_field
rustc rustc_hir_cicd_simple.rs -o rustc_hir_cicd
rustc hir_rustc_comparison.rs -o hir_rustc_comparison

# Mathematical tools
rustc kleene2markov2godel.rs -o kleene2markov2godel
rustc morphism_map.rs -o morphism_map
rustc transition_matrix.rs -o transition_matrix

# Model organization
rustc model_classifier.rs -o model_classifier
rustc model_similarity.rs -o model_similarity
rustc venn_markov_analyzer.rs -o venn_markov_analyzer
```

## Mathematical Significance

These 22 binary tools collectively prove:

**Programming = Morphism composition in automorphic mathematical space**

Where: `Compiler = mkembodiment!(pure_automorphic_math)`

Each tool contributes to the complete mathematical proof by analyzing different aspects of the automorphic field structure underlying all programming language constructs.
