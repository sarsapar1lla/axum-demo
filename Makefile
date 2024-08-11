.PHONY: upload-file
upload-file:
	docker exec -it localstack awslocal s3 cp /tmp/events/event.json s3://test-bucket/input/1234.json
