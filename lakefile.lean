import Lake
open Lake DSL

package «meta-introspector-compression» where
  version := v!"0.1.0"
  keywords := #["compression", "unity", "kleene", "monster-group"]
  description := "Formal proof of logarithmic compression to Unity"

require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git"

@[default_target]
lean_lib «MetaIntrospectorCompression» where
  -- add any library configuration options here
