#!/bin/bash

echo "Creating queue"
awslocal sqs create-queue --queue-name test-queue

echo "Creating bucket"
awslocal s3api create-bucket --bucket test-bucket
