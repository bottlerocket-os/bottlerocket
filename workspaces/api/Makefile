CLIENT_CRATE_VERSION = 0.1.0

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

# This target generates the API client to the apiclient/ folder
# using templates contained in the client-templates/ folder.
# If changes are made to the templates, please increment the
# above CLIENT_CRATE_VERSION variable and run this target
# to re-generate the client. CLIENT_CRATE_VERSION is used as the
# apiclient crate version in Cargo.toml.
client: .generate-client
