#!/bin/bash

cargo build --release
strip target/release/INI_parser 
upx -5 target/release/INI_parser
cp target/release/INI_parser /lustre/database/genomes/INI/
