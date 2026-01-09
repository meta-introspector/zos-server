#!/usr/bin/env python3
"""
Intelligent Lattice Slicer - Creates optimal work batches based on:
1. Performance data from previous runs
2. Available worker capacity
3. Network coordination via libp2p
4. GitHub Actions runner limits
"""

import json
import os
import sys
import time
import statistics
from datetime import datetime, timedelta
from typing import List, Dict, Any, Tuple

class LatticePerformanceAnalyzer:
    def __init__(self):
        self.results_dir = "lattice_test_results"
        self.archives_dir = "archives"
        self.perf_data = {}

    def analyze_previous_runs(self) -> Dict[str, Any]:
        """Analyze performance data from previous test runs"""
        perf_stats = {
            'avg_build_time': 60,  # Default 60s
            'success_rate': 0.5,   # Default 50%
            'worker_efficiency': 1.0,
            'coordinate_performance': {},
            'feature_performance': {},
            'total_tests_run': 0
        }

        # Scan archives for performance data
        if os.path.exists(self.archives_dir):
            for archive_dir in os.listdir(self.archives_dir):
                if archive_dir.startswith('lattice_results_'):
                    self._analyze_archive(os.path.join(self.archives_dir, archive_dir), perf_stats)

        return perf_stats

    def _analyze_archive(self, archive_path: str, perf_stats: Dict):
        """Analyze a single archive for performance metrics"""
        try:
            # Look for timing data in log files
            build_times = []

            for filename in os.listdir(archive_path):
                if filename.endswith('.log'):
                    log_path = os.path.join(archive_path, filename)
                    build_time = self._extract_build_time(log_path)
                    if build_time:
                        build_times.append(build_time)

                        # Extract coordinate and feature info
                        parts = filename.replace('.log', '').split('_')
                        if len(parts) >= 3:
                            coord = parts[0]
                            features = parts[1]

                            # Track performance by coordinate
                            if coord not in perf_stats['coordinate_performance']:
                                perf_stats['coordinate_performance'][coord] = []
                            perf_stats['coordinate_performance'][coord].append(build_time)

                            # Track performance by feature
                            if features not in perf_stats['feature_performance']:
                                perf_stats['feature_performance'][features] = []
                            perf_stats['feature_performance'][features].append(build_time)

            if build_times:
                perf_stats['avg_build_time'] = statistics.mean(build_times)
                perf_stats['total_tests_run'] += len(build_times)

        except Exception as e:
            print(f"Warning: Could not analyze archive {archive_path}: {e}")

    def _extract_build_time(self, log_path: str) -> float:
        """Extract build time from log file"""
        try:
            with open(log_path, 'r') as f:
                content = f.read()
                # Look for "Finished" line with timing
                for line in content.split('\n'):
                    if 'Finished' in line and 'target(s) in' in line:
                        # Extract time like "1.23s" or "2m 34s"
                        parts = line.split('in ')[-1].split(' ')[0]
                        if 's' in parts:
                            return float(parts.replace('s', ''))
                        elif 'm' in parts:
                            # Handle "2m" format
                            return float(parts.replace('m', '')) * 60
            return None
        except:
            return None

class LibP2PCoordinator:
    """Simulates libp2p coordination for distributed testing"""

    def __init__(self):
        self.node_id = os.environ.get('GITHUB_RUN_ID', 'local')
        self.network_size = self._estimate_network_size()

    def _estimate_network_size(self) -> int:
        """Estimate available worker nodes in the network"""
        # In real implementation, this would query libp2p network
        # For now, simulate based on GitHub Actions capacity

        if os.environ.get('GITHUB_ACTIONS'):
            # GitHub Actions environment - estimate concurrent runners
            return min(20, max(1, int(os.environ.get('GITHUB_RUN_NUMBER', '1')) % 10))
        else:
            return 1  # Local testing

    def get_worker_capacity(self) -> Dict[str, Any]:
        """Get current worker capacity and load"""
        return {
            'total_workers': self.network_size,
            'available_workers': max(1, self.network_size - 2),  # Reserve 2 for coordination
            'estimated_parallel_capacity': min(5, self.network_size),
            'network_latency': 0.1,  # Simulated
            'coordination_overhead': 0.05
        }

class IntelligentSlicer:
    def __init__(self):
        self.analyzer = LatticePerformanceAnalyzer()
        self.coordinator = LibP2PCoordinator()

    def create_optimal_slices(self, job_type: str, slice_pattern: str, batch_size: int) -> List[Dict]:
        """Create optimal work slices based on performance and capacity"""

        # Analyze previous performance
        perf_stats = self.analyzer.analyze_previous_runs()
        worker_capacity = self.coordinator.get_worker_capacity()

        print(f"📊 Performance Analysis:")
        print(f"  Average build time: {perf_stats['avg_build_time']:.1f}s")
        print(f"  Success rate: {perf_stats['success_rate']*100:.1f}%")
        print(f"  Total tests run: {perf_stats['total_tests_run']}")

        print(f"🌐 Network Capacity:")
        print(f"  Available workers: {worker_capacity['available_workers']}")
        print(f"  Parallel capacity: {worker_capacity['estimated_parallel_capacity']}")

        # Calculate optimal slice size
        optimal_slice_size = self._calculate_optimal_slice_size(perf_stats, worker_capacity, batch_size)

        # Load lattice and create slices
        with open('rustc_flag_lattice.json') as f:
            lattice_data = json.load(f)

        # Get tested combinations
        tested = self._get_tested_combinations()

        # Filter based on job type and pattern
        work_items = self._filter_work_items(lattice_data, tested, job_type, slice_pattern)

        # Create intelligent slices
        slices = self._create_slices(work_items, optimal_slice_size, perf_stats)

        return slices

    def _calculate_optimal_slice_size(self, perf_stats: Dict, capacity: Dict, requested_batch: int) -> int:
        """Calculate optimal slice size based on performance and capacity"""

        avg_time = perf_stats['avg_build_time']
        success_rate = perf_stats['success_rate']
        workers = capacity['available_workers']

        # Target: Complete work within 10 minutes per slice
        target_time = 600  # 10 minutes

        # Account for failures (need to retry)
        effective_time_per_test = avg_time / success_rate

        # Calculate how many tests can fit in target time
        tests_per_slice = max(1, int(target_time / effective_time_per_test))

        # Adjust for worker capacity
        optimal_size = min(tests_per_slice, requested_batch * workers)

        print(f"🧮 Slice Calculation:")
        print(f"  Target time per slice: {target_time}s")
        print(f"  Effective time per test: {effective_time_per_test:.1f}s")
        print(f"  Optimal slice size: {optimal_size}")

        return optimal_size

    def _get_tested_combinations(self) -> set:
        """Get already tested combinations"""
        tested = set()
        if os.path.exists(self.analyzer.results_dir):
            for filename in os.listdir(self.analyzer.results_dir):
                if filename.endswith('.result'):
                    # Parse: L0.0.0.0_default_dev.result
                    key = filename.replace('.result', '')
                    tested.add(key)
        return tested

    def _filter_work_items(self, lattice_data: List, tested: set, job_type: str, pattern: str) -> List:
        """Filter work items based on job type and pattern"""

        if job_type == 'gap-fill':
            # Load previous gap report
            if os.path.exists('gap_report.json'):
                with open('gap_report.json') as f:
                    gap_report = json.load(f)
                return gap_report.get('gaps', [])
            else:
                print("❌ No gap report found for gap-fill job")
                return []

        # Filter untested items
        untested = []
        for item in lattice_data:
            key = f"{item['coordinate']}_{item['features']}_{item['profile']}"
            if key not in tested:
                # Apply pattern filter
                if pattern == 'auto' or self._matches_pattern(item['coordinate'], pattern):
                    untested.append(item)

        return untested

    def _matches_pattern(self, coordinate: str, pattern: str) -> bool:
        """Check if coordinate matches pattern (e.g., L0.*, *.*.0.*)"""
        import fnmatch
        return fnmatch.fnmatch(coordinate, pattern)

    def _create_slices(self, work_items: List, slice_size: int, perf_stats: Dict) -> List[Dict]:
        """Create intelligent slices prioritized by performance data"""

        # Sort work items by predicted performance (fastest first)
        work_items.sort(key=lambda x: self._predict_build_time(x, perf_stats))

        slices = []
        for i in range(0, len(work_items), slice_size):
            slice_items = work_items[i:i + slice_size]

            slice_info = {
                'slice_id': f"slice_{len(slices)}",
                'items': slice_items,
                'estimated_time': sum(self._predict_build_time(item, perf_stats) for item in slice_items),
                'priority': self._calculate_priority(slice_items, perf_stats),
                'worker_assignment': f"worker_{len(slices) % self.coordinator.network_size}"
            }

            slices.append(slice_info)

        return slices

    def _predict_build_time(self, item: Dict, perf_stats: Dict) -> float:
        """Predict build time for a work item"""
        coord = item['coordinate']
        features = item['features']

        # Use historical data if available
        if coord in perf_stats['coordinate_performance']:
            coord_avg = statistics.mean(perf_stats['coordinate_performance'][coord])
        else:
            coord_avg = perf_stats['avg_build_time']

        if features in perf_stats['feature_performance']:
            feature_avg = statistics.mean(perf_stats['feature_performance'][features])
        else:
            feature_avg = perf_stats['avg_build_time']

        # Weighted average
        return (coord_avg * 0.6 + feature_avg * 0.4)

    def _calculate_priority(self, slice_items: List, perf_stats: Dict) -> float:
        """Calculate priority score for a slice (higher = more important)"""
        # Prioritize slices with:
        # 1. Faster expected completion
        # 2. Higher success probability
        # 3. More diverse coordinate coverage

        avg_time = sum(self._predict_build_time(item, perf_stats) for item in slice_items) / len(slice_items)

        # Diversity score (unique coordinates)
        unique_coords = len(set(item['coordinate'].split('.')[0] for item in slice_items))

        # Priority = speed + diversity (lower time = higher priority)
        priority = (1.0 / avg_time) * 100 + unique_coords * 10

        return priority

def main():
    if len(sys.argv) < 2:
        print("Usage: intelligent_slicer.py <job_type> [slice_pattern] [batch_size]")
        sys.exit(1)

    job_type = sys.argv[1]
    slice_pattern = sys.argv[2] if len(sys.argv) > 2 else 'auto'
    batch_size = int(sys.argv[3]) if len(sys.argv) > 3 else 10

    slicer = IntelligentSlicer()
    slices = slicer.create_optimal_slices(job_type, slice_pattern, batch_size)

    print(f"\n🎯 Created {len(slices)} intelligent slices:")

    for i, slice_info in enumerate(slices):
        print(f"\n📦 Slice {i}: {slice_info['slice_id']}")
        print(f"  Items: {len(slice_info['items'])}")
        print(f"  Estimated time: {slice_info['estimated_time']:.1f}s")
        print(f"  Priority: {slice_info['priority']:.1f}")
        print(f"  Worker: {slice_info['worker_assignment']}")

        # Output first slice items for execution
        if i == 0:  # Execute first slice
            print(f"\n🚀 Executing slice {i}:")
            for item in slice_info['items']:
                print(f"{item['coordinate']}|{item['rustflags']}|{item['features']}|{item['profile']}")

    # Save slice plan for coordination
    with open('slice_plan.json', 'w') as f:
        json.dump({
            'timestamp': datetime.now().isoformat(),
            'job_type': job_type,
            'slice_pattern': slice_pattern,
            'total_slices': len(slices),
            'slices': slices
        }, f, indent=2)

    print(f"\n💾 Slice plan saved to slice_plan.json")

if __name__ == '__main__':
    main()
