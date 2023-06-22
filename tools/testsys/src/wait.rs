use crate::error::{self, Result};
use futures::future::join_all;
use log::{error, info, warn};
use snafu::{OptionExt, ResultExt};
use std::collections::HashMap;
use std::time::Duration;
use testsys_model::clients::{AllowNotFound, CrdClient};
use testsys_model::test_manager::TestManager;
use testsys_model::{
    Crd, CrdExt, DestructionPolicy, Outcome, Resource, ResourceError, TaskState, Test, TestResults,
};
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

const WAIT_TIMEOUT: Duration = Duration::from_secs(600);
const WAIT_CHECK: Duration = Duration::from_secs(10);

const RETRY_ATTEMPTS: usize = 5;

/// Wait until all of the created crds have completed and return a HashMap containing each `Test`'s
/// results.
pub(crate) async fn wait_for_crds(
    client: &TestManager,
    crds: &Vec<Crd>,
) -> Result<HashMap<String, Vec<TestResults>>> {
    // Get list of all dependencies of the created crds
    let dependencies: Vec<_> = crds
        .iter()
        .filter_map(|crd| match crd {
            Crd::Test(test) => Some(test.spec.resources.to_owned()),
            Crd::Resource(resource) => resource.spec.depends_on.to_owned(),
        })
        .flatten()
        .collect();

    // Get a list of all conflicting resources with the created crds
    let conflicting_resource_names: Vec<_> = crds
        .iter()
        .filter_map(|crd| match crd {
            Crd::Test(_) => None,
            Crd::Resource(resource) => resource.spec.conflicts_with.to_owned(),
        })
        .flatten()
        .collect();

    // Loop until all conflicting resources have been cleaned up.
    wait_for_conflicting_resources(client, &conflicting_resource_names).await?;

    // Follow the creation progress of all dependencies.
    for dependency in dependencies {
        wait_for_resource_creation(client, &dependency)
            .await?
            .context()?;
    }

    // For each CRD created, add a section to results with the test results.
    let mut results = HashMap::new();
    for crd in crds {
        results.extend(wait_for_crd(client, crd).await?.context()?)
    }

    Ok(results)
}

/// Wait until the conflicting resources are deleted.
async fn wait_for_conflicting_resources(client: &TestManager, resources: &[String]) -> Result<()> {
    // While any of the conflicting resources still exist
    while let Some(resource) = fetch_resources(client, resources).await?.pop() {
        let resource_name = resource.object_name();

        if matches!(
            resource.creation_task_state(),
            TaskState::Running | TaskState::Unknown
        ) {
            info!("Resource '{resource_name}' is still running.");
            continue;
        }

        if let Some(e) = resource.creation_error() {
            warn!("A resource '{resource_name}' had a creation error '{}' and will need to be manually deleted.", e)
        }

        if let Some(e) = resource.destruction_error() {
            warn!("A resource '{resource_name}' had a destruction error '{}' and will need to be manually deleted.", e)
        }

        // If a resource has been completed, determine why it hasn't been cleaned up yet.
        if resource.created_resource().is_some() {
            let dependent_tests = fetch_tests_using_resource(client, resource_name).await?;
            if let Some(still_running_test) = dependent_tests.iter().find(|test| {
                matches!(
                    test.agent_status().task_state,
                    TaskState::Running | TaskState::Unknown
                )
            }) {
                info!("A conflicting resource '{resource_name}' is waiting for a dependent test '{}' to be completed.", still_running_test.object_name());
                continue;
            }
            if let Some(failed_test) = dependent_tests
                .iter()
                .find(|test| matches!(test.agent_status().task_state, TaskState::Error))
            {
                if matches!(
                    resource.spec.destruction_policy,
                    DestructionPolicy::OnTestSuccess
                ) {
                    warn!("A conflicting resource '{resource_name}' with `DestructionPolicy:OnTestSuccess' is waiting for a failed test '{}' to be deleted. The test will need to be deleted manually.", failed_test.object_name());

                    // If it's stuck on a test give the user a few minutes to correct before erroring.
                    tokio::time::timeout(
                        WAIT_TIMEOUT,
                        wait_for_test_correction(client, failed_test.object_name()),
                    )
                    .await
                    .context(error::WaitTimeoutSnafu {
                        what: format!("'{}' to be corrected", failed_test.object_name()),
                    })??;
                    continue;
                }
            }

            if matches!(
                resource.spec.destruction_policy,
                DestructionPolicy::OnDeletion | DestructionPolicy::Never
            ) {
                warn!("A resource '{resource_name}' is completed, but has 'DestructionPolicy:{}' and will need to be manually deleted.", resource.spec.destruction_policy)
            }
        }

        warn!(
            "Waiting {} seconds for '{resource_name}' to be deleted.",
            WAIT_TIMEOUT.as_secs()
        );
        // If it's stuck on a resource give the user a few minutes to delete it before erroring.
        tokio::time::timeout(
            WAIT_TIMEOUT,
            wait_for_resource_destruction(client, resource_name),
        )
        .await
        .context(error::WaitTimeoutSnafu {
            what: format!("'{}' to be deleted", resource_name),
        })??;
    }

    Ok(())
}

/// Wait until the resource has completed destruction.
async fn wait_for_resource_destruction(client: &TestManager, resource_name: &str) -> Result<()> {
    loop {
        if fetch_resources(client, &[resource_name.to_string()])
            .await?
            .is_empty()
        {
            return Ok(());
        }

        info!(
            "Still waiting for resource '{resource_name}' to be deleted. Sleeping {}s",
            WAIT_CHECK.as_secs()
        );
        tokio::time::sleep(WAIT_CHECK).await;
    }
}

/// Wait until the errored test has been restarted or deleted.
async fn wait_for_test_correction(client: &TestManager, test_name: &str) -> Result<()> {
    let test_client = client.test_client();
    loop {
        if let Some(test) = test_client
            .get(test_name)
            .await
            .allow_not_found(|_| info!("The test '{test_name}' was deleted. Continuing ... "))
            .context(error::TestsysClientSnafu)?
        {
            if test.agent_status().task_state != TaskState::Error
                && (test.agent_status().results.is_empty()
                    || test
                        .agent_status()
                        .results
                        .iter()
                        .any(|res| res.outcome == Outcome::Pass))
            {
                info!("The test '{test_name}' was restarted. Continuing ...")
            }
        }

        info!(
            "Still waiting for test '{test_name}' to be corrected. Sleeping {}s",
            WAIT_CHECK.as_secs()
        );
        tokio::time::sleep(WAIT_CHECK).await;
    }
}

/// Wait until the resource has completed creation.
async fn wait_for_resource_creation(
    client: &TestManager,
    resource_name: &str,
) -> Result<TestRunResult> {
    loop {
        let resource = fetch_resources(client, &[resource_name.to_string()])
            .await?
            .pop()
            .context(error::MissingSnafu {
                item: resource_name,
                what: "TestSys cluster",
            })?;

        if let Some(e) = resource.creation_error() {
            error!(
                "Resource '{}' had a creation error '{}'.",
                resource.object_name(),
                e
            );
            return Ok(TestRunResult::ResourceCreationError {
                resource: resource_name.to_string(),
                error: e.to_owned(),
            });
        }

        if matches!(resource.creation_task_state(), TaskState::Completed) {
            return Ok(TestRunResult::Successful {
                results: Default::default(),
            });
        }

        info!(
            "Still waiting for resource '{resource_name}' to be created. Sleeping {}s",
            WAIT_CHECK.as_secs()
        );
        tokio::time::sleep(WAIT_CHECK).await;
    }
}

/// Try to fetch the requested resources with retries. Missing resources will be ignored.
async fn fetch_resources(client: &TestManager, resource_names: &[String]) -> Result<Vec<Resource>> {
    let resource_client = &client.resource_client();
    Retry::spawn(
        ExponentialBackoff::from_millis(10)
            .map(jitter)
            .take(RETRY_ATTEMPTS),
        || async move {
            join_all(
                resource_names.iter().map(|resource_name| async {
                    resource_client.get(resource_name.clone()).await
                }),
            )
            .await
            .into_iter()
            .filter_map(|resource_result| {
                resource_result
                    .allow_not_found(|_| warn!("A resource could not be found."))
                    .context(error::TestsysClientSnafu)
                    .transpose()
            })
            .collect::<Result<Vec<Resource>>>()
        },
    )
    .await
}

/// Try to fetch the names of all tests that are dependent on this resource.
async fn fetch_tests_using_resource(
    client: &TestManager,
    resource_name: &str,
) -> Result<Vec<Test>> {
    let test_client = &client.test_client();

    let tests = Retry::spawn(
        ExponentialBackoff::from_millis(10)
            .map(jitter)
            .take(RETRY_ATTEMPTS),
        || async move { test_client.get_all().await },
    )
    .await
    .context(error::TestsysClientSnafu)?;

    Ok(tests
        .iter()
        .filter(|test| test.spec.resources.contains(&resource_name.to_string()))
        .cloned()
        .collect())
}

/// Wait until the test has completed.
async fn wait_for_test(client: &TestManager, test_name: &str) -> Result<TestRunResult> {
    let test_client = client.test_client();
    loop {
        let test = test_client
            .get(test_name)
            .await
            .context(error::TestsysClientSnafu)?;

        match test.agent_status().task_state {
            TaskState::Running | TaskState::Unknown => {
                info!(
                    "Test '{test_name} is still running. Sleeping {}s.",
                    WAIT_CHECK.as_secs()
                )
            }
            TaskState::Completed => {
                return Ok(TestRunResult::Successful {
                    results: vec![(test_name.to_string(), test.agent_status().results.clone())]
                        .into_iter()
                        .collect(),
                })
            }
            TaskState::Error => {
                return Ok(TestRunResult::TestError {
                    test: test_name.to_string(),
                    error: test.agent_error().unwrap_or_default().to_string(),
                })
            }
        }
        tokio::time::sleep(WAIT_CHECK).await;
    }
}

/// Wait until the CRD has completed and determine whether it was successful or not.
async fn wait_for_crd(client: &TestManager, crd: &Crd) -> Result<TestRunResult> {
    match crd {
        Crd::Test(test) => wait_for_test(client, test.object_name()).await,
        Crd::Resource(resource) => wait_for_resource_creation(client, resource.object_name()).await,
    }
}

#[derive(Debug)]
pub enum TestRunResult {
    ResourceCreationError {
        resource: String,
        error: ResourceError,
    },
    TestError {
        test: String,
        error: String,
    },
    Successful {
        results: HashMap<String, Vec<TestResults>>,
    },
}

impl TestRunResult {
    fn context(self) -> Result<HashMap<String, Vec<TestResults>>> {
        match self {
            TestRunResult::ResourceCreationError { resource, error } => {
                Err(error::Error::ResourceCreation {
                    resource_name: resource,
                    error,
                })
            }
            TestRunResult::TestError { test, error } => Err(error::Error::TestRun {
                test_name: test,
                error,
            }),
            TestRunResult::Successful { results } => Ok(results),
        }
    }
}
