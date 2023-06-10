#!/usr/bin/env bash

now=$(date +%Y-%m-%dT%H:%M:%SZ)

echo "---
title: $1
date: $(date +%Y-%m-%d)
---" > til/$now.md
