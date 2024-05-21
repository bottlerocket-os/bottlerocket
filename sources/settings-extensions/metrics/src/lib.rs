/// The aws settings can be used to configure settings related to AWS
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use model_derive::model;
use modeled_types::Url;
use std::convert::Infallible;

// Platform-specific settings
#[model(impl_default = true)]
pub struct MetricsSettingsV1 {
    metrics_url: Url,
    send_metrics: bool,
    service_checks: Vec<String>,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for MetricsSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // allow anything that parses as MetricsSettingsV1
        Ok(())
    }

    fn generate(
        existing_partial: Option<Self::PartialKind>,
        _dependent_settings: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>> {
        Ok(GenerateResult::Complete(
            existing_partial.unwrap_or_default(),
        ))
    }

    fn validate(_value: Self, _validated_settings: Option<serde_json::Value>) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_metrics() {
        let generated = MetricsSettingsV1::generate(None, None).unwrap();
        assert_eq!(
            generated,
            GenerateResult::Complete(MetricsSettingsV1 {
                metrics_url: None,
                send_metrics: None,
                service_checks: None,
            })
        )
    }

    #[test]
    fn test_serde_metrics() {
        let test_json = r#"{"metrics-url":"https://metrics.bottlerocket.aws/v1/metrics","send-metrics":true,"service-checks":["apiserver","chronyd"]}"#;

        let metrics: MetricsSettingsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            metrics,
            MetricsSettingsV1 {
                metrics_url: Some(
                    Url::try_from("https://metrics.bottlerocket.aws/v1/metrics").unwrap()
                ),
                send_metrics: Some(true),
                service_checks: Some(vec![String::from("apiserver"), String::from("chronyd")])
            }
        );

        let results = serde_json::to_string(&metrics).unwrap();
        assert_eq!(results, test_json);
    }
}
