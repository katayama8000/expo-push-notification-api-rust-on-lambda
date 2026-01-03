#!/bin/bash
# Load AWS credentials from .env file
# Usage: source setup_deploy_env.sh (or . setup_deploy_env.sh)

if [ -f .env ]; then
  export $(grep -v '^#' .env | xargs)
  echo "✓ AWS credentials loaded from .env"
else
  echo "Error: .env file not found"
  return 1
fi
