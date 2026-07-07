OPENAI_OPENAPI_VERSION ?= 2.0.0

.PHONY: update

update:
	git submodule update --init --recursive
	git -C vendor/openai-openapi fetch --tags
	git -C vendor/openai-openapi checkout $(OPENAI_OPENAPI_VERSION)
	sudo openapi-generator-cli generate -i vendor/openai-openapi/openapi.yaml \
		-g rust-axum -o crates/openai-api-gen \
		--enable-post-process-file \
		--additional-properties=packageName=openai-api-gen,packageVersion=$(OPENAI_OPENAPI_VERSION),generateAliasAsModel=true
