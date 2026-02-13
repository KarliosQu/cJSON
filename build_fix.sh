#!/bin/bash
# Try to build with minimal version
cargo build --lib 2>&1 | head -50