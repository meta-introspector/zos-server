#!/usr/bin/env python3
import itertools
import json
from typing import List, Dict, Any

class RustcFlagLattice:
    def __init__(self):
        # Core optimization flags
        self.opt_levels = ['0', '1', '2', '3', 's', 'z']

        # Debug info levels
        self.debug_levels = ['0', '1', '2']

        # Common codegen flags
        self.codegen_flags = {
            'panic': ['unwind', 'abort'],
            'lto': ['off', 'thin', 'fat'],
            'target-cpu': ['native', 'generic'],
            'overflow-checks': ['on', 'off'],
            'debug-assertions': ['on', 'off'],
        }

        # Link-time flags
        self.link_flags = {
            'prefer-dynamic': [True, False],
            'static-crt': [True, False],
        }

        # Feature flags for ZOS Server
        self.zos_features = [
            'default',
            'all-plugins',
            'self-build',
            'core-only',
            'extra-plugins',
            'notebooklm',
            'dynamic-loading'
        ]

    def generate_lattice_coordinates(self, max_combinations=50):
        """Generate lattice coordinates for flag combinations"""
        combinations = []

        # Generate systematic combinations
        for i, opt in enumerate(self.opt_levels[:3]):  # Limit to first 3 opt levels
            for j, debug in enumerate(self.debug_levels):
                for k, panic in enumerate(self.codegen_flags['panic']):
                    for l, lto in enumerate(self.codegen_flags['lto']):
                        # Create lattice coordinate
                        coord = f"L{i}.{j}.{k}.{l}"

                        # Build flag combination
                        flags = {
                            'opt_level': opt,
                            'debug_info': debug,
                            'panic': panic,
                            'lto': lto,
                            'coordinate': coord
                        }

                        combinations.append(flags)

                        if len(combinations) >= max_combinations:
                            return combinations

        return combinations

    def generate_rustflags(self, combo: Dict[str, Any]) -> str:
        """Convert combination to RUSTFLAGS string"""
        flags = []

        flags.append(f"-C opt-level={combo['opt_level']}")
        flags.append(f"-C debuginfo={combo['debug_info']}")
        flags.append(f"-C panic={combo['panic']}")
        flags.append(f"-C lto={combo['lto']}")

        return " ".join(flags)

    def generate_test_matrix(self):
        """Generate complete test matrix"""
        lattice = self.generate_lattice_coordinates()

        matrix = []
        for combo in lattice:
            for feature in self.zos_features[:5]:  # Limit features
                test_case = {
                    'coordinate': combo['coordinate'],
                    'rustflags': self.generate_rustflags(combo),
                    'features': feature,
                    'profile': 'dev' if combo['opt_level'] in ['0', '1'] else 'release'
                }
                matrix.append(test_case)

        return matrix

# Generate the lattice
generator = RustcFlagLattice()
test_matrix = generator.generate_test_matrix()

print(f"Generated {len(test_matrix)} test combinations")
print("\nSample combinations:")
for i, test in enumerate(test_matrix[:10]):
    print(f"{test['coordinate']}: {test['rustflags']} --features {test['features']}")

# Save to JSON
with open('rustc_flag_lattice.json', 'w') as f:
    json.dump(test_matrix, f, indent=2)

print(f"\nFull lattice saved to rustc_flag_lattice.json")
