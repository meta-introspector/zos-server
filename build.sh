#!/bin/bash
cargo build 2> build.log || grep error build.log | sort | uniq -c | sort -rn | head
