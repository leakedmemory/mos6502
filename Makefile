all: test

test:
	@echo "Testing..."
	@go test ./... -v

lint:
	@golangci-lint run --fix

.PHONY: all test lint
