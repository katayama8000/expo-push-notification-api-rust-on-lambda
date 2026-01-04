#!/bin/bash

# run build.sh
source ./build.sh

# run AWS credentials
source ./setup_deploy_env.sh

echo "AWS credentials loaded."

# Deploy the lambda function
cargo lambda deploy

echo "Deployment completed successfully!"