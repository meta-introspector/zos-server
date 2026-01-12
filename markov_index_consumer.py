#!/usr/bin/env python3
"""
Markov Index Consumer - Builds Markov models from ~/nix/index
Demonstrates: Index -> Contents -> Structure -> Binary -> Runtime flow
"""

import json
import re
from collections import defaultdict, Counter
from pathlib import Path
import sys

class MarkovIndexConsumer:
    def __init__(self, index_path="~/nix/index"):
        self.index_path = Path(index_path).expanduser()
        self.transitions = defaultdict(Counter)
        self.word_transitions = defaultdict(Counter)
        self.structure_patterns = defaultdict(Counter)

    def analyze_rust_files(self):
        """Analyze allrs.txt for Rust code patterns"""
        allrs_path = self.index_path / "allrs.txt"
        if not allrs_path.exists():
            print(f"❌ {allrs_path} not found")
            return

        print(f"📊 Analyzing {allrs_path.stat().st_size / 1024 / 1024:.1f}MB of Rust files...")

        with open(allrs_path, 'r', encoding='utf-8', errors='ignore') as f:
            prev_token = None
            for line_num, line in enumerate(f):
                if line_num % 100000 == 0:
                    print(f"  Processed {line_num:,} lines...")

                # Extract Rust tokens
                tokens = re.findall(r'\b\w+\b|[{}();,]', line.strip())

                for token in tokens:
                    if prev_token:
                        self.transitions[prev_token][token] += 1
                    prev_token = token

                # Structure patterns
                if 'fn ' in line:
                    self.structure_patterns['function'][line.strip()[:50]] += 1
                elif 'struct ' in line:
                    self.structure_patterns['struct'][line.strip()[:50]] += 1
                elif 'impl ' in line:
                    self.structure_patterns['impl'][line.strip()[:50]] += 1

    def analyze_github_metadata(self):
        """Analyze GitHub metadata for repository patterns"""
        for json_file in self.index_path.glob("*.json"):
            print(f"📋 Analyzing {json_file.name}...")
            try:
                with open(json_file, 'r') as f:
                    data = json.load(f)

                if isinstance(data, list):
                    for repo in data:
                        if isinstance(repo, dict):
                            # Language transitions
                            lang = repo.get('language', 'unknown')
                            name = repo.get('name', '')
                            self.word_transitions[lang][name[:20]] += 1

            except Exception as e:
                print(f"⚠️  Error reading {json_file}: {e}")

    def build_markov_chains(self):
        """Build Markov chains from collected data"""
        print("\n🔗 Building Markov chains...")

        # Token transitions
        print(f"  Token transitions: {len(self.transitions):,}")
        print(f"  Word transitions: {len(self.word_transitions):,}")
        print(f"  Structure patterns: {len(self.structure_patterns):,}")

        return {
            'token_transitions': dict(self.transitions),
            'word_transitions': dict(self.word_transitions),
            'structure_patterns': dict(self.structure_patterns)
        }

    def demonstrate_flow(self, chains):
        """Demonstrate Index -> Contents -> Structure -> Binary -> Runtime"""
        print("\n🌊 MARKOV FLOW DEMONSTRATION:")
        print("=" * 60)

        # 1. Index informs Contents
        print("1️⃣  INDEX → CONTENTS:")
        top_tokens = sorted(self.transitions.items(),
                          key=lambda x: sum(x[1].values()), reverse=True)[:5]
        for token, transitions in top_tokens:
            most_common = transitions.most_common(3)
            print(f"   '{token}' → {most_common}")

        # 2. Contents inform Structure
        print("\n2️⃣  CONTENTS → STRUCTURE:")
        for pattern_type, patterns in self.structure_patterns.items():
            top_patterns = patterns.most_common(3)
            print(f"   {pattern_type}: {top_patterns}")

        # 3. Structure informs Binary
        print("\n3️⃣  STRUCTURE → BINARY:")
        print("   Functions → Compiled symbols")
        print("   Structs → Memory layout")
        print("   Impls → Virtual tables")

        # 4. Binary informs Runtime
        print("\n4️⃣  BINARY → RUNTIME:")
        print("   Symbols → Dynamic loading")
        print("   Memory layout → Cache patterns")
        print("   Virtual tables → Dispatch")

    def generate_predictions(self, chains, seed_token="fn"):
        """Generate predictions using Markov model"""
        print(f"\n🔮 MARKOV PREDICTIONS (seed: '{seed_token}'):")

        current = seed_token
        prediction = [current]

        for _ in range(10):
            if current in self.transitions:
                next_tokens = self.transitions[current]
                if next_tokens:
                    # Weighted random selection
                    most_likely = next_tokens.most_common(1)[0][0]
                    prediction.append(most_likely)
                    current = most_likely
                else:
                    break
            else:
                break

        print(f"   Predicted sequence: {' '.join(prediction)}")

def main():
    consumer = MarkovIndexConsumer()

    print("🧠 MARKOV INDEX CONSUMER")
    print("=" * 40)

    # Analyze the index
    consumer.analyze_rust_files()
    consumer.analyze_github_metadata()

    # Build chains
    chains = consumer.build_markov_chains()

    # Demonstrate the flow
    consumer.demonstrate_flow(chains)

    # Generate predictions
    consumer.generate_predictions(chains, "fn")
    consumer.generate_predictions(chains, "struct")

    print("\n✅ Markov analysis complete!")
    print("🎯 Index → Contents → Structure → Binary → Runtime flow demonstrated")

if __name__ == "__main__":
    main()
