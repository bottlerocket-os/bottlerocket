use bloodhound::results::{CheckStatus, Checker, CheckerMetadata, CheckerResult, Mode};
use bloodhound::*;

const PROC_MODULES_FILE: &str = "/proc/modules";
const MODPROBE_CMD: &str = "/bin/modprobe";

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct BR01010101Checker {}

impl Checker for BR01010101Checker {
    fn execute(&self) -> CheckerResult {
        let module_result = check_file_contains!(
            PROC_MODULES_FILE,
            &[" udf,"],
            "unable to parse modules to check for udf",
            "udf is currently loaded"
        );
        if module_result.status != CheckStatus::PASS {
            return module_result;
        }
        check_output_contains!(
            MODPROBE_CMD,
            ["-n", "-v", "udf"],
            &["install /bin/true"],
            "unable to parse modprobe output to check if udf is enabled",
            "modprobe for udf is not disabled"
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure mounting of udf filesystems is disabled".to_string(),
            id: "1.1.1.1".to_string(),
            level: 2,
            name: "br01010101".to_string(),
            mode: Mode::Automatic,
        }
    }
}
