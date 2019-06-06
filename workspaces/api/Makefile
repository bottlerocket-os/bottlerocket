CLIENT_CRATE_VERSION = 0.1.0

# comment
.generate-client: openapi.yaml $(wildcard client-templates/*) $(wildcard client-templates/*/*)
	@docker run --rm \
		-v ${PWD}:/local \
		openapitools/openapi-generator-cli \
		generate \
		-i /local/openapi.yaml \
		-g rust \
		--library reqwest \
		-o /local/apiclient \
		-t /local/client-templates \
		--type-mappings Settings=model::Settings,Services=model::Services,ConfigurationFiles=model::ConfigurationFiles \
		--additional-properties=packageName=apiclient,packageVersion=$(CLIENT_CRATE_VERSION),hideGenerationTimestamp=false
	touch .generate-client

client: .generate-client
