//! The ssm module owns the getting and setting of parameters in SSM.

use super::{SsmKey, SsmParameters};
use aws_sdk_ssm::error::{ProvideErrorMetadata, SdkError};
use aws_sdk_ssm::operation::{
    get_parameters::{GetParametersError, GetParametersOutput},
    put_parameter::{PutParameterError, PutParameterOutput},
};
use aws_sdk_ssm::{config::Region, types::ParameterType, Client as SsmClient};
use futures::future::{join, ready};
use futures::stream::{self, FuturesUnordered, StreamExt};
use log::{debug, error, info, trace, warn};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

/// Fetches the values of the given SSM keys using the given clients
// TODO: We can batch GET requests so throttling is less likely here, but if we need to handle
// hundreds of parameters for a given build, we could use the throttling logic from
// `set_parameters`
pub(crate) async fn get_parameters<K>(
    requested: &[K],
    clients: &HashMap<Region, SsmClient>,
) -> Result<SsmParameters>
where
    K: AsRef<SsmKey>,
{
    // Build requests for parameters; we have to request with a regional client so we split them by
    // region
    let mut requests = Vec::with_capacity(requested.len());
    let mut regional_names: HashMap<Region, Vec<String>> = HashMap::new();
    for key in requested {
        let SsmKey { region, name } = key.as_ref();
        regional_names
            .entry(region.clone())
            .or_default()
            .push(name.clone());
    }
    for (region, names) in regional_names {
        // At most 10 parameters can be requested at a time.
        for names_chunk in names.chunks(10) {
            trace!("Requesting {:?} in {}", names_chunk, region);
            let ssm_client = &clients[&region];
            let len = names_chunk.len();
            let get_future = ssm_client
                .get_parameters()
                .set_names((!names_chunk.is_empty()).then_some(names_chunk.to_vec().clone()))
                .send();

            // Store the region so we can include it in errors and the output map
            let info_future = ready((region.clone(), len));
            requests.push(join(info_future, get_future));
        }
    }

    // Send requests in parallel and wait for responses, collecting results into a list.
    let request_stream = stream::iter(requests).buffer_unordered(4);
    #[allow(clippy::type_complexity)]
    let responses: Vec<(
        (Region, usize),
        std::result::Result<GetParametersOutput, SdkError<GetParametersError>>,
    )> = request_stream.collect().await;

    // If you're checking parameters in a region you haven't pushed to before, you can get an
    // error here about the parameter's namespace being new.  We want to treat these as new
    // parameters rather than failing.  Unfortunately, we don't know which parameter in the region
    // was considered new, but we expect that most people are publishing all of their parameters
    // under the same namespace, so treating the whole region as new is OK.  We use this just to
    // warn the user.
    let mut new_regions = HashSet::new();

    // For each existing parameter in the response, get the name and value for our output map.
    let mut parameters = HashMap::with_capacity(requested.len());
    for ((region, expected_len), response) in responses {
        // Get the image description, ensuring we only have one.
        let response = match response {
            Ok(response) => response,
            Err(e) => {
                // Note: there's no structured error type for this so we have to string match.
                if e.to_string().contains("is not a valid namespace") {
                    new_regions.insert(region.clone());
                    continue;
                } else {
                    return Err(e).context(error::GetParametersSnafu {
                        region: region.as_ref(),
                    });
                }
            }
        };

        // Check that we received a response including every parameter
        // Note: response.invalid_parameters includes both new parameters and ill-formatted
        // parameter names...
        let valid_count = response.parameters.as_ref().map(|v| v.len()).unwrap_or(0);
        let invalid_count = response.invalid_parameters.map(|v| v.len()).unwrap_or(0);
        let total_count = valid_count + invalid_count;
        ensure!(
            total_count == expected_len,
            error::MissingInResponseSnafu {
                region: region.as_ref(),
                request_type: "GetParameters",
                missing: format!(
                    "parameters - got {}, expected {}",
                    total_count, expected_len
                ),
            }
        );

        // Save the successful parameters
        if let Some(valid_parameters) = response.parameters {
            if !valid_parameters.is_empty() {
                for parameter in valid_parameters {
                    let name = parameter.name.context(error::MissingInResponseSnafu {
                        region: region.as_ref(),
                        request_type: "GetParameters",
                        missing: "parameter name",
                    })?;
                    let value = parameter.value.context(error::MissingInResponseSnafu {
                        region: region.as_ref(),
                        request_type: "GetParameters",
                        missing: format!("value for parameter {}", name),
                    })?;
                    parameters.insert(SsmKey::new(region.clone(), name), value);
                }
            }
        }
    }

    for region in new_regions {
        warn!(
            "Invalid namespace in {}, this is OK for the first publish in a region",
            region
        );
    }

    Ok(parameters)
}

/// Fetches all SSM parameters under a given prefix using the given clients
pub(crate) async fn get_parameters_by_prefix<'a>(
    clients: &'a HashMap<Region, SsmClient>,
    ssm_prefix: &str,
) -> HashMap<&'a Region, Result<SsmParameters>> {
    // Build requests for parameters; we have to request with a regional client so we split them by
    // region
    let mut requests = Vec::with_capacity(clients.len());
    for region in clients.keys() {
        trace!("Requesting parameters in {}", region);
        let ssm_client: &SsmClient = &clients[region];
        let get_future = get_parameters_by_prefix_in_region(region, ssm_client, ssm_prefix);

        requests.push(join(ready(region), get_future));
    }

    // Send requests in parallel and wait for responses, collecting results into a list.
    requests
        .into_iter()
        .collect::<FuturesUnordered<_>>()
        .collect()
        .await
}

/// Fetches all SSM parameters under a given prefix in a single region
pub(crate) async fn get_parameters_by_prefix_in_region(
    region: &Region,
    client: &SsmClient,
    ssm_prefix: &str,
) -> Result<SsmParameters> {
    info!("Retrieving SSM parameters in {}", region.to_string());
    let mut parameters = HashMap::new();

    // Send the request
    let mut get_future = client
        .get_parameters_by_path()
        .path(ssm_prefix)
        .recursive(true)
        .into_paginator()
        .send();

    // Iterate over the retrieved parameters
    while let Some(page) = get_future.next().await {
        let retrieved_parameters = page
            .context(error::GetParametersByPathSnafu {
                path: ssm_prefix,
                region: region.to_string(),
            })?
            .parameters()
            .unwrap_or_default()
            .to_owned();
        for parameter in retrieved_parameters {
            // Insert a new key-value pair into the map, with the key containing region and parameter name
            // and the value containing the parameter value
            parameters.insert(
                SsmKey::new(
                    region.to_owned(),
                    parameter
                        .name()
                        .ok_or(error::Error::MissingField {
                            region: region.to_string(),
                            field: "name".to_string(),
                        })?
                        .to_owned(),
                ),
                parameter
                    .value()
                    .ok_or(error::Error::MissingField {
                        region: region.to_string(),
                        field: "value".to_string(),
                    })?
                    .to_owned(),
            );
        }
    }
    info!(
        "SSM parameters in {} have been retrieved",
        region.to_string()
    );
    Ok(parameters)
}

/// Sets the values of the given SSM keys using the given clients
pub(crate) async fn set_parameters(
    parameters_to_set: &SsmParameters,
    ssm_clients: &HashMap<Region, SsmClient>,
) -> Result<()> {
    // Start with a small delay between requests, and increase if we get throttled.
    let mut request_interval = Duration::from_millis(100);
    let max_interval = Duration::from_millis(1600);
    let interval_factor = 2;
    let mut should_increase_interval = false;

    // We run all requests in a batch, and any failed requests are added to the next batch for
    // retry
    let mut failed_parameters: HashMap<Region, Vec<(String, String)>> = HashMap::new();
    let max_failures = 5;

    /// Stores the values we need to be able to retry requests
    struct RequestContext<'a> {
        region: &'a Region,
        name: &'a str,
        value: &'a str,
        failures: u8,
    }

    // Create the initial request contexts
    let mut contexts = Vec::new();
    for (SsmKey { region, name }, value) in parameters_to_set {
        contexts.push(RequestContext {
            region,
            name,
            value,
            failures: 0,
        });
    }
    let total_count = contexts.len();

    // We drain requests out of the contexts list and put them back if we need to retry; we do this
    // until all requests have succeeded or we've hit the max failures
    while !contexts.is_empty() {
        debug!("Starting {} SSM put requests", contexts.len());

        if should_increase_interval {
            request_interval *= interval_factor;
            warn!(
                "Requests were throttled, increasing interval to {:?}",
                request_interval
            );
        }
        should_increase_interval = false;

        ensure!(
            request_interval <= max_interval,
            error::ThrottledSnafu { max_interval }
        );

        // Build requests for parameters.  We need to group them by region so we can run each
        // region in parallel.  Each region's stream will be throttled to run one request per
        // request_interval.
        let mut regional_requests = HashMap::new();
        // Remove contexts from the list with drain; they get added back in if we retry the
        // request.
        for context in contexts.drain(..) {
            let ssm_client = &ssm_clients[context.region];

            let put_future = ssm_client
                .put_parameter()
                .set_name(Some(context.name.to_string()))
                .set_value(Some(context.value.to_string()))
                .set_overwrite(Some(true))
                .set_type(Some(ParameterType::String))
                .send();

            let regional_list = regional_requests
                .entry(context.region)
                .or_insert_with(Vec::new);
            // Store the context so we can retry as needed
            regional_list.push(join(ready(context), put_future));
        }

        // Create a throttled stream per region; throttling applies per region.  (Request futures
        // are already regional, by virtue of being created with a regional client, so we don't
        // need the region again here.)
        let mut throttled_streams = Vec::new();
        for (_region, request_list) in regional_requests {
            throttled_streams.push(Box::pin(tokio_stream::StreamExt::throttle(
                stream::iter(request_list),
                request_interval,
            )));
        }

        // Run all regions in parallel and wait for responses.
        let parallel_requests = stream::select_all(throttled_streams).buffer_unordered(4);
        let responses: Vec<(
            RequestContext<'_>,
            std::result::Result<PutParameterOutput, SdkError<PutParameterError>>,
        )> = parallel_requests.collect().await;

        // For each error response, check if we should retry or bail.
        for (context, response) in responses {
            if let Err(e) = response {
                // Throttling errors are not currently surfaced in AWS SDK Rust, doing a string match is best we can do
                let error_type = e
                    .into_service_error()
                    .code()
                    .unwrap_or("unknown")
                    .to_owned();
                if error_type.contains("ThrottlingException") {
                    // We only want to increase the interval once per loop, not once per error,
                    // because when you get throttled you're likely to get a bunch of throttling
                    // errors at once.
                    should_increase_interval = true;
                    // Retry the request without increasing the failure counter; the request didn't
                    // fail, a throttle means we couldn't even make the request.
                    contexts.push(context);
                // -1 so we don't try again next loop; this keeps failure checking in one place
                } else if context.failures >= max_failures - 1 {
                    // Past max failures, store the failure for reporting, don't retry.
                    failed_parameters
                        .entry(context.region.clone())
                        .or_default()
                        .push((context.name.to_string(), error_type));
                } else {
                    // Increase failure counter and try again.
                    let context = RequestContext {
                        failures: context.failures + 1,
                        ..context
                    };
                    debug!(
                        "Request attempt {} of {} failed in {}: {}",
                        context.failures, max_failures, context.region, error_type
                    );
                    contexts.push(context);
                }
            }
        }
    }

    if !failed_parameters.is_empty() {
        for (region, failures) in &failed_parameters {
            for (parameter, error) in failures {
                error!("Failed to set {} in {}: {}", parameter, region, error);
            }
        }
        return error::SetParametersSnafu {
            failure_count: failed_parameters.len(),
            total_count,
        }
        .fail();
    }

    Ok(())
}

/// Fetch the given parameters, and ensure the live values match the given values
pub(crate) async fn validate_parameters(
    expected_parameters: &SsmParameters,
    ssm_clients: &HashMap<Region, SsmClient>,
) -> Result<()> {
    // Fetch the given parameter names
    let expected_parameter_names: Vec<&SsmKey> = expected_parameters.keys().collect();
    let updated_parameters = get_parameters(&expected_parameter_names, ssm_clients).await?;

    // Walk through and check each value
    let mut success = true;
    for (expected_key, expected_value) in expected_parameters {
        let SsmKey {
            region: expected_region,
            name: expected_name,
        } = expected_key;
        // All parameters should have a value, and it should match the given value, otherwise the
        // parameter wasn't updated / created.
        if let Some(updated_value) = updated_parameters.get(expected_key) {
            if updated_value != expected_value {
                error!("Failed to set {} in {}", expected_name, expected_region);
                success = false;
            }
        } else {
            error!(
                "{} in {} still doesn't exist",
                expected_name, expected_region
            );
            success = false;
        }
    }
    ensure!(success, error::ValidateParametersSnafu);

    Ok(())
}

pub(crate) mod error {
    use aws_sdk_ssm::error::SdkError;
    use aws_sdk_ssm::operation::{
        get_parameters::GetParametersError, get_parameters_by_path::GetParametersByPathError,
    };
    use snafu::Snafu;
    use std::error::Error as _;
    use std::time::Duration;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    #[allow(clippy::large_enum_variant)]
    pub enum Error {
        #[snafu(display("Failed to fetch SSM parameters in {}: {}", region, source.source().map(|x| x.to_string()).unwrap_or("unknown".to_string())))]
        GetParameters {
            region: String,
            source: SdkError<GetParametersError>,
        },

        #[snafu(display(
            "Failed to fetch SSM parameters by path {} in {}: {}",
            path,
            region,
            source
        ))]
        GetParametersByPath {
            path: String,
            region: String,
            source: SdkError<GetParametersByPathError>,
        },

        #[snafu(display("Missing field in parameter in {}: {}", region, field))]
        MissingField { region: String, field: String },

        #[snafu(display("Response to {} was missing {}", request_type, missing))]
        MissingInResponse {
            region: String,
            request_type: String,
            missing: String,
        },

        #[snafu(display(
            "Failed to set {} of {} parameters; see above",
            failure_count,
            total_count
        ))]
        SetParameters {
            failure_count: usize,
            total_count: usize,
        },

        #[snafu(display(
            "SSM requests throttled too many times, went beyond our max interval {:?}",
            max_interval
        ))]
        Throttled { max_interval: Duration },

        #[snafu(display("Failed to validate all changes; see above."))]
        ValidateParameters,
    }
}
pub(crate) use error::Error;
pub(crate) type Result<T> = std::result::Result<T, error::Error>;
